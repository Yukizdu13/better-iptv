use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;
use anyhow::{Context, Result};
use log::{debug, info, warn};
use regex::Regex;
use wait_timeout::ChildExt;

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

/// Mask sensitive credentials in URLs for safe logging
/// Replaces username and password parameters with ***
fn mask_sensitive_data(input: &str) -> String {
    // Mask username parameter: ?username=...&  or &username=...&
    let masked = Regex::new(r"([?&]username=)[^&]*")
        .unwrap()
        .replace_all(input, "${1}***");

    // Mask password parameter: ?password=...&  or &password=...&
    Regex::new(r"([?&]password=)[^&]*")
        .unwrap()
        .replace_all(&masked, "${1}***")
        .to_string()
}

/// Get the path to MPV executable
///
/// On Windows, checks for bundled MPV first, then falls back to system PATH.
/// On macOS/Linux, uses system MPV only.
fn get_mpv_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Try bundled MPV first (Windows only)
        // Bundled MPV is in resources/mpv/mpv.exe relative to the executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Check if mpv.exe exists in the bundle
                let bundled_mpv = exe_dir.join("mpv").join("mpv.exe");
                if bundled_mpv.exists() {
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

/// MPV player controller using external process approach
pub struct MpvPlayer {
    process: Option<Child>,
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
        Self { process: None }
    }

    /// Check if MPV is installed on the system
    pub fn check_installed() -> bool {
        Command::new(get_mpv_path())
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Apply default MPV arguments for high-quality playback
    fn apply_default_args(cmd: &mut Command) {
        cmd.arg("--hwdec=auto")
            .arg("--vo=gpu-next")
            .arg("--profile=high-quality")
            .arg("--msg-level=all=error")
            .arg("--cache=yes")
            .arg("--cache-secs=30")
            .arg("--demuxer-max-bytes=100M");
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
        let safe_args = args.iter().map(|arg| mask_sensitive_data(arg)).collect::<Vec<_>>();

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
        let mut cmd = Command::new(get_mpv_path());
        Self::apply_default_args(&mut cmd);
        Self::apply_playback_options(&mut cmd, &MpvPlaybackOptions {
            title,
            audio_lang,
            subtitle_lang,
        });

        // Add URL
        cmd.arg(url);

        // Log and spawn
        Self::log_command(&cmd, None);
        self.spawn_mpv(&mut cmd)
    }

    /// Play a playlist of URLs (for series episodes)
    pub fn play_with_playlist(
        &mut self,
        urls: &[String],
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
        let mut cmd = Command::new(get_mpv_path());
        Self::apply_default_args(&mut cmd);
        Self::apply_playback_options(&mut cmd, &MpvPlaybackOptions {
            title,
            audio_lang,
            subtitle_lang,
        });

        // Add all URLs as arguments (MPV will play them in sequence)
        for url in urls {
            cmd.arg(url);
        }

        // Log and spawn
        Self::log_command(&cmd, Some(urls.len()));
        self.spawn_mpv(&mut cmd)
    }

    /// Stop current playback with timeout to prevent hanging
    pub fn stop(&mut self) -> Result<()> {
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

    #[test]
    fn test_validate_url_length() {
        let long_url = format!("http://example.com/{}", "a".repeat(5000));
        assert!(validate_stream_url(&long_url).is_err());
    }

    #[test]
    fn test_mask_sensitive_data() {
        let url = "http://server.com?username=secret&password=hunter2&action=play";
        let masked = mask_sensitive_data(url);
        assert!(!masked.contains("secret"));
        assert!(!masked.contains("hunter2"));
        assert!(masked.contains("username=***"));
        assert!(masked.contains("password=***"));
    }
}
