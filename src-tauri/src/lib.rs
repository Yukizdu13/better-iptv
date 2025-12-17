mod commands;
mod db;
mod epg;
pub mod error;
mod http;
mod mpv;
mod playlist;
mod state;
mod utils;

pub use error::{AppError, AppResult};

use commands::*;
use db::schema::{init_schema, ensure_active_profile};
use log::{info, warn};
use rusqlite::Connection;
use state::AppState;
use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind, RotationStrategy, TimezoneStrategy};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("better-ip-tv".to_string()),
                    }),
                ])
                .max_file_size(10_000_000) // 10 MB
                .rotation_strategy(RotationStrategy::KeepOne)
                .timezone_strategy(TimezoneStrategy::UseLocal)
                .build(),
        )
        .setup(|app| {
            // Get app data directory
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            // Create directory if it doesn't exist
            std::fs::create_dir_all(&app_data_dir)
                .expect("Failed to create app data directory");

            // Database path
            let db_path = app_data_dir.join("better-ip-tv.db");

            // Initialize database
            let conn = Connection::open(&db_path)
                .expect("Failed to open database");

            init_schema(&conn)
                .expect("Failed to initialize database schema");

            // Run migration for active profile setting
            ensure_active_profile(&conn)
                .expect("Failed to ensure active profile setting");

            // Update EPG IDs for existing channels (for migration)
            match db::operations::update_channel_epg_ids(&conn) {
                Ok(count) => {
                    if count > 0 {
                        info!("Updated EPG IDs for {} channels", count);
                    }
                }
                Err(e) => warn!("Failed to update EPG IDs: {}", e),
            }

            // Create and manage app state
            let state = AppState::new(conn);
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // MPV commands
            check_mpv_installed,
            play_channel,
            stop_playback,
            is_playing,
            // Playlist commands
            import_playlist,
            import_xtream_playlist,
            get_playlists,
            delete_playlist,
            // Channel commands
            get_channels,
            get_channel_groups,
            search_channels,
            toggle_favorite,
            get_favorites,
            // Series commands
            get_series_info,
            play_episode_with_season,
            // Settings commands
            get_setting,
            set_setting,
            // Profile management commands
            get_active_profile_id,
            set_active_profile_id,
            rename_playlist,
            // EPG commands
            fetch_epg_data,
            get_channel_epg,
            // Parental controls commands
            set_parental_pin,
            verify_parental_pin,
            reset_parental_pin,
            get_blocked_channels,
            set_blocked_channels,
            get_parental_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
