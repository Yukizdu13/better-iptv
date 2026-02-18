use lazy_static::lazy_static;
use reqwest::Client;
use std::time::Duration;

pub const DEFAULT_HTTP_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Better-IPTV/2.1.1";
const TIVIMATE_HTTP_USER_AGENT: &str = "TiviMate/4.7.0 (Linux;Android 10) ExoPlayerLib/2.18.1";
const VLC_HTTP_USER_AGENT: &str = "VLC/3.0.20 LibVLC/3.0.20";
pub const MAX_CUSTOM_USER_AGENT_LENGTH: usize = 512;

lazy_static! {
    /// Shared HTTP client with custom user-agent for all external requests
    static ref HTTP_CLIENT: Client = create_http_client();
}

/// Create HTTP client with custom user-agent and reasonable timeouts
fn create_http_client() -> Client {
    Client::builder()
        .user_agent(DEFAULT_HTTP_USER_AGENT)
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client")
}

pub fn is_valid_playlist_user_agent_mode(mode: &str) -> bool {
    matches!(mode, "default" | "tivimate" | "vlc" | "custom")
}

pub fn resolve_playlist_user_agent(mode: Option<&str>, custom_value: Option<&str>) -> String {
    match mode.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("tivimate") => TIVIMATE_HTTP_USER_AGENT.to_string(),
        Some("vlc") => VLC_HTTP_USER_AGENT.to_string(),
        Some("custom") => custom_value
            .and_then(normalize_custom_user_agent)
            .unwrap_or_else(|| DEFAULT_HTTP_USER_AGENT.to_string()),
        _ => DEFAULT_HTTP_USER_AGENT.to_string(),
    }
}

pub fn normalize_custom_user_agent(value: &str) -> Option<String> {
    let normalized = value.trim();
    if normalized.is_empty()
        || normalized.len() > MAX_CUSTOM_USER_AGENT_LENGTH
        || normalized.contains('\r')
        || normalized.contains('\n')
    {
        return None;
    }

    Some(normalized.to_string())
}

/// Get the shared HTTP client for making requests
pub fn get_http_client() -> &'static Client {
    &HTTP_CLIENT
}
