use crate::db::models::Channel;
use crate::http::get_http_client;
use anyhow::{Context, Result};
use backoff::{ExponentialBackoff, future::retry};
use log::warn;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Accepte un entier JSON ou une chaîne JSON et retourne une String.
/// Nécessaire car l'API Xtream Codes retourne parfois season_number comme entier.
fn de_string_or_int<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    struct StrOrInt;
    impl<'de> Visitor<'de> for StrOrInt {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("string or integer")
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<String, E> {
            Ok(v.to_owned())
        }
        fn visit_string<E: de::Error>(self, v: String) -> Result<String, E> {
            Ok(v)
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<String, E> {
            Ok(v.to_string())
        }
    }
    deserializer.deserialize_any(StrOrInt)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XtreamCredentials {
    pub server_url: String,
    pub username: String,
    pub password: String,
}

/// Accepte les trois formes de `episodes` renvoyées par les serveurs Xtream :
/// - objet `{"1": [...], "2": [...]}` (cas courant) ;
/// - liste de listes `[[...], [...]]` : PHP sérialise en liste un tableau dont les clés sont
///   0..n consécutives (ex. série avec une saison 0) — l'index est alors le numéro de saison ;
/// - liste plate d'épisodes `[{...}, ...]`, regroupée d'après le champ `season` ;
/// - liste vide `[]` (série sans épisode).
fn de_episodes<'de, D>(deserializer: D) -> Result<HashMap<String, Vec<Episode>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Episodes {
        BySeason(HashMap<String, Vec<Episode>>),
        Nested(Vec<Vec<Episode>>),
        Flat(Vec<Episode>),
    }

    Ok(match Episodes::deserialize(deserializer)? {
        Episodes::BySeason(map) => map,
        Episodes::Nested(seasons) => seasons
            .into_iter()
            .enumerate()
            .filter(|(_, eps)| !eps.is_empty())
            .map(|(i, eps)| (i.to_string(), eps))
            .collect(),
        Episodes::Flat(eps) => {
            let mut map: HashMap<String, Vec<Episode>> = HashMap::new();
            for ep in eps {
                map.entry(ep.season.to_string()).or_default().push(ep);
            }
            map
        }
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeriesInfo {
    pub seasons: Vec<Season>,
    pub info: SeriesMetadata,
    #[serde(deserialize_with = "de_episodes")]
    pub episodes: HashMap<String, Vec<Episode>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Season {
    pub id: String,
    pub name: String,
    #[serde(deserialize_with = "de_string_or_int")]
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
    #[allow(dead_code)]
    num: Option<i64>,
    name: String,
    #[serde(alias = "series_id")]
    stream_id: i64,
    #[serde(alias = "cover")]
    stream_icon: Option<String>,
    category_id: Option<String>,
    #[serde(rename = "type")]
    stream_type: Option<String>,
    /// Rating from provider (string "8" or integer 8 depending on server)
    #[serde(default, deserialize_with = "de_rating_opt")]
    rating: Option<f32>,
}

/// Accepte null / absent / string / number pour le champ rating.
fn de_rating_opt<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    struct RatingVisitor;
    impl<'de> Visitor<'de> for RatingVisitor {
        type Value = Option<f32>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("null, string or number")
        }
        fn visit_unit<E: de::Error>(self) -> Result<Option<f32>, E> { Ok(None) }
        fn visit_none<E: de::Error>(self) -> Result<Option<f32>, E> { Ok(None) }
        fn visit_some<D2: de::Deserializer<'de>>(self, d: D2) -> Result<Option<f32>, D2::Error> {
            self.visit_str(&String::deserialize(d)?)
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<f32>, E> {
            let v = v.trim();
            if v.is_empty() { return Ok(None); }
            v.parse::<f32>().map(Some).or(Ok(None))
        }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Option<f32>, E> {
            Ok(Some(v as f32))
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Option<f32>, E> {
            Ok(Some(v as f32))
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Option<f32>, E> {
            Ok(Some(v as f32))
        }
    }
    deserializer.deserialize_any(RatingVisitor)
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
    user_agent: Option<&str>,
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
    let live_categories = fetch_categories(creds, "get_live_categories", user_agent)
        .await
        .unwrap_or_default();
    let vod_categories = fetch_categories(creds, "get_vod_categories", user_agent)
        .await
        .unwrap_or_default();
    let series_categories = fetch_categories(creds, "get_series_categories", user_agent)
        .await
        .unwrap_or_default();

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
    let live_streams = fetch_live_streams(creds, &category_map, user_agent).await?;
    progress.live_count = live_streams.len();
    all_channels.extend(live_streams);
    progress_callback(progress.clone());

    // Fetch VOD streams
    let vod_streams = fetch_vod_streams(creds, &category_map, user_agent).await?;
    progress.vod_count = vod_streams.len();
    all_channels.extend(vod_streams);
    progress_callback(progress.clone());

    // Fetch series
    let series_streams = fetch_series(creds, &category_map, user_agent).await?;
    progress.series_count = series_streams.len();
    all_channels.extend(series_streams);
    progress_callback(progress.clone());

    Ok(all_channels)
}

/// Fetch channels from Xtream Codes API (without progress)
#[allow(dead_code)] // Convenience function for future use
pub async fn fetch_xtream_channels(creds: &XtreamCredentials) -> Result<Vec<Channel>> {
    fetch_xtream_channels_with_progress(creds, None, |_| {}).await
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
async fn fetch_json_with_retry<T: for<'de> Deserialize<'de>>(
    url: &str,
    action: &str,
    user_agent: Option<&str>,
) -> Result<T> {
    let url = url.to_string();
    let action = action.to_string();
    let user_agent = user_agent.map(|ua| ua.to_string());

    retry(create_backoff(), || {
        let url = url.clone();
        let action = action.clone();
        let user_agent = user_agent.clone();
        async move {
            let request = get_http_client().get(&url);
            let request = if let Some(ua) = user_agent.as_deref() {
                request.header(reqwest::header::USER_AGENT, ua)
            } else {
                request
            };

            let response = request
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

            let bytes = response.bytes().await.map_err(|e| {
                warn!("Failed to read {} response, retrying: {}", action, e);
                backoff::Error::transient(anyhow::anyhow!("Read failed: {}", e))
            })?;

            serde_json::from_slice::<T>(&bytes).map_err(|e| {
                if e.is_data() {
                    // Valid JSON but not the shape we expect: the server will answer the same
                    // way every time, retrying only delays the failure.
                    warn!("Unexpected {} response format (not retried): {}", action, e);
                    backoff::Error::permanent(anyhow::anyhow!("Unexpected format: {}", e))
                } else {
                    // Truncated / garbled body (e.g. overloaded server): worth another try
                    warn!("Failed to parse {} response, retrying: {}", action, e);
                    backoff::Error::transient(anyhow::anyhow!("Parse failed: {}", e))
                }
            })
        }
    })
    .await
    .with_context(|| format!("Failed to fetch {} from Xtream API after retries", action))
}

/// Fetch categories from Xtream API
async fn fetch_categories(
    creds: &XtreamCredentials,
    action: &str,
    user_agent: Option<&str>,
) -> Result<Vec<XtreamCategory>> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action={}",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password,
        action
    );

    fetch_json_with_retry(&url, action, user_agent).await
}

/// Fetch live TV streams
async fn fetch_live_streams(
    creds: &XtreamCredentials,
    category_map: &std::collections::HashMap<String, (String, i32)>,
    user_agent: Option<&str>,
) -> Result<Vec<Channel>> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_live_streams",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password
    );

    let response: Vec<XtreamStream> =
        fetch_json_with_retry(&url, "live streams", user_agent).await?;
    Ok(convert_streams_to_channels(creds, response, "live", category_map))
}

/// Fetch VOD streams
async fn fetch_vod_streams(
    creds: &XtreamCredentials,
    category_map: &std::collections::HashMap<String, (String, i32)>,
    user_agent: Option<&str>,
) -> Result<Vec<Channel>> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_vod_streams",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password
    );

    let response: Vec<XtreamStream> =
        fetch_json_with_retry(&url, "VOD streams", user_agent).await?;
    Ok(convert_streams_to_channels(creds, response, "vod", category_map))
}

/// Fetch series
async fn fetch_series(
    creds: &XtreamCredentials,
    category_map: &std::collections::HashMap<String, (String, i32)>,
    user_agent: Option<&str>,
) -> Result<Vec<Channel>> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_series",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password
    );

    let response: Vec<XtreamStream> = fetch_json_with_retry(&url, "series", user_agent).await?;
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
                playlist_id: 0,
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
                rating: stream.rating,
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

    fetch_json_with_retry(&url, "series info", None).await
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

// ========== VOD Info ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct VodInfo {
    pub info: VodMetadata,
    pub movie_data: VodMovieData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VodMetadata {
    pub name: Option<String>,
    #[serde(alias = "description")]
    pub plot: Option<String>,
    #[serde(alias = "actors")]
    pub cast: Option<String>,
    pub director: Option<String>,
    pub genre: Option<String>,
    pub rating: Option<String>,
    pub duration: Option<String>,
    #[serde(alias = "cover_big")]
    pub movie_image: Option<String>,
    pub backdrop_path: Option<Vec<String>>,
    #[serde(alias = "releasedate", alias = "releaseDate")]
    pub release_date: Option<String>,
    pub youtube_trailer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VodMovieData {
    pub stream_id: Option<i64>,
    pub name: Option<String>,
    pub container_extension: Option<String>,
}

/// Fetch VOD movie information (metadata, poster, description…)
pub async fn fetch_vod_info(creds: &XtreamCredentials, vod_id: i64) -> Result<VodInfo> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_vod_info&vod_id={}",
        creds.server_url.trim_end_matches('/'),
        creds.username,
        creds.password,
        vod_id
    );

    fetch_json_with_retry(&url, "vod info", None).await
}

#[cfg(test)]
mod series_info_tests {
    use super::*;

    fn ep(id: u32, season: i32, num: i32) -> String {
        format!(
            r#"{{"id":"{id}","episode_num":{num},"title":"E{num}","container_extension":"mkv","season":{season},"info":{{}}}}"#
        )
    }

    fn series_json(episodes: &str) -> String {
        format!(r#"{{"seasons":[],"info":{{"name":"Test"}},"episodes":{episodes}}}"#)
    }

    #[test]
    fn episodes_as_object_keyed_by_season() {
        let json = series_json(&format!(r#"{{"1":[{}],"2":[{},{}]}}"#, ep(1, 1, 1), ep(2, 2, 1), ep(3, 2, 2)));
        let info: SeriesInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.episodes["1"].len(), 1);
        assert_eq!(info.episodes["2"].len(), 2);
    }

    /// PHP turns an array with keys 0..n into a JSON list (series with a season 0, e.g. Big Mouth)
    #[test]
    fn episodes_as_list_of_lists_uses_index_as_season() {
        let json = series_json(&format!(r#"[[{}],[{},{}]]"#, ep(1, 0, 1), ep(2, 1, 1), ep(3, 1, 2)));
        let info: SeriesInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.episodes["0"].len(), 1);
        assert_eq!(info.episodes["1"].len(), 2);
    }

    #[test]
    fn episodes_as_flat_list_grouped_by_season_field() {
        let json = series_json(&format!(r#"[{},{},{}]"#, ep(1, 1, 1), ep(2, 2, 1), ep(3, 1, 2)));
        let info: SeriesInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.episodes["1"].len(), 2);
        assert_eq!(info.episodes["2"].len(), 1);
    }

    #[test]
    fn episodes_as_empty_list() {
        let info: SeriesInfo = serde_json::from_str(&series_json("[]")).unwrap();
        assert!(info.episodes.is_empty());
    }
}
