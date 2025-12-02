use lazy_static::lazy_static;
use reqwest::Client;
use std::time::Duration;

lazy_static! {
    /// Shared HTTP client with custom user-agent for all external requests
    static ref HTTP_CLIENT: Client = create_http_client();
}

/// Create HTTP client with custom user-agent and reasonable timeouts
fn create_http_client() -> Client {
    Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Better-IPTV/2.1.1")
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client")
}

/// Get the shared HTTP client for making requests
pub fn get_http_client() -> &'static Client {
    &HTTP_CLIENT
}
