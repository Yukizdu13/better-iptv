use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::{Deserialize, Serialize};

/// Lightweight struct for tracking current channel (avoids cloning full Channel)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentChannel {
    pub id: Option<i64>,
    pub name: String,
    pub url: String,
    pub content_type: String,
}

impl CurrentChannel {
    pub fn from_channel(c: &crate::db::models::Channel) -> Self {
        Self {
            id: c.id,
            name: c.name.clone(),
            url: c.url.clone(),
            content_type: c.content_type.clone(),
        }
    }
}

/// Global application state shared across all Tauri commands
#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<SqliteConnectionManager>,
    pub current_channel: Arc<RwLock<Option<CurrentChannel>>>,

    /// MPV player — desktop only (iOS plays via HTML5 <video> in WebView)
    #[cfg(not(target_os = "ios"))]
    pub mpv_player: Arc<Mutex<crate::playback::mpv::MpvPlayer>>,
}

impl AppState {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self {
            pool,
            current_channel: Arc::new(RwLock::new(None)),
            #[cfg(not(target_os = "ios"))]
            mpv_player: Arc::new(Mutex::new(crate::playback::mpv::MpvPlayer::new())),
        }
    }
}
