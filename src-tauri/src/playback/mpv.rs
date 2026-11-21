use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};
use anyhow::{Context, Result};
use log::{debug, info, warn};
use wait_timeout::ChildExt;

use crate::utils::mask_credentials;

/// Allowed URL schemes for stream playback
const ALLOWED_SCHEMES: &[&str] = &["http://", "https://", "rtsp://", "rtmp://", "rtp://", "udp://"];

/// Characters that could be used for shell injection
const FORBIDDEN_CHARS: &[char] = &['`', '$', ';', '|', '&', '>', '<', '\n', '\r', '\0'];

/// Maximum allowed URL length
const MAX_URL_LENGTH: usize = 4096;

/// Validate a stream URL before passing to MPV
///
/// Checks:
/// - URL scheme is whitelisted (http, https, rtsp, rtmp, rtp, udp)
/// - No shell metacharacters that could enable command injection
/// - Reasonable URL length
fn validate_stream_url(url: &str) -> Result<()> {
    // Check URL length
    if url.len() > MAX_URL_LENGTH {
        return Err(anyhow::anyhow!("URL exceeds maximum length of {} characters", MAX_URL_LENGTH));
    }

    // Check for allowed scheme
    let has_valid_scheme = ALLOWED_SCHEMES.iter().any(|scheme| url.starts_with(scheme));
    if !has_valid_scheme {
        warn!("Rejected URL with invalid scheme: {}", url.chars().take(50).collect::<String>());
        return Err(anyhow::anyhow!(
            "Invalid URL scheme. Allowed: http, https, rtsp, rtmp, rtp, udp"
        ));
    }

    // Check for forbidden characters (shell injection prevention)
    if url.chars().any(|c| FORBIDDEN_CHARS.contains(&c)) {
        warn!("Rejected URL containing forbidden characters");
        return Err(anyhow::anyhow!("URL contains forbidden characters"));
    }

    Ok(())
}

/// Get the path to MPV executable
///
/// On Windows, checks for bundled MPV first, then falls back to system PATH.
/// On macOS/Linux, uses system MPV only.
fn get_mpv_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Try bundled MPV first (Windows only)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // `bundle.resources = ["../resources/mpv/*"]` : Tauri remplace `..` par `_up_`,
                // l'installeur (NSIS/MSI) dépose donc MPV dans `<exe>/_up_/resources/mpv/`
                // (vérifié sur l'installeur NSIS 2026-11-21). `<exe>/mpv/` reste accepté.
                let candidates = [
                    exe_dir.join("_up_").join("resources").join("mpv").join("mpv.exe"),
                    exe_dir.join("mpv").join("mpv.exe"),
                ];
                if let Some(bundled_mpv) = candidates.into_iter().find(|p| p.exists()) {
                    info!("Using bundled MPV at: {}", bundled_mpv.display());
                    return bundled_mpv;
                }
            }
        }

        // Fallback to system MPV
        info!("Bundled MPV not found, falling back to system MPV");
        PathBuf::from("mpv.exe")
    }

    #[cfg(not(target_os = "windows"))]
    {
        // macOS and Linux: Use system MPV only
        PathBuf::from("mpv")
    }
}

/// Build a unique path for the MPV IPC socket/pipe for this process.
///
/// Unix: a socket file under the system temp dir.
/// Windows: a named pipe (opened as a regular file through the `\\.\pipe\` namespace).
fn ipc_socket_path() -> PathBuf {
    #[cfg(unix)]
    {
        std::env::temp_dir().join(format!("better-iptv-mpv-{}.sock", std::process::id()))
    }
    #[cfg(windows)]
    {
        PathBuf::from(format!(r"\\.\pipe\better-iptv-mpv-{}", std::process::id()))
    }
}

/// How long an IPC command keeps retrying while MPV is still starting up / loading the file
const IPC_READY_TIMEOUT: Duration = Duration::from_secs(8);

/// Delay between two IPC attempts while waiting for MPV to be ready
const IPC_RETRY_DELAY: Duration = Duration::from_millis(150);

/// Max time to wait for MPV's reply on an open IPC connection
const IPC_READ_TIMEOUT: Duration = Duration::from_secs(2);

/// Content of the input.conf given to MPV (see `MpvPlayer::apply_input_conf`)
fn input_conf_contents() -> String {
    format!(
        "RIGHT seek 10\nLEFT seek -10\n\
         c expand-properties set {CANCEL_CREDITS_PROPERTY} ${{playlist-pos}}; \
         show-text \"Générique conservé\" 2000\n"
    )
}

/// Snapshot of MPV playback state, read over IPC
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct PlaybackStatus {
    /// Current position in seconds (None while the file is loading)
    pub time_pos: Option<f64>,
    /// Duration of the current file in seconds (None for live streams / while loading)
    pub duration: Option<f64>,
    /// Index of the current entry in MPV's playlist (series episodes queue)
    pub playlist_pos: Option<i64>,
    pub paused: bool,
    /// Playlist index on which the user pressed the "keep the credits" key in MPV's window
    /// (see `CANCEL_CREDITS_PROPERTY`), None if never pressed
    pub cancel_credits_pos: Option<i64>,
}

/// MPV user-data property set by the `c` key binding (value = playlist index at press time).
/// The app reads it back through `playback_status` to cancel the credits auto-skip.
const CANCEL_CREDITS_PROPERTY: &str = "user-data/better-iptv/cancel-credits";

/// Error returned by a single IPC round-trip
#[derive(Debug)]
enum IpcError {
    /// Socket not created yet / MPV gone — worth retrying while MPV starts
    Connect(std::io::Error),
    /// MPV answered with an error (e.g. "property unavailable" while the file loads)
    Mpv(String),
    /// I/O or protocol failure on an established connection
    Other(anyhow::Error),
}

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpcError::Connect(e) => write!(f, "cannot connect to MPV IPC: {}", e),
            IpcError::Mpv(e) => write!(f, "MPV error: {}", e),
            IpcError::Other(e) => write!(f, "MPV IPC failure: {}", e),
        }
    }
}

/// Send one JSON IPC command to MPV and wait for its reply (matched by request_id).
///
/// Returns the reply's `data` field on success.
fn ipc_request(path: &Path, command: &[serde_json::Value]) -> Result<serde_json::Value, IpcError> {
    static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let request_id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let mut line = serde_json::json!({ "command": command, "request_id": request_id }).to_string();
    line.push('\n');

    #[cfg(unix)]
    let (mut writer, reader) = {
        use std::os::unix::net::UnixStream;
        let stream = UnixStream::connect(path).map_err(IpcError::Connect)?;
        stream
            .set_read_timeout(Some(IPC_READ_TIMEOUT))
            .map_err(|e| IpcError::Other(e.into()))?;
        let reader = stream.try_clone().map_err(|e| IpcError::Other(e.into()))?;
        (stream, reader)
    };
    #[cfg(windows)]
    let (mut writer, reader) = {
        use std::fs::OpenOptions;
        let pipe = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(IpcError::Connect)?;
        let reader = pipe.try_clone().map_err(|e| IpcError::Other(e.into()))?;
        (pipe, reader)
    };

    writer
        .write_all(line.as_bytes())
        .map_err(|e| IpcError::Other(e.into()))?;

    // MPV may interleave event lines ({"event": ...}) before our reply
    for reply in BufReader::new(reader).lines() {
        let reply = reply.map_err(|e| IpcError::Other(e.into()))?;
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&reply) else {
            continue;
        };
        if value.get("request_id").and_then(|v| v.as_u64()) != Some(request_id) {
            continue;
        }
        return match value.get("error").and_then(|v| v.as_str()) {
            Some("success") => Ok(value.get("data").cloned().unwrap_or(serde_json::Value::Null)),
            Some(err) => Err(IpcError::Mpv(err.to_string())),
            None => Err(IpcError::Other(anyhow::anyhow!("malformed MPV reply"))),
        };
    }
    Err(IpcError::Other(anyhow::anyhow!("MPV closed the IPC connection")))
}

/// Like `ipc_request`, but keeps retrying while MPV is starting up or still loading the file.
///
/// Right after spawn the socket does not exist yet, and until the stream is opened MPV
/// rejects seeks ("error running command") and property reads ("property unavailable").
fn ipc_request_when_ready(
    path: &Path,
    command: &[serde_json::Value],
    timeout: Duration,
) -> Result<serde_json::Value> {
    let deadline = Instant::now() + timeout;
    loop {
        match ipc_request(path, command) {
            Ok(data) => return Ok(data),
            Err(IpcError::Other(e)) => return Err(e),
            Err(err) if Instant::now() >= deadline => {
                return Err(anyhow::anyhow!("{}", err));
            }
            Err(err) => {
                debug!("MPV not ready yet ({}), retrying", err);
                std::thread::sleep(IPC_RETRY_DELAY);
            }
        }
    }
}

/// Seek relative to the current position, waiting for MPV to be ready if needed.
///
/// Also shows "+10s" / "-10s" in MPV's own window (OSD), since that is where the video is.
pub fn seek_relative(path: &Path, seconds: f64) -> Result<()> {
    ipc_request_when_ready(
        path,
        &[serde_json::json!("seek"), serde_json::json!(seconds), serde_json::json!("relative")],
        IPC_READY_TIMEOUT,
    )
    .context("Seek failed")?;
    // Cosmetic only: ignore failures
    let _ = ipc_request(
        path,
        &[
            serde_json::json!("show-text"),
            serde_json::json!(format!("{:+}s", seconds.round() as i64)),
            serde_json::json!(1000),
        ],
    );
    Ok(())
}

/// Toggle pause/play
pub fn toggle_pause(path: &Path) -> Result<()> {
    ipc_request(path, &[serde_json::json!("cycle"), serde_json::json!("pause")])
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(())
}

/// Show a message in MPV's own window (OSD) for `duration_ms`
pub fn show_text(path: &Path, text: &str, duration_ms: u64) -> Result<()> {
    ipc_request(
        path,
        &[serde_json::json!("show-text"), serde_json::json!(text), serde_json::json!(duration_ms)],
    )
    .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(())
}

/// Jump to the next entry of MPV's playlist (next queued episode)
pub fn playlist_next(path: &Path) -> Result<()> {
    ipc_request(path, &[serde_json::json!("playlist-next")])
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(())
}

/// Read the current playback status. Properties MPV can't provide yet are left as None.
pub fn playback_status(path: &Path) -> Result<PlaybackStatus> {
    let get = |name: &str| ipc_request(path, &[serde_json::json!("get_property"), serde_json::json!(name)]);

    // A connection failure means there is no usable MPV session at all
    let time_pos = match get("time-pos") {
        Ok(v) => v.as_f64(),
        Err(IpcError::Mpv(_)) => None,
        Err(e) => return Err(anyhow::anyhow!("{}", e)),
    };
    Ok(PlaybackStatus {
        time_pos,
        duration: get("duration").ok().and_then(|v| v.as_f64()),
        playlist_pos: get("playlist-pos").ok().and_then(|v| v.as_i64()).filter(|p| *p >= 0),
        paused: get("pause").ok().and_then(|v| v.as_bool()).unwrap_or(false),
        // Set through `expand-properties set`, so MPV stores it as a string
        cancel_credits_pos: get(CANCEL_CREDITS_PROPERTY).ok().and_then(|v| {
            v.as_i64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
        }),
    })
}

/// MPV player controller using external process approach
pub struct MpvPlayer {
    process: Option<Child>,
    /// IPC socket/pipe path for the currently running MPV process, if any
    ipc_path: Option<PathBuf>,
    /// Rendering quality preset (CPU/GPU load vs. image quality)
    quality: VideoQuality,
}

/// Rendering quality preset, chosen in Settings > Playback (`video_quality`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoQuality {
    /// Cheapest scalers: lowest CPU/GPU load, fans stay quiet
    Economy,
    /// MPV defaults (no `high-quality` profile) — good picture, moderate load
    #[default]
    Balanced,
    /// `--profile=high-quality` + gpu-next: heavy on an integrated GPU
    High,
}

impl VideoQuality {
    /// Parse the stored setting; anything unknown falls back to the default
    pub fn from_setting(value: Option<&str>) -> Self {
        match value {
            Some("economy") => Self::Economy,
            Some("high") => Self::High,
            _ => Self::Balanced,
        }
    }
}

/// MPV playback options
struct MpvPlaybackOptions<'a> {
    title: Option<&'a str>,
    audio_lang: Option<&'a str>,
    subtitle_lang: Option<&'a str>,
}

impl MpvPlayer {
    /// Create a new MPV player instance
    pub fn new() -> Self {
        Self { process: None, ipc_path: None, quality: VideoQuality::default() }
    }

    /// Check if MPV is installed on the system
    pub fn check_installed() -> bool {
        Command::new(get_mpv_path())
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Choose the rendering quality preset for the next playback
    pub fn set_video_quality(&mut self, quality: VideoQuality) {
        self.quality = quality;
    }

    /// Apply default MPV arguments (quality preset, cache limits)
    fn apply_default_args(cmd: &mut Command, quality: VideoQuality) {
        cmd.arg("--hwdec=auto").arg("--msg-level=all=error");
        // Never open a window larger than 90% of the screen: the controls (title bar, OSC)
        // would end up off-screen and force a manual resize
        cmd.arg("--autofit-larger=90%x90%");
        match quality {
            VideoQuality::Economy => {
                cmd.arg("--vo=gpu")
                    .arg("--profile=fast")
                    .arg("--scale=bilinear")
                    .arg("--dscale=bilinear")
                    .arg("--cscale=bilinear")
                    .arg("--deband=no");
            }
            VideoQuality::Balanced => {
                cmd.arg("--vo=gpu");
            }
            VideoQuality::High => {
                cmd.arg("--vo=gpu-next").arg("--profile=high-quality");
            }
        }
        cmd.arg("--cache=yes")
            .arg("--cache-secs=30")
            // Many IPTV servers don't support HTTP range requests: MPV then disables
            // seeking entirely. Forcing it keeps +/-10s working within the cache.
            .arg("--force-seekable=yes")
            // Bounded RAM: forward cache + already-played data kept for backward seeks
            .arg("--demuxer-max-bytes=100M")
            .arg("--demuxer-max-back-bytes=50M");
    }

    /// Key bindings for MPV's own window (where the video is shown): arrows seek by 10s,
    /// matching the +10s/-10s buttons of the main window; `c` keeps the credits.
    fn apply_input_conf(cmd: &mut Command) {
        let path = std::env::temp_dir().join(format!("better-iptv-input-{}.conf", std::process::id()));
        if std::fs::write(&path, input_conf_contents()).is_ok() {
            cmd.arg(format!("--input-conf={}", path.display()));
        }
    }

    /// Enable the JSON IPC control socket used for seek/pause commands
    fn apply_ipc_arg(cmd: &mut Command, ipc_path: &std::path::Path) {
        cmd.arg(format!("--input-ipc-server={}", ipc_path.display()));
    }

    /// Apply playback options (title, audio/subtitle language)
    fn apply_playback_options(cmd: &mut Command, options: &MpvPlaybackOptions) {
        // Add title if provided
        if let Some(title_str) = options.title {
            cmd.arg(format!("--title={}", title_str));
        }

        // Add audio language preference if provided
        if let Some(lang) = options.audio_lang {
            if !lang.is_empty() {
                cmd.arg(format!("--alang={}", lang));
            }
        }

        // Add subtitle language preference if provided
        if let Some(lang) = options.subtitle_lang {
            if !lang.is_empty() {
                cmd.arg(format!("--slang={}", lang));
            }
        }
    }

    /// Log MPV command with masked credentials
    fn log_command(cmd: &Command, episode_count: Option<usize>) {
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        let safe_args = args.iter().map(|arg| mask_credentials(arg)).collect::<Vec<_>>();

        if let Some(count) = episode_count {
            info!("MPV playlist command: {} episodes", count);
        }
        debug!("MPV command: mpv {}", safe_args.join(" "));
    }

    /// Spawn MPV process and store handle
    fn spawn_mpv(&mut self, cmd: &mut Command) -> Result<()> {
        let child = cmd
            .spawn()
            .context("Failed to spawn MPV process. Is MPV installed?")?;

        self.process = Some(child);
        Ok(())
    }

    /// Play a stream URL with optional title
    #[allow(dead_code)] // Convenience method for future use
    pub fn play(&mut self, url: &str) -> Result<()> {
        self.play_with_title(url, None, None, None)
    }

    /// Play a stream URL with a specific title and language preferences
    pub fn play_with_title(
        &mut self,
        url: &str,
        title: Option<&str>,
        audio_lang: Option<&str>,
        subtitle_lang: Option<&str>,
    ) -> Result<()> {
        // Validate URL before passing to MPV
        validate_stream_url(url).context("Stream URL validation failed")?;

        // Stop any existing playback
        self.stop()?;

        // Build MPV command
        let ipc_path = ipc_socket_path();
        let mut cmd = Command::new(get_mpv_path());
        Self::apply_default_args(&mut cmd, self.quality);
        Self::apply_input_conf(&mut cmd);
        Self::apply_ipc_arg(&mut cmd, &ipc_path);
        Self::apply_playback_options(&mut cmd, &MpvPlaybackOptions {
            title,
            audio_lang,
            subtitle_lang,
        });

        // Add URL
        cmd.arg(url);

        // Log and spawn
        Self::log_command(&cmd, None);
        self.spawn_mpv(&mut cmd)?;
        self.ipc_path = Some(ipc_path);
        Ok(())
    }

    /// Play a playlist of URLs (for series episodes).
    ///
    /// `start_seconds` resumes the FIRST episode at that position (the others start at 0).
    pub fn play_with_playlist(
        &mut self,
        urls: &[String],
        start_seconds: Option<f64>,
        title: Option<&str>,
        audio_lang: Option<&str>,
        subtitle_lang: Option<&str>,
    ) -> Result<()> {
        if urls.is_empty() {
            return Err(anyhow::anyhow!("Cannot play empty playlist"));
        }

        // Validate all URLs before passing to MPV
        for (i, url) in urls.iter().enumerate() {
            validate_stream_url(url)
                .with_context(|| format!("Stream URL validation failed for episode {}", i + 1))?;
        }

        // Stop any existing playback
        self.stop()?;

        // Build MPV command
        let ipc_path = ipc_socket_path();
        let mut cmd = Command::new(get_mpv_path());
        Self::apply_default_args(&mut cmd, self.quality);
        Self::apply_input_conf(&mut cmd);
        Self::apply_ipc_arg(&mut cmd, &ipc_path);
        Self::apply_playback_options(&mut cmd, &MpvPlaybackOptions {
            title,
            audio_lang,
            subtitle_lang,
        });

        // Add all URLs as arguments (MPV will play them in sequence).
        // A per-file option group (--{ ... --}) scopes --start to the first entry only.
        for (i, url) in urls.iter().enumerate() {
            match start_seconds.filter(|s| i == 0 && *s > 0.0) {
                Some(start) => {
                    cmd.arg("--{")
                        .arg(format!("--start={:.1}", start))
                        .arg(url)
                        .arg("--}");
                }
                None => {
                    cmd.arg(url);
                }
            }
        }

        // Log and spawn
        Self::log_command(&cmd, Some(urls.len()));
        self.spawn_mpv(&mut cmd)?;
        self.ipc_path = Some(ipc_path);
        Ok(())
    }

    /// IPC socket/pipe path of the running MPV process, if any
    pub fn ipc_path(&mut self) -> Option<PathBuf> {
        if self.is_playing() {
            self.ipc_path.clone()
        } else {
            None
        }
    }

    /// Stop current playback with timeout to prevent hanging
    pub fn stop(&mut self) -> Result<()> {
        self.ipc_path = None;
        if let Some(mut child) = self.process.take() {
            // Send kill signal
            let _ = child.kill();

            // Wait with timeout (5 seconds) to prevent indefinite blocking
            match child.wait_timeout(Duration::from_secs(5))? {
                Some(_status) => {
                    // Process terminated normally
                }
                None => {
                    // Process didn't respond within timeout, force kill
                    warn!("MPV process did not respond to kill signal, forcing termination");
                    let _ = child.kill();
                    // Try waiting again briefly
                    let _ = child.wait_timeout(Duration::from_secs(1));
                }
            }
        }
        Ok(())
    }

    /// Check if currently playing
    pub fn is_playing(&mut self) -> bool {
        if let Some(child) = &mut self.process {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    self.process = None;
                    false
                }
                Ok(None) => true, // Still running
                Err(_) => {
                    self.process = None;
                    false
                }
            }
        } else {
            false
        }
    }
}

impl Drop for MpvPlayer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_installed() {
        // This will pass if MPV is installed on the system
        let installed = MpvPlayer::check_installed();
        debug!("MPV installed: {}", installed);
    }

    #[test]
    fn test_validate_url_valid_schemes() {
        assert!(validate_stream_url("http://example.com/stream.m3u8").is_ok());
        assert!(validate_stream_url("https://example.com/stream.m3u8").is_ok());
        assert!(validate_stream_url("rtsp://example.com/live").is_ok());
        assert!(validate_stream_url("rtmp://example.com/live").is_ok());
        assert!(validate_stream_url("rtp://example.com:1234").is_ok());
        assert!(validate_stream_url("udp://239.0.0.1:1234").is_ok());
    }

    #[test]
    fn test_validate_url_invalid_schemes() {
        assert!(validate_stream_url("file:///etc/passwd").is_err());
        assert!(validate_stream_url("ftp://example.com/file").is_err());
        assert!(validate_stream_url("javascript:alert(1)").is_err());
        assert!(validate_stream_url("/etc/passwd").is_err());
        assert!(validate_stream_url("./local/file").is_err());
    }

    #[test]
    fn test_validate_url_shell_injection() {
        assert!(validate_stream_url("http://example.com/`whoami`").is_err());
        assert!(validate_stream_url("http://example.com/$HOME").is_err());
        assert!(validate_stream_url("http://example.com/;rm -rf /").is_err());
        assert!(validate_stream_url("http://example.com/|cat /etc/passwd").is_err());
        assert!(validate_stream_url("http://example.com/&id").is_err());
        assert!(validate_stream_url("http://example.com/\nmalicious").is_err());
    }

    /// Fake MPV IPC server: answers each request with an event line first, then the reply
    #[cfg(unix)]
    fn fake_mpv(replies: Vec<serde_json::Value>) -> (tempdir_path::Guard, PathBuf) {
        use std::os::unix::net::UnixListener;
        let guard = tempdir_path::Guard::new();
        let path = guard.path.clone();
        let listener = UnixListener::bind(&path).unwrap();
        std::thread::spawn(move || {
            for mut reply in replies {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let req: serde_json::Value = serde_json::from_str(&line).unwrap();
                reply["request_id"] = req["request_id"].clone();
                let mut w = stream;
                writeln!(w, "{}", serde_json::json!({"event": "playback-restart"})).unwrap();
                writeln!(w, "{}", reply).unwrap();
            }
        });
        (guard, path)
    }

    #[cfg(unix)]
    mod tempdir_path {
        pub struct Guard {
            pub path: std::path::PathBuf,
        }
        impl Guard {
            pub fn new() -> Self {
                static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let path = std::env::temp_dir()
                    .join(format!("better-iptv-test-{}-{}.sock", std::process::id(), n));
                let _ = std::fs::remove_file(&path);
                Self { path }
            }
        }
        impl Drop for Guard {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.path);
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_ipc_request_skips_events_and_returns_data() {
        let (_g, path) = fake_mpv(vec![serde_json::json!({"error": "success", "data": 42.5})]);
        let data = ipc_request(&path, &[serde_json::json!("get_property"), serde_json::json!("time-pos")])
            .unwrap();
        assert_eq!(data.as_f64(), Some(42.5));
    }

    #[cfg(unix)]
    #[test]
    fn test_seek_retries_until_file_loaded() {
        // MPV rejects the seek while the stream is still opening, then accepts it
        let (_g, path) = fake_mpv(vec![
            serde_json::json!({"error": "error running command"}),
            serde_json::json!({"error": "error running command"}),
            serde_json::json!({"error": "success", "data": null}),
        ]);
        assert!(seek_relative(&path, 10.0).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn test_seek_fails_without_mpv() {
        let guard = tempdir_path::Guard::new();
        let err = ipc_request_when_ready(
            &guard.path,
            &[serde_json::json!("seek"), serde_json::json!(10), serde_json::json!("relative")],
            Duration::from_millis(400),
        );
        assert!(err.is_err());
    }

    #[test]
    fn test_input_conf_binds_seek_and_cancel_credits() {
        let conf = input_conf_contents();
        assert!(conf.contains("RIGHT seek 10\n"));
        assert!(conf.contains("LEFT seek -10\n"));
        assert!(conf.contains(
            "c expand-properties set user-data/better-iptv/cancel-credits ${playlist-pos}; show-text"
        ));
    }

    #[cfg(unix)]
    #[test]
    fn test_playback_status_reads_cancel_credits_as_string_or_number() {
        // time-pos, duration, playlist-pos, pause, cancel-credits (string, as MPV stores it)
        let (_g, path) = fake_mpv(vec![
            serde_json::json!({"error": "success", "data": 12.0}),
            serde_json::json!({"error": "success", "data": 1300.0}),
            serde_json::json!({"error": "success", "data": 3}),
            serde_json::json!({"error": "success", "data": false}),
            serde_json::json!({"error": "success", "data": "3"}),
        ]);
        let status = playback_status(&path).unwrap();
        assert_eq!(status.playlist_pos, Some(3));
        assert_eq!(status.cancel_credits_pos, Some(3));
    }

    #[test]
    fn test_validate_url_length() {
        let long_url = format!("http://example.com/{}", "a".repeat(5000));
        assert!(validate_stream_url(&long_url).is_err());
    }

}
