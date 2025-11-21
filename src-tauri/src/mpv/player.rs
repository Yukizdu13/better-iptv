use std::process::{Child, Command};
use anyhow::{Context, Result};
use log::{debug, info};
use regex::Regex;

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

/// MPV player controller using external process approach
pub struct MpvPlayer {
    process: Option<Child>,
}

impl MpvPlayer {
    /// Create a new MPV player instance
    pub fn new() -> Self {
        Self { process: None }
    }

    /// Check if MPV is installed on the system
    pub fn check_installed() -> bool {
        Command::new("mpv")
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Play a stream URL with optional title
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
        // Stop any existing playback
        self.stop()?;

        // Build MPV command
        let mut cmd = Command::new("mpv");
        cmd.arg("--hwdec=auto")
            .arg("--vo=gpu-next")
            .arg("--profile=high-quality")
            .arg("--msg-level=all=error")
            .arg("--cache=yes")
            .arg("--cache-secs=30")
            .arg("--demuxer-max-bytes=100M");

        // Add title if provided
        if let Some(title_str) = title {
            cmd.arg(format!("--title={}", title_str));
        }

        // Add audio language preference if provided
        if let Some(lang) = audio_lang {
            if !lang.is_empty() {
                cmd.arg(format!("--alang={}", lang));
            }
        }

        // Add subtitle language preference if provided
        if let Some(lang) = subtitle_lang {
            if !lang.is_empty() {
                cmd.arg(format!("--slang={}", lang));
            }
        }

        // Add URL
        cmd.arg(url);

        // Log the command with masked credentials
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        let safe_args = args.iter().map(|arg| mask_sensitive_data(arg)).collect::<Vec<_>>();
        debug!("MPV command: mpv {}", safe_args.join(" "));

        // Spawn MPV process
        let child = cmd
            .spawn()
            .context("Failed to spawn MPV process. Is MPV installed?")?;

        self.process = Some(child);
        Ok(())
    }

    /// Play a playlist of URLs (for series episodes)
    pub fn play_with_playlist(
        &mut self,
        urls: &[String],
        title: Option<&str>,
        audio_lang: Option<&str>,
        subtitle_lang: Option<&str>,
    ) -> Result<()> {
        // Stop any existing playback
        self.stop()?;

        if urls.is_empty() {
            return Err(anyhow::anyhow!("Cannot play empty playlist"));
        }

        // Build MPV command
        let mut cmd = Command::new("mpv");
        cmd.arg("--hwdec=auto")
            .arg("--vo=gpu-next")
            .arg("--profile=high-quality")
            .arg("--msg-level=all=error")
            .arg("--cache=yes")
            .arg("--cache-secs=30")
            .arg("--demuxer-max-bytes=100M");

        // Add title if provided (will show for first episode)
        if let Some(title_str) = title {
            cmd.arg(format!("--title={}", title_str));
        }

        // Add audio language preference if provided
        if let Some(lang) = audio_lang {
            if !lang.is_empty() {
                cmd.arg(format!("--alang={}", lang));
            }
        }

        // Add subtitle language preference if provided
        if let Some(lang) = subtitle_lang {
            if !lang.is_empty() {
                cmd.arg(format!("--slang={}", lang));
            }
        }

        // Add all URLs as arguments (MPV will play them in sequence)
        for url in urls {
            cmd.arg(url);
        }

        // Log the command with masked credentials
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        let safe_args = args.iter().map(|arg| mask_sensitive_data(arg)).collect::<Vec<_>>();
        info!("MPV playlist command: {} episodes", urls.len());
        debug!("MPV command: mpv {}", safe_args.join(" "));

        // Spawn MPV process
        let child = cmd
            .spawn()
            .context("Failed to spawn MPV process. Is MPV installed?")?;

        self.process = Some(child);
        Ok(())
    }

    /// Stop current playback
    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.process.take() {
            child.kill().context("Failed to kill MPV process")?;
            child.wait().context("Failed to wait for MPV process")?;
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
}
