//! Series domain business logic
//!
//! This module contains pure business logic for series/VOD operations.
//! Functions here are synchronous and do NOT include database operations.
//! Database operations remain in the commands layer.

use crate::error::AppError;
use serde::{Deserialize, Serialize};

/// Episode data for playlist playback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistEpisode {
    pub id: String,
    pub title: String,
    pub extension: String,
}

/// Validate episode list
///
/// # Errors
/// Returns `AppError::InvalidInput` if the episode list is empty
pub fn validate_episodes(episodes: &[PlaylistEpisode]) -> Result<(), AppError> {
    if episodes.is_empty() {
        return Err(AppError::InvalidInput("No episodes provided".to_string()));
    }
    Ok(())
}

/// Build episode URLs from server info and episodes
///
/// Constructs streaming URLs for each episode using the Xtream Codes API format:
/// `{server_url}/series/{username}/{password}/{episode_id}.{extension}`
///
/// # Arguments
/// * `server_url` - Base server URL (trailing slashes are stripped)
/// * `username` - Xtream Codes username
/// * `password` - Xtream Codes password
/// * `episodes` - List of episodes to generate URLs for
///
/// # Returns
/// Vector of formatted URLs, one per episode in the same order
pub fn build_episode_urls(
    server_url: &str,
    username: &str,
    password: &str,
    episodes: &[PlaylistEpisode],
) -> Vec<String> {
    episodes
        .iter()
        .map(|episode| {
            format!(
                "{}/series/{}/{}/{}.{}",
                server_url.trim_end_matches('/'),
                username,
                password,
                episode.id,
                episode.extension
            )
        })
        .collect()
}

/// Validate server URL format
///
/// Ensures the server URL is properly formatted and not empty
///
/// # Errors
/// Returns `AppError::InvalidInput` if URL is empty or invalid
pub fn validate_server_url(url: &str) -> Result<(), AppError> {
    if url.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Server URL cannot be empty".to_string(),
        ));
    }

    // Basic URL validation - must start with http:// or https://
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(AppError::InvalidInput(
            "Server URL must start with http:// or https://".to_string(),
        ));
    }

    Ok(())
}

/// Validate Xtream credentials
///
/// Ensures username and password are not empty
///
/// # Errors
/// Returns `AppError::InvalidInput` if username or password is empty
pub fn validate_credentials(username: &str, password: &str) -> Result<(), AppError> {
    if username.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Username cannot be empty".to_string(),
        ));
    }

    if password.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Password cannot be empty".to_string(),
        ));
    }

    Ok(())
}

/// Extract first episode title from episode list
///
/// Used for window title when playing a series
///
/// # Arguments
/// * `episodes` - List of episodes
///
/// # Returns
/// Title of the first episode, or a default message if list is empty
#[allow(dead_code)]
pub fn get_first_episode_title(episodes: &[PlaylistEpisode]) -> String {
    episodes
        .first()
        .map(|ep| ep.title.clone())
        .unwrap_or_else(|| "Unknown Episode".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_episodes_empty() {
        let result = validate_episodes(&[]);
        assert!(result.is_err());
        match result {
            Err(AppError::InvalidInput(msg)) => {
                assert_eq!(msg, "No episodes provided");
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_validate_episodes_valid() {
        let episodes = vec![PlaylistEpisode {
            id: "123".to_string(),
            title: "Episode 1".to_string(),
            extension: "mkv".to_string(),
        }];
        assert!(validate_episodes(&episodes).is_ok());
    }

    #[test]
    fn test_build_episode_urls() {
        let episodes = vec![
            PlaylistEpisode {
                id: "123".to_string(),
                title: "Episode 1".to_string(),
                extension: "mkv".to_string(),
            },
            PlaylistEpisode {
                id: "456".to_string(),
                title: "Episode 2".to_string(),
                extension: "mp4".to_string(),
            },
        ];

        let urls = build_episode_urls("http://example.com", "user", "pass", &episodes);

        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "http://example.com/series/user/pass/123.mkv");
        assert_eq!(urls[1], "http://example.com/series/user/pass/456.mp4");
    }

    #[test]
    fn test_build_episode_urls_trailing_slash() {
        let episodes = vec![PlaylistEpisode {
            id: "123".to_string(),
            title: "Episode 1".to_string(),
            extension: "mkv".to_string(),
        }];

        let urls = build_episode_urls("http://example.com/", "user", "pass", &episodes);

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "http://example.com/series/user/pass/123.mkv");
    }

    #[test]
    fn test_validate_server_url_empty() {
        assert!(validate_server_url("").is_err());
        assert!(validate_server_url("   ").is_err());
    }

    #[test]
    fn test_validate_server_url_no_protocol() {
        let result = validate_server_url("example.com");
        assert!(result.is_err());
        match result {
            Err(AppError::InvalidInput(msg)) => {
                assert!(msg.contains("http://"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_validate_server_url_valid() {
        assert!(validate_server_url("http://example.com").is_ok());
        assert!(validate_server_url("https://example.com").is_ok());
    }

    #[test]
    fn test_validate_credentials_empty_username() {
        let result = validate_credentials("", "pass");
        assert!(result.is_err());
        match result {
            Err(AppError::InvalidInput(msg)) => {
                assert!(msg.contains("Username"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_validate_credentials_empty_password() {
        let result = validate_credentials("user", "");
        assert!(result.is_err());
        match result {
            Err(AppError::InvalidInput(msg)) => {
                assert!(msg.contains("Password"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_validate_credentials_valid() {
        assert!(validate_credentials("user", "pass").is_ok());
    }

    #[test]
    fn test_get_first_episode_title() {
        let episodes = vec![
            PlaylistEpisode {
                id: "1".to_string(),
                title: "First Episode".to_string(),
                extension: "mkv".to_string(),
            },
            PlaylistEpisode {
                id: "2".to_string(),
                title: "Second Episode".to_string(),
                extension: "mkv".to_string(),
            },
        ];

        assert_eq!(get_first_episode_title(&episodes), "First Episode");
    }

    #[test]
    fn test_get_first_episode_title_empty() {
        assert_eq!(get_first_episode_title(&[]), "Unknown Episode");
    }
}
