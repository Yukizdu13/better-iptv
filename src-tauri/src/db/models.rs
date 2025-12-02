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
    pub category_order: i32, // Order from provider's category list
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpgProgram {
    pub id: Option<i64>,
    pub channel_epg_id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub category: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchHistory {
    pub id: Option<i64>,
    pub channel_id: i64,
    pub watched_at: String,
    pub duration_seconds: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpgSource {
    pub id: Option<i64>,
    pub name: String,
    pub url: String,
    pub last_fetched: Option<String>,
    pub auto_refresh: bool,
    pub refresh_interval_hours: i32,
    pub created_at: Option<String>,
}
