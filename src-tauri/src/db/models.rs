use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: Option<i64>,
    pub name: String,
    pub url: Option<String>,
    pub file_path: Option<String>,
    pub last_updated: Option<String>,
    pub auto_refresh: bool,
    pub xtream_username: Option<String>,
    pub xtream_password: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Option<i64>,
    pub playlist_id: i64,
    pub name: String,
    pub url: String,
    pub logo: Option<String>,
    pub group_name: Option<String>,
    pub epg_id: Option<String>,
    pub tvg_name: Option<String>,
    pub content_type: String, // "live", "vod", "series"
    pub is_favorite: bool,
    pub sort_order: i32,
    pub category_order: i32,
    pub rating: Option<f32>,
    pub created_at: Option<String>,
}

/// Last episode watched for a given series (keyed by the series' channel id)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastEpisode {
    pub channel_id: i64,
    pub season: String,
    pub episode_id: String,
    pub episode_num: i32,
    pub title: String,
    /// Last known playback position (None = only the episode is known, not the position)
    pub position_seconds: Option<f64>,
    pub duration_seconds: Option<f64>,
    pub updated_at: Option<String>,
}

/// A saved selection of series for cross-series random play
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomPlayPreset {
    pub id: Option<i64>,
    pub playlist_id: i64,
    pub name: String,
    pub series_channel_ids: Vec<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Result of a merge-based playlist refresh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub added: usize,
    pub updated: usize,
    pub removed: usize,
    pub total: usize,
}
