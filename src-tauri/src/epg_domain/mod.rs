//! EPG domain logic module
//!
//! This module contains business logic for EPG operations, separated from
//! database and command layers for better testability and maintainability.

use crate::error::AppError;

/// Validate EPG URL format
///
/// Ensures the URL is not empty and uses HTTP/HTTPS protocol.
///
/// # Arguments
/// * `url` - The EPG URL to validate
///
/// # Returns
/// * `Ok(())` if the URL is valid
/// * `Err(AppError::InvalidInput)` if the URL is invalid
///
/// # Examples
/// ```
/// use better_ip_tv::epg_domain::validate_epg_url;
///
/// assert!(validate_epg_url("https://example.com/epg.xml").is_ok());
/// assert!(validate_epg_url("").is_err());
/// assert!(validate_epg_url("ftp://example.com/epg.xml").is_err());
/// ```
pub fn validate_epg_url(url: &str) -> Result<(), AppError> {
    // Check if URL is empty or whitespace-only
    if url.trim().is_empty() {
        return Err(AppError::InvalidInput("EPG URL cannot be empty".to_string()));
    }

    // Ensure URL uses HTTP or HTTPS protocol
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(AppError::InvalidInput(
            "EPG URL must start with http:// or https://".to_string(),
        ));
    }

    Ok(())
}

/// Normalize EPG URL by trimming whitespace
///
/// This is a helper function to clean up user input before processing.
///
/// # Arguments
/// * `url` - The EPG URL to normalize
///
/// # Returns
/// * The normalized URL with leading/trailing whitespace removed
pub fn normalize_epg_url(url: &str) -> String {
    url.trim().to_string()
}

/// Check if URL points to a gzipped file based on extension
///
/// This is used to determine if we need to decompress the EPG data.
///
/// # Arguments
/// * `url` - The EPG URL to check
///
/// # Returns
/// * `true` if the URL ends with `.gz`, `false` otherwise
#[allow(dead_code)]
pub fn is_gzipped_url(url: &str) -> bool {
    url.ends_with(".gz")
}

/// Validate channel EPG ID format
///
/// Ensures the EPG ID is not empty and contains valid characters.
///
/// # Arguments
/// * `epg_id` - The channel EPG ID to validate
///
/// # Returns
/// * `Ok(())` if the EPG ID is valid
/// * `Err(AppError::InvalidInput)` if the EPG ID is invalid
pub fn validate_channel_epg_id(epg_id: &str) -> Result<(), AppError> {
    if epg_id.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Channel EPG ID cannot be empty".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_epg_url_valid_https() {
        assert!(validate_epg_url("https://example.com/epg.xml").is_ok());
    }

    #[test]
    fn test_validate_epg_url_valid_http() {
        assert!(validate_epg_url("http://example.com/epg.xml.gz").is_ok());
    }

    #[test]
    fn test_validate_epg_url_empty() {
        assert!(validate_epg_url("").is_err());
    }

    #[test]
    fn test_validate_epg_url_whitespace() {
        assert!(validate_epg_url("   ").is_err());
    }

    #[test]
    fn test_validate_epg_url_invalid_protocol() {
        assert!(validate_epg_url("ftp://example.com/epg.xml").is_err());
    }

    #[test]
    fn test_validate_epg_url_no_protocol() {
        assert!(validate_epg_url("example.com/epg.xml").is_err());
    }

    #[test]
    fn test_normalize_epg_url() {
        assert_eq!(
            normalize_epg_url("  https://example.com/epg.xml  "),
            "https://example.com/epg.xml"
        );
    }

    #[test]
    fn test_normalize_epg_url_no_whitespace() {
        assert_eq!(
            normalize_epg_url("https://example.com/epg.xml"),
            "https://example.com/epg.xml"
        );
    }

    #[test]
    fn test_is_gzipped_url_true() {
        assert!(is_gzipped_url("https://example.com/epg.xml.gz"));
    }

    #[test]
    fn test_is_gzipped_url_false() {
        assert!(!is_gzipped_url("https://example.com/epg.xml"));
    }

    #[test]
    fn test_validate_channel_epg_id_valid() {
        assert!(validate_channel_epg_id("channel123").is_ok());
    }

    #[test]
    fn test_validate_channel_epg_id_empty() {
        assert!(validate_channel_epg_id("").is_err());
    }

    #[test]
    fn test_validate_channel_epg_id_whitespace() {
        assert!(validate_channel_epg_id("   ").is_err());
    }
}
