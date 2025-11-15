use crate::db::{models::*, operations::*};
use crate::mpv::player::MpvPlayer;
use crate::playlist::parse_m3u;
use crate::state::AppState;
use tauri::State;

// ========== MPV Commands ==========

#[tauri::command]
pub async fn check_mpv_installed() -> Result<bool, String> {
    Ok(MpvPlayer::check_installed())
}

#[tauri::command]
pub async fn play_channel(
    state: State<'_, AppState>,
    channel: Channel,
) -> Result<(), String> {
    // Set current channel
    {
        let mut current = state.current_channel.write().await;
        *current = Some(channel.clone());
    }

    // Play the stream
    let mut player = state.mpv_player.lock().await;
    player
        .play(&channel.url)
        .map_err(|e| format!("Failed to play channel: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn stop_playback(state: State<'_, AppState>) -> Result<(), String> {
    let mut player = state.mpv_player.lock().await;
    player
        .stop()
        .map_err(|e| format!("Failed to stop playback: {}", e))?;

    // Clear current channel
    let mut current = state.current_channel.write().await;
    *current = None;

    Ok(())
}

#[tauri::command]
pub async fn is_playing(state: State<'_, AppState>) -> Result<bool, String> {
    let mut player = state.mpv_player.lock().await;
    Ok(player.is_playing())
}

// ========== Playlist Commands ==========

#[tauri::command]
pub async fn import_playlist(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> Result<Playlist, String> {
    // Parse M3U file
    let channels = parse_m3u(&source)
        .await
        .map_err(|e| format!("Failed to parse M3U: {}", e))?;

    let db = state.db.lock().await;

    // Determine if it's a URL or file path
    let (url, file_path) = if source.starts_with("http://") || source.starts_with("https://") {
        (Some(source.clone()), None)
    } else {
        (None, Some(source.clone()))
    };

    // Create playlist
    let playlist = Playlist {
        id: None,
        name,
        url,
        file_path,
        last_updated: None,
        auto_refresh: false,
        created_at: None,
    };

    let playlist_id = create_playlist(&db, &playlist)
        .map_err(|e| format!("Failed to create playlist: {}", e))?;

    // Insert channels
    for mut channel in channels {
        channel.playlist_id = playlist_id;
        create_channel(&db, &channel)
            .map_err(|e| format!("Failed to create channel: {}", e))?;
    }

    // Return playlist with ID
    let mut result = playlist;
    result.id = Some(playlist_id);
    Ok(result)
}

#[tauri::command]
pub async fn get_playlists(state: State<'_, AppState>) -> Result<Vec<Playlist>, String> {
    let db = state.db.lock().await;
    crate::db::operations::get_playlists(&db).map_err(|e| format!("Database error: {}", e))
}

#[tauri::command]
pub async fn delete_playlist(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().await;
    crate::db::operations::delete_playlist(&db, id).map_err(|e| format!("Database error: {}", e))
}

// ========== Channel Commands ==========

#[tauri::command]
pub async fn get_channels(
    state: State<'_, AppState>,
    playlist_id: Option<i64>,
) -> Result<Vec<Channel>, String> {
    let db = state.db.lock().await;
    crate::db::operations::get_channels(&db, playlist_id).map_err(|e| format!("Database error: {}", e))
}

#[tauri::command]
pub async fn search_channels(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<Channel>, String> {
    let db = state.db.lock().await;
    crate::db::operations::search_channels(&db, &query).map_err(|e| format!("Database error: {}", e))
}

#[tauri::command]
pub async fn toggle_favorite(state: State<'_, AppState>, channel_id: i64) -> Result<(), String> {
    let db = state.db.lock().await;
    crate::db::operations::toggle_favorite(&db, channel_id).map_err(|e| format!("Database error: {}", e))
}

#[tauri::command]
pub async fn get_favorites(state: State<'_, AppState>) -> Result<Vec<Channel>, String> {
    let db = state.db.lock().await;
    crate::db::operations::get_favorites(&db).map_err(|e| format!("Database error: {}", e))
}

// ========== Settings Commands ==========

#[tauri::command]
pub async fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    let db = state.db.lock().await;
    crate::db::operations::get_setting(&db, &key).map_err(|e| format!("Database error: {}", e))
}

#[tauri::command]
pub async fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let db = state.db.lock().await;
    crate::db::operations::set_setting(&db, &key, &value).map_err(|e| format!("Database error: {}", e))
}
