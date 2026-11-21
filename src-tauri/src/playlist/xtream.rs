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

/// Accepte null / absent / chaîne / nombre / booléen et retourne `Option<String>`.
/// Les fournisseurs renvoient par exemple `rating` en chaîne ("7.5") ou en nombre (7.5)
/// selon la fiche (Atlas Pro : `get_vod_info`).
fn de_opt_string_lenient<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => Some(s),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    })
}

/// Accepte null / absent / entier / flottant / chaîne numérique pour un entier 32 bits
/// (0 si absent ou illisible).
fn de_i32_lenient<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    Ok(match Value::deserialize(deserializer)? {
        Value::Number(n) => n.as_i64().map(|v| v as i32).or_else(|| n.as_f64().map(|v| v as i32)),
        Value::String(s) => s.trim().parse::<i32>().ok(),
        _ => None,
    }
    .unwrap_or(0))
}

/// Accepte une liste de chaînes, une chaîne seule ou null (`backdrop_path`).
fn de_opt_string_vec_lenient<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) if !s.is_empty() => Some(vec![s]),
        Value::Array(items) => Some(
            items
                .into_iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect(),
        ),
        _ => None,
    })
}

/// Désérialise un objet `T`, ou `T::default()` si le champ est null, absent ou n'est pas un objet
/// (ex. `"info": []` renvoyé par PHP quand l'objet est vide). Une erreur sur un vrai objet reste
/// remontée : l'avaler donnerait une fiche vide sans aucun message.
fn de_object_or_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: serde::de::DeserializeOwned + Default,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    if value.is_object() {
        serde_json::from_value(value).map_err(serde::de::Error::custom)
    } else {
        Ok(T::default())
    }
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
    #[serde(default)]
    pub seasons: Vec<Season>,
    #[serde(default, deserialize_with = "de_object_or_default")]
    pub info: SeriesMetadata,
    #[serde(deserialize_with = "de_episodes")]
    pub episodes: HashMap<String, Vec<Episode>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Season {
    /// Entier chez certains fournisseurs (Atlas Pro), chaîne chez d'autres.
    #[serde(default, deserialize_with = "de_string_or_int")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(deserialize_with = "de_string_or_int")]
    pub season_number: String,
    #[serde(default, deserialize_with = "de_i32_lenient")]
    pub episode_count: i32,
    pub air_date: Option<String>,
    pub overview: Option<String>,
    pub cover: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Episode {
    #[serde(deserialize_with = "de_string_or_int")]
    pub id: String,
    #[serde(default, deserialize_with = "de_i32_lenient")]
    pub episode_num: i32,
    #[serde(default)]
    pub title: String,
    pub container_extension: String,
    #[serde(default, deserialize_with = "de_i32_lenient")]
    pub season: i32,
    #[serde(default, deserialize_with = "de_object_or_default")]
    pub info: EpisodeInfo,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EpisodeInfo {
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    pub plot: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    pub movie_image: Option<String>,
    #[serde(default, rename = "releaseDate", deserialize_with = "de_opt_string_lenient")]
    pub release_date: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    pub duration: Option<String>,
    /// Chaîne ("7.5"), entier ou flottant selon l'épisode.
    #[serde(default, deserialize_with = "de_rating_opt")]
    pub rating: Option<f32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SeriesMetadata {
    #[serde(default)]
    pub name: String,
    pub cover: Option<String>,
    pub plot: Option<String>,
    pub cast: Option<String>,
    pub director: Option<String>,
    pub genre: Option<String>,
    #[serde(default, rename = "releaseDate")]
    pub release_date: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    pub rating: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_vec_lenient")]
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
    /// Extension réelle du fichier VOD (mkv, avi, mp4...). Le serveur renvoie 404 si l'URL
    /// demande une autre extension que celle du fichier (Atlas Pro : 81 % de mkv/avi).
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    container_extension: Option<String>,
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

            let url = build_stream_url(
                creds,
                stream.stream_id,
                content_type,
                stream.container_extension.as_deref(),
            );

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
fn build_stream_url(
    creds: &XtreamCredentials,
    stream_id: i64,
    content_type: &str,
    container_extension: Option<&str>,
) -> String {
    let base_url = creds.server_url.trim_end_matches('/');
    // Extension propre au fichier pour la VOD ; mp4 si le fournisseur ne la donne pas
    let vod_ext = container_extension
        .map(|e| e.trim().trim_start_matches('.'))
        .filter(|e| !e.is_empty() && e.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or("mp4");

    match content_type {
        "live" => format!(
            "{}/live/{}/{}/{}.m3u8",
            base_url, creds.username, creds.password, stream_id
        ),
        "vod" => format!(
            "{}/movie/{}/{}/{}.{}",
            base_url, creds.username, creds.password, stream_id, vod_ext
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
    #[serde(default, deserialize_with = "de_object_or_default")]
    pub info: VodMetadata,
    #[serde(default, deserialize_with = "de_object_or_default")]
    pub movie_data: VodMovieData,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(from = "VodMetadataRaw")]
pub struct VodMetadata {
    pub name: Option<String>,
    pub plot: Option<String>,
    pub cast: Option<String>,
    pub director: Option<String>,
    pub genre: Option<String>,
    pub rating: Option<String>,
    pub duration: Option<String>,
    pub movie_image: Option<String>,
    pub backdrop_path: Option<Vec<String>>,
    pub release_date: Option<String>,
    pub youtube_trailer: Option<String>,
}

/// Fiche film telle que renvoyée par le fournisseur. Beaucoup envoient les deux variantes d'un
/// même champ à la fois (Atlas Pro : `plot` + `description`, `cast` + `actors`,
/// `movie_image` + `cover_big`) : un `alias` serde y provoquerait « duplicate field », d'où des
/// champs séparés fusionnés ensuite (première valeur non vide).
#[derive(Deserialize)]
struct VodMetadataRaw {
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    name: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    plot: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    description: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    cast: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    actors: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    director: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    genre: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    rating: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    duration: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    movie_image: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    cover_big: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_vec_lenient")]
    backdrop_path: Option<Vec<String>>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    releasedate: Option<String>,
    #[serde(default, rename = "releaseDate", deserialize_with = "de_opt_string_lenient")]
    release_date_camel: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_lenient")]
    youtube_trailer: Option<String>,
}

/// Première valeur non vide.
fn first_non_empty(values: [Option<String>; 3]) -> Option<String> {
    values.into_iter().flatten().find(|v| !v.trim().is_empty())
}

impl From<VodMetadataRaw> for VodMetadata {
    fn from(r: VodMetadataRaw) -> Self {
        VodMetadata {
            name: first_non_empty([r.name, None, None]),
            plot: first_non_empty([r.plot, r.description, None]),
            cast: first_non_empty([r.cast, r.actors, None]),
            director: first_non_empty([r.director, None, None]),
            genre: first_non_empty([r.genre, None, None]),
            rating: first_non_empty([r.rating, None, None]),
            duration: first_non_empty([r.duration, None, None]),
            movie_image: first_non_empty([r.movie_image, r.cover_big, None]),
            backdrop_path: r.backdrop_path.filter(|v| !v.is_empty()),
            release_date: first_non_empty([r.releasedate, r.release_date_camel, None]),
            youtube_trailer: first_non_empty([r.youtube_trailer, None, None]),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
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

/// Formes de réponses observées chez Atlas Pro (atlaspro.to, 2026-11-21), reproduites sans données réelles.
#[cfg(test)]
mod atlas_pro_tests {
    use super::*;

    fn creds() -> XtreamCredentials {
        XtreamCredentials {
            server_url: "http://example.com/".into(),
            username: "u".into(),
            password: "p".into(),
        }
    }

    /// `season.id` entier, `episode_count` entier, `rating` d'épisode en chaîne / entier / flottant,
    /// `info` d'épisode vide renvoyé comme liste `[]`, `info` de série sans `rating`.
    #[test]
    fn series_info_with_integer_season_id_and_mixed_ratings() {
        let json = r#"{
            "seasons":[{"id":1234,"name":"Saison 1","season_number":1,"episode_count":3,"air_date":null,"overview":"","cover":""}],
            "info":{"name":"The Rain","cover":"c","plot":"p","cast":"","director":"","genre":"g","releaseDate":"2018-05-04","rating":"7","backdrop_path":[]},
            "episodes":{"1":[
                {"id":"10","episode_num":1,"title":"E1","container_extension":"mkv","season":1,"info":{"rating":"7.5","duration":"45:00"}},
                {"id":"11","episode_num":2,"title":"E2","container_extension":"mkv","season":1,"info":{"rating":8}},
                {"id":"12","episode_num":3,"title":"E3","container_extension":"avi","season":1,"info":{"rating":6.2,"plot":null}},
                {"id":"13","episode_num":4,"title":"E4","container_extension":"mp4","season":1,"info":[]}
            ]}
        }"#;
        let info: SeriesInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.seasons[0].id, "1234");
        assert_eq!(info.seasons[0].season_number, "1");
        let eps = &info.episodes["1"];
        assert_eq!(eps.len(), 4);
        assert_eq!(eps[0].info.rating, Some(7.5));
        assert_eq!(eps[1].info.rating, Some(8.0));
        assert_eq!(eps[2].info.rating, Some(6.2));
        assert_eq!(eps[3].info.rating, None);
        assert_eq!(eps[2].container_extension, "avi");
    }

    #[test]
    fn series_info_with_empty_info_object_as_list() {
        let json = r#"{"seasons":[],"info":[],"episodes":[]}"#;
        let info: SeriesInfo = serde_json::from_str(json).unwrap();
        assert!(info.episodes.is_empty());
        assert_eq!(info.info.name, "");
    }

    /// `rating` de la fiche film : nombre (9/150 fiches observées) ou chaîne.
    #[test]
    fn vod_info_rating_as_number_or_string() {
        let as_number = r#"{"info":{"name":"A","rating":5.5,"backdrop_path":["b"]},"movie_data":{"stream_id":1,"name":"A","container_extension":"mkv"}}"#;
        let v: VodInfo = serde_json::from_str(as_number).unwrap();
        assert_eq!(v.info.rating.as_deref(), Some("5.5"));
        assert_eq!(v.movie_data.container_extension.as_deref(), Some("mkv"));

        let as_int = r#"{"info":{"rating":5},"movie_data":{}}"#;
        assert_eq!(serde_json::from_str::<VodInfo>(as_int).unwrap().info.rating.as_deref(), Some("5"));

        let as_string = r#"{"info":{"rating":"6.1","backdrop_path":"https://x/y.jpg"},"movie_data":{}}"#;
        let v: VodInfo = serde_json::from_str(as_string).unwrap();
        assert_eq!(v.info.rating.as_deref(), Some("6.1"));
        assert_eq!(v.info.backdrop_path.unwrap(), vec!["https://x/y.jpg".to_string()]);
    }

    /// Forme réelle d'Atlas Pro : les deux variantes de chaque champ sont présentes en même temps.
    /// Régression : un `alias` serde donnait « duplicate field » → fiche vide ou erreur.
    #[test]
    fn vod_info_with_both_field_variants_keeps_content() {
        let json = r#"{"info":{"kinopoisk_url":"x","tmdb_id":"1","name":"El correo (2024)","o_name":"El correo",
            "cover_big":"https://img/big.jpg","movie_image":"https://img/poster.jpg","releasedate":"2024-01-19",
            "episode_run_time":"101","youtube_trailer":"","director":"D. Calparsoro","actors":"A, B","cast":"A, B",
            "description":"Un facteur.","plot":"Un facteur.","genre":"Thriller","rating":6.4,"duration":"01:41:00",
            "backdrop_path":["https://img/bd.jpg"]},
            "movie_data":{"stream_id":1,"name":"El correo","container_extension":"mkv"}}"#;
        let v: VodInfo = serde_json::from_str(json).unwrap();
        assert_eq!(v.info.name.as_deref(), Some("El correo (2024)"));
        assert_eq!(v.info.plot.as_deref(), Some("Un facteur."));
        assert_eq!(v.info.cast.as_deref(), Some("A, B"));
        assert_eq!(v.info.movie_image.as_deref(), Some("https://img/poster.jpg"));
        assert_eq!(v.info.release_date.as_deref(), Some("2024-01-19"));
        assert_eq!(v.info.rating.as_deref(), Some("6.4"));
        assert_eq!(v.info.youtube_trailer, None);
        assert_eq!(v.info.backdrop_path.as_deref(), Some(&["https://img/bd.jpg".to_string()][..]));
    }

    /// Seule une variante présente (ou l'autre vide) : on prend celle qui a du contenu.
    #[test]
    fn vod_info_falls_back_to_alternate_field_when_first_is_empty() {
        let json = r#"{"info":{"plot":"","description":"Texte","cover_big":"c","movie_image":""},"movie_data":{}}"#;
        let v: VodInfo = serde_json::from_str(json).unwrap();
        assert_eq!(v.info.plot.as_deref(), Some("Texte"));
        assert_eq!(v.info.movie_image.as_deref(), Some("c"));
    }

    /// Fiche film absente : `info` vide en liste `[]`
    #[test]
    fn vod_info_with_empty_info_list() {
        let v: VodInfo = serde_json::from_str(r#"{"info":[],"movie_data":{"stream_id":2}}"#).unwrap();
        assert!(v.info.name.is_none());
        assert_eq!(v.movie_data.stream_id, Some(2));
    }

    /// Entrées du catalogue : `rating` int/float, `tmdb_id` null/chaîne/entier, `tv_archive_duration` int/chaîne
    #[test]
    fn catalog_entries_with_mixed_types() {
        let json = r#"[
            {"num":1,"name":"Film A","stream_type":"movie","stream_id":131837,"stream_icon":"i","rating":5,"rating_5based":2.5,"added":"1","category_id":"148","container_extension":"mkv","custom_sid":"","direct_source":"","tmdb_id":"1159477"},
            {"num":2,"name":"Film B","stream_type":"movie","stream_id":2,"stream_icon":"i","rating":7.4,"rating_5based":3.7,"added":"1","category_id":"148","container_extension":"avi","custom_sid":"","direct_source":"","tmdb_id":null}
        ]"#;
        let streams: Vec<XtreamStream> = serde_json::from_str(json).unwrap();
        let channels = convert_streams_to_channels(&creds(), streams, "vod", &HashMap::new());
        assert_eq!(channels[0].url, "http://example.com/movie/u/p/131837.mkv");
        assert_eq!(channels[0].rating, Some(5.0));
        assert_eq!(channels[1].url, "http://example.com/movie/u/p/2.avi");
        assert_eq!(channels[1].rating, Some(7.4));

        // Chaîne live : epg_channel_id null, tv_archive_duration en chaîne ou en entier
        let live = r#"[
            {"num":3,"name":"Chaîne","stream_type":"live","stream_id":7819,"stream_icon":"i","epg_channel_id":null,"added":"1","category_id":"104","custom_sid":"","tv_archive":1,"direct_source":"","tv_archive_duration":"10"},
            {"num":4,"name":"Chaîne 2","stream_type":"live","stream_id":7820,"stream_icon":"i","epg_channel_id":"TF1.fr","added":"1","category_id":"104","custom_sid":"","tv_archive":0,"direct_source":"","tv_archive_duration":0}
        ]"#;
        let streams: Vec<XtreamStream> = serde_json::from_str(live).unwrap();
        let channels = convert_streams_to_channels(&creds(), streams, "live", &HashMap::new());
        assert_eq!(channels[0].content_type, "live");
        assert_eq!(channels[0].url, "http://example.com/live/u/p/7819.m3u8");
        assert_eq!(channels.len(), 2);
    }

    #[test]
    fn series_entries_use_cover_and_string_rating() {
        let json = r#"[{"num":1,"name":"S","series_id":2,"cover":"c","plot":"","rating":"7.1","rating_5based":3.5,"backdrop_path":[],"category_id":"9"}]"#;
        let streams: Vec<XtreamStream> = serde_json::from_str(json).unwrap();
        let channels = convert_streams_to_channels(&creds(), streams, "series", &HashMap::new());
        assert_eq!(channels[0].content_type, "series");
        assert_eq!(channels[0].rating, Some(7.1));
        assert_eq!(channels[0].logo.as_deref(), Some("c"));
    }

    #[test]
    fn vod_url_falls_back_to_mp4_without_valid_extension() {
        let c = creds();
        assert_eq!(build_stream_url(&c, 5, "vod", None), "http://example.com/movie/u/p/5.mp4");
        assert_eq!(build_stream_url(&c, 5, "vod", Some("")), "http://example.com/movie/u/p/5.mp4");
        assert_eq!(build_stream_url(&c, 5, "vod", Some(".mkv")), "http://example.com/movie/u/p/5.mkv");
        // Valeur douteuse (injection de chemin) : ignorée
        assert_eq!(build_stream_url(&c, 5, "vod", Some("mp4/../x")), "http://example.com/movie/u/p/5.mp4");
        // L'extension n'affecte pas le live
        assert_eq!(build_stream_url(&c, 5, "live", Some("mkv")), "http://example.com/live/u/p/5.m3u8");
    }
}
