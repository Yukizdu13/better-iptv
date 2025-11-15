use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use rusqlite::Connection;

/// Global application state shared across all Tauri commands
#[derive(Clone)]
pub struct AppState {
    /// Database connection
    pub db: Arc<Mutex<Connection>>,

    /// Currently playing channel (if any)
    pub current_channel: Arc<RwLock<Option<crate::db::models::Channel>>>,

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
