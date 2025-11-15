use std::process::{Child, Command};
use anyhow::{Context, Result};

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

    /// Play a stream URL
    pub fn play(&mut self, url: &str) -> Result<()> {
        // Stop any existing playback
        self.stop()?;

        // Spawn MPV process
        let child = Command::new("mpv")
            .arg("--keep-open")
            .arg("--force-window=yes")
            .arg("--cache=yes")
            .arg("--cache-secs=30")
            .arg("--demuxer-max-bytes=100M")
            .arg("--hwdec=auto")
            .arg("--vo=gpu")
            .arg(url)
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
        println!("MPV installed: {}", installed);
    }
}
