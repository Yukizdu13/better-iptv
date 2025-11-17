mod commands;
mod db;
mod epg;
mod mpv;
mod playlist;
mod state;

use commands::*;
use db::schema::init_schema;
use rusqlite::Connection;
use state::AppState;
use std::path::PathBuf;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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

            // Update EPG IDs for existing channels (for migration)
            match db::operations::update_channel_epg_ids(&conn) {
                Ok(count) => {
                    if count > 0 {
                        println!("Updated EPG IDs for {} channels", count);
                    }
                }
                Err(e) => eprintln!("Warning: Failed to update EPG IDs: {}", e),
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
            search_channels,
            toggle_favorite,
            get_favorites,
            // Series commands
            get_series_info,
            play_episode_with_season,
            // Settings commands
            get_setting,
            set_setting,
            // EPG commands
            fetch_epg_data,
            get_channel_epg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
