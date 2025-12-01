/// Utility functions shared across modules

/// Country code configuration for EPG ID generation
/// Can be extended to support more countries
#[derive(Debug, Clone, Default)]
pub struct EpgConfig {
    pub country_suffix: String,
    pub country_code: String,
}

impl EpgConfig {
    /// Create config for Swedish channels (default)
    pub fn swedish() -> Self {
        Self {
            country_suffix: " SE".to_string(),
            country_code: "se".to_string(),
        }
    }
}

/// Generate EPG ID from channel name
///
/// Examples:
/// - "SVT1 HD SE" -> "SVT1 HD.se"
/// - "TV4 FHD SE" -> "TV4.se"
/// - "Discovery Channel" -> None (not a Swedish channel)
///
/// # Arguments
/// * `channel_name` - The full channel name
/// * `config` - EPG configuration with country settings
pub fn generate_epg_id(channel_name: &str, config: &EpgConfig) -> Option<String> {
    // Check if channel matches the configured country
    let is_target_country = channel_name.ends_with(&config.country_suffix)
        || channel_name.ends_with(&format!(" FHD{}", config.country_suffix))
        || channel_name.ends_with(&format!(" HD{}", config.country_suffix))
        || channel_name.ends_with(&format!(" SD{}", config.country_suffix));

    if !is_target_country {
        return None;
    }

    // Remove quality suffixes and country code to get clean name
    let clean_name = channel_name
        .trim_end_matches(&config.country_suffix)
        .trim_end_matches(" FHD")
        .trim_end_matches(" HD")
        .trim_end_matches(" SD")
        .trim();

    Some(format!("{}.{}", clean_name, config.country_code))
}

/// Generate EPG ID with default Swedish configuration
pub fn generate_epg_id_swedish(channel_name: &str) -> Option<String> {
    generate_epg_id(channel_name, &EpgConfig::swedish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_epg_id_swedish() {
        let config = EpgConfig::swedish();

        // HD suffix is stripped along with SE
        assert_eq!(
            generate_epg_id("SVT1 HD SE", &config),
            Some("SVT1.se".to_string())
        );

        // FHD suffix is stripped
        assert_eq!(
            generate_epg_id("TV4 FHD SE", &config),
            Some("TV4.se".to_string())
        );

        // Simple name without quality suffix
        assert_eq!(
            generate_epg_id("Kanal 5 SE", &config),
            Some("Kanal 5.se".to_string())
        );

        // SD suffix is stripped
        assert_eq!(
            generate_epg_id("TV3 SD SE", &config),
            Some("TV3.se".to_string())
        );

        // Non-Swedish channel should return None
        assert_eq!(generate_epg_id("BBC One", &config), None);
        assert_eq!(generate_epg_id("CNN International", &config), None);
    }

    #[test]
    fn test_generate_epg_id_swedish_shorthand() {
        // HD suffix is stripped
        assert_eq!(
            generate_epg_id_swedish("SVT2 HD SE"),
            Some("SVT2.se".to_string())
        );

        // Without quality suffix
        assert_eq!(
            generate_epg_id_swedish("Discovery SE"),
            Some("Discovery.se".to_string())
        );
    }
}
