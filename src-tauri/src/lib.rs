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
            get_playlists,
            delete_playlist,
            // Channel commands
            get_channels,
            search_channels,
            toggle_favorite,
            get_favorites,
            // Settings commands
            get_setting,
            set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
