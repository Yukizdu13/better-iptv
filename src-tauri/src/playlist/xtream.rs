use crate::db::models::Channel;
use crate::http::get_http_client;
use anyhow::{Context, Result};
use backoff::{ExponentialBackoff, future::retry};
use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct XtreamCredentials {
    pub server_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeriesInfo {
    pub seasons: Vec<Season>,
    pub info: SeriesMetadata,
    pub episodes: HashMap<String, Vec<Episode>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Season {
    pub id: String,
    pub name: String,
    pub season_number: String,
    pub episode_count: i32,
    pub air_date: Option<String>,
    pub overview: Option<String>,
    pub cover: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub episode_num: i32,
    pub title: String,
    pub container_extension: String,
    pub season: i32,
    pub info: EpisodeInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpisodeInfo {
    pub plot: Option<String>,
    pub movie_image: Option<String>,
    #[serde(rename = "releaseDate")]
    pub release_date: Option<String>,
    pub duration: Option<String>,
    pub rating: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeriesMetadata {
    pub name: String,
    pub cover: Option<String>,
    pub plot: Option<String>,
    pub cast: Option<String>,
    pub director: Option<String>,
    pub genre: Option<String>,
    #[serde(rename = "releaseDate")]
    pub release_date: Option<String>,
    pub rating: Option<String>,
    pub backdrop_path: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct XtreamStream {
    #[allow(dead_code)] // May be used in future for stream ordering
    num: Option<i64>,
    name: String,
    #[serde(alias = "series_id")]
    stream_id: i64,
    stream_icon: Option<String>,
    category_id: Option<String>,
    #[serde(rename = "type")]
    stream_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct XtreamCategory {
    category_id: String,
    category_name: String,
}

/// Progress information during channel fetching
#[derive(Debug, Clone, Serialize)]
pub struct FetchProgress {
    pub live_count: usize,
    pub vod_count: usize,
    pub series_count: usize,
}

/// Fetch channels from Xtream Codes API with progress callback
pub async fn fetch_xtream_channels_with_progress<F>(
    creds: &XtreamCredentials,
    mut progress_callback: F,
) -> Result<Vec<Channel>>
where
    F: FnMut(FetchProgress),
{
    use std::collections::HashMap;

    let mut all_channels = Vec::new();
    let mut progress = FetchProgress {
        live_count: 0,
        vod_count: 0,
        series_count: 0,
    };

    // Fetch all categories first to build category_id -> (category_name, order) map
    // Each content type has its own ordering starting from 0
    let live_categories = fetch_categories(creds, "get_live_categories").await.unwrap_or_default();
    let vod_categories = fetch_categories(creds, "get_vod_categories").await.unwrap_or_default();
    let series_categories = fetch_categories(creds, "get_series_categories").await.unwrap_or_default();

    // Build map with (name, order) - order is the index in the provider's list
    let mut category_map: HashMap<String, (String, i32)> = HashMap::new();
    for (idx, cat) in live_categories.into_iter().enumerate() {
        category_map.insert(cat.category_id, (cat.category_name, idx as i32));
    }
    for (idx, cat) in vod_categories.into_iter().enumerate() {
        category_map.insert(cat.category_id, (cat.category_name, idx as i32));
    }
    for (idx, cat) in series_categories.into_iter().enumerate() {
        category_map.insert(cat.category_id, (cat.category_name, idx as i32));
    }

    // Fetch live streams
    let live_streams = fetch_live_streams(creds, &category_map).await?;
    progress.live_count = live_streams.len();
    all_channels.extend(live_streams);
    progress_callback(progress.clone());

    // Fetch VOD streams
    let vod_streams = fetch_vod_streams(creds, &category_map).await?;
    progress.vod_count = vod_streams.len();
    all_channels.extend(vod_streams);
    progress_callback(progress.clone());

    // Fetch series
    let series_streams = fetch_series(creds, &category_map).await?;
    progress.series_count = series_streams.len();
    all_channels.extend(series_streams);
    progress_callback(progress.clone());

    Ok(all_channels)
}

/// Fetch channels from Xtream Codes API (without progress)
#[allow(dead_code)] // Convenience function for future use
pub async fn fetch_xtream_channels(creds: &XtreamCredentials) -> Result<Vec<Channel>> {
    fetch_xtream_channels_with_progress(creds, |_| {}).await
}

/// Create exponential backoff configuration for API retries
fn create_backoff() -> ExponentialBackoff {
    ExponentialBackoff {
        initial_interval: Duration::from_millis(500),
        max_interval: Duration::from_secs(10),
        max_elapsed_time: Some(Duration::from_secs(60)),
        multiplier: 2.0,
        ..ExponentialBackoff::default()
    }
}

/// Fetch JSON from URL with retry logic
async fn fetch_json_with_retry<T: for<'de> Deserialize<'de>>(url: &str, action: &str) -> Result<T> {
    let url = url.to_string();
    let action = action.to_string();

    retry(create_backoff(), || {
        let url = url.clone();
        let action = action.clone();
        async move {
            let response = get_http_client()
                .get(&url)
                .send()
                .await
                .map_err(|e| {
                    warn!("Xtream API {} failed, retrying: {}", action, e);
                    backoff::Error::transient(anyhow::anyhow!("Request failed: {}", e))
                })?;

            // Check for HTTP errors
            if !response.status().is_success() {
                let status = response.status();
                warn!("Xtream API {} returned status {}, retrying", action, status);
                return Err(backoff::Error::transient(anyhow::anyhow!(
                    "HTTP error: {}",
                    status
                )));
            }

            response
                .json::<T>()
                .await
                .map_err(|e| {
                    warn!("Failed to parse {} response, retrying: {}", action, e);
                    backoff::Error::transient(anyhow::anyhow!("Parse failed: {}", e))
                })
        }
    })
    .await
    .with_context(|| format!("Failed to fetch {} from Xtream API after retries", action))
}

/// Fetch categories from Xtream API
async fn fetch_categories(creds: &XtreamCredentials, action: &str) -> Result<Vec<XtreamCategory>> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action={}",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password,
        action
    );

    fetch_json_with_retry(&url, action).await
}

/// Fetch live TV streams
async fn fetch_live_streams(
    creds: &XtreamCredentials,
    category_map: &std::collections::HashMap<String, (String, i32)>,
) -> Result<Vec<Channel>> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_live_streams",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password
    );

    let response: Vec<XtreamStream> = fetch_json_with_retry(&url, "live streams").await?;
    Ok(convert_streams_to_channels(creds, response, "live", category_map))
}

/// Fetch VOD streams
async fn fetch_vod_streams(
    creds: &XtreamCredentials,
    category_map: &std::collections::HashMap<String, (String, i32)>,
) -> Result<Vec<Channel>> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_vod_streams",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password
    );

    let response: Vec<XtreamStream> = fetch_json_with_retry(&url, "VOD streams").await?;
    Ok(convert_streams_to_channels(creds, response, "vod", category_map))
}

/// Fetch series
async fn fetch_series(
    creds: &XtreamCredentials,
    category_map: &std::collections::HashMap<String, (String, i32)>,
) -> Result<Vec<Channel>> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_series",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password
    );

    let response: Vec<XtreamStream> = fetch_json_with_retry(&url, "series").await?;
    Ok(convert_streams_to_channels(creds, response, "series", category_map))
}

// Use shared EPG ID generation from utils module
use crate::utils::generate_epg_id_swedish;

/// Convert Xtream streams to our Channel format
fn convert_streams_to_channels(
    creds: &XtreamCredentials,
    streams: Vec<XtreamStream>,
    default_content_type: &str,
    category_map: &std::collections::HashMap<String, (String, i32)>,
) -> Vec<Channel> {
    streams
        .into_iter()
        .enumerate()
        .map(|(idx, stream)| {
            // Use stream_type from API if available, otherwise use default
            let content_type = match stream.stream_type.as_deref() {
                Some("live") => "live",
                Some("movie") => "vod",
                Some("series") => "series",
                _ => default_content_type,
            };

            let url = build_stream_url(creds, stream.stream_id, content_type);

            // Generate EPG ID for live channels
            let epg_id = if content_type == "live" {
                generate_epg_id_swedish(&stream.name)
            } else {
                None
            };

            // Look up category name and order from category_id
            let (group_name, category_order) = stream
                .category_id
                .as_ref()
                .and_then(|id| category_map.get(id))
                .map(|(name, order)| (Some(name.clone()), *order))
                .unwrap_or((None, i32::MAX)); // Uncategorized items go last

            Channel {
                id: None,
                playlist_id: 0, // Will be set when inserting to database
                name: stream.name,
                url,
                logo: stream.stream_icon,
                group_name,
                epg_id,
                tvg_name: None,
                content_type: content_type.to_string(),
                is_favorite: false,
                sort_order: idx as i32,
                category_order,
                created_at: None,
            }
        })
        .collect()
}

/// Build stream URL for Xtream Codes
fn build_stream_url(creds: &XtreamCredentials, stream_id: i64, content_type: &str) -> String {
    let base_url = creds.server_url.trim_end_matches('/');

    match content_type {
        "live" => format!(
            "{}/live/{}/{}/{}.m3u8",
            base_url, creds.username, creds.password, stream_id
        ),
        "vod" => format!(
            "{}/movie/{}/{}/{}.mp4",
            base_url, creds.username, creds.password, stream_id
        ),
        "series" => format!(
            "{}/series/{}/{}/{}.mp4",
            base_url, creds.username, creds.password, stream_id
        ),
        _ => format!(
            "{}/live/{}/{}/{}.m3u8",
            base_url, creds.username, creds.password, stream_id
        ),
    }
}

/// Generate M3U URL from Xtream credentials
#[allow(dead_code)] // Alternative playlist format for future use
pub fn get_xtream_m3u_url(creds: &XtreamCredentials) -> String {
    format!(
        "{}/get.php?username={}&password={}&type=m3u_plus&output=ts",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password
    )
}

/// Generate EPG (XMLTV) URL from Xtream credentials
/// Most Xtream providers serve EPG data at /xmltv.php endpoint
pub fn get_xtream_epg_url(creds: &XtreamCredentials) -> String {
    format!(
        "{}/xmltv.php?username={}&password={}",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password
    )
}

/// Fetch series information (seasons and episodes)
pub async fn fetch_series_info(creds: &XtreamCredentials, series_id: i64) -> Result<SeriesInfo> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_series_info&series_id={}",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password,
        series_id
    );

    fetch_json_with_retry(&url, "series info").await
}

/// Build episode playback URL
#[allow(dead_code)] // Helper function for future episode playback features
pub fn build_episode_url(creds: &XtreamCredentials, episode_id: &str, extension: &str) -> String {
    format!(
        "{}/series/{}/{}/{}.{}",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password,
        episode_id,
        extension
    )
}
