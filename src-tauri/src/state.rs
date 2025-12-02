use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use rusqlite::Connection;
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
    /// Create from a full Channel struct, extracting only necessary fields
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
    /// Database connection
    pub db: Arc<Mutex<Connection>>,

    /// Currently playing channel (if any) - uses lightweight struct
    pub current_channel: Arc<RwLock<Option<CurrentChannel>>>,

    /// MPV player instance
    pub mpv_player: Arc<Mutex<crate::mpv::player::MpvPlayer>>,
}

impl AppState {
    pub fn new(db: Connection) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            current_channel: Arc::new(RwLock::new(None)),
            mpv_player: Arc::new(Mutex::new(crate::mpv::player::MpvPlayer::new())),
        }
    }
}
