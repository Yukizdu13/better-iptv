use crate::db::models::Channel;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    num: Option<i64>,
    name: String,
    #[serde(alias = "series_id")]
    stream_id: i64,
    stream_icon: Option<String>,
    category_id: Option<String>,
    category_name: Option<String>,
    #[serde(rename = "type")]
    stream_type: Option<String>,
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
    let mut all_channels = Vec::new();
    let mut progress = FetchProgress {
        live_count: 0,
        vod_count: 0,
        series_count: 0,
    };

    // Fetch live streams
    let live_streams = fetch_live_streams(creds).await?;
    progress.live_count = live_streams.len();
    all_channels.extend(live_streams);
    progress_callback(progress.clone());

    // Fetch VOD streams
    let vod_streams = fetch_vod_streams(creds).await?;
    progress.vod_count = vod_streams.len();
    all_channels.extend(vod_streams);
    progress_callback(progress.clone());

    // Fetch series
    let series_streams = fetch_series(creds).await?;
    progress.series_count = series_streams.len();
    all_channels.extend(series_streams);
    progress_callback(progress.clone());

    Ok(all_channels)
}

/// Fetch channels from Xtream Codes API (without progress)
pub async fn fetch_xtream_channels(creds: &XtreamCredentials) -> Result<Vec<Channel>> {
    fetch_xtream_channels_with_progress(creds, |_| {}).await
}

/// Fetch live TV streams
async fn fetch_live_streams(creds: &XtreamCredentials) -> Result<Vec<Channel>> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_live_streams",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password
    );

    let response = reqwest::get(&url)
        .await
        .context("Failed to fetch live streams from Xtream API")?
        .json::<Vec<XtreamStream>>()
        .await
        .context("Failed to parse live streams response")?;

    Ok(convert_streams_to_channels(creds, response, "live"))
}

/// Fetch VOD streams
async fn fetch_vod_streams(creds: &XtreamCredentials) -> Result<Vec<Channel>> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_vod_streams",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password
    );

    let response = reqwest::get(&url)
        .await
        .context("Failed to fetch VOD streams from Xtream API")?
        .json::<Vec<XtreamStream>>()
        .await
        .context("Failed to parse VOD streams response")?;

    Ok(convert_streams_to_channels(creds, response, "vod"))
}

/// Fetch series
async fn fetch_series(creds: &XtreamCredentials) -> Result<Vec<Channel>> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_series",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password
    );

    let response = reqwest::get(&url)
        .await
        .context("Failed to fetch series from Xtream API")?
        .json::<Vec<XtreamStream>>()
        .await
        .context("Failed to parse series response")?;

    Ok(convert_streams_to_channels(creds, response, "series"))
}

/// Generate EPG ID from channel name (e.g., "SVT1 HD SE" -> "SVT1 HD.se")
fn generate_epg_id(channel_name: &str) -> Option<String> {
    // Remove quality suffixes and convert to EPG format
    let clean_name = channel_name
        .trim_end_matches(" SE")
        .trim_end_matches(" FHD")
        .trim_end_matches(" HD")
        .trim_end_matches(" SD")
        .trim();

    // Only generate EPG ID if the name ends with "SE" (Swedish channels)
    if channel_name.ends_with(" SE") || channel_name.ends_with(" FHD SE") || channel_name.ends_with(" HD SE") {
        Some(format!("{}.se", clean_name))
    } else {
        None
    }
}

/// Convert Xtream streams to our Channel format
fn convert_streams_to_channels(
    creds: &XtreamCredentials,
    streams: Vec<XtreamStream>,
    default_content_type: &str,
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
                generate_epg_id(&stream.name)
            } else {
                None
            };

            Channel {
                id: None,
                playlist_id: 0, // Will be set when inserting to database
                name: stream.name,
                url,
                logo: stream.stream_icon,
                group_name: stream.category_name,
                epg_id,
                tvg_name: None,
                content_type: content_type.to_string(),
                is_favorite: false,
                sort_order: idx as i32,
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
pub fn get_xtream_m3u_url(creds: &XtreamCredentials) -> String {
    format!(
        "{}/get.php?username={}&password={}&type=m3u_plus&output=ts",
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

    let response = reqwest::get(&url)
        .await
        .context("Failed to fetch series info from Xtream API")?
        .json::<SeriesInfo>()
        .await
        .context("Failed to parse series info response")?;

    Ok(response)
}

/// Build episode playback URL
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
