use crate::channel_domain;
use crate::db::models::Channel;
use crate::db::{queries, mutations};
use crate::error::AppError;
use crate::state::AppState;
use tauri::State;

// ========== Channel Commands ==========

#[tauri::command]
pub async fn get_channels(
    state: State<'_, AppState>,
    playlist_id: Option<i64>,
) -> Result<Vec<Channel>, AppError> {
    // Validate playlist ID if provided
    if let Some(id) = playlist_id {
        channel_domain::validate_playlist_id(id)?;
    }

    let db = state.db.lock().await;
    Ok(queries::get_channels(&db, playlist_id)?)
}

#[tauri::command]
pub async fn get_channel_groups(
    state: State<'_, AppState>,
    playlist_id: i64,
    content_type: Option<String>,
) -> Result<Vec<String>, AppError> {
    // Validate playlist ID
    channel_domain::validate_playlist_id(playlist_id)?;

    // Validate content type if provided
    if let Some(ref ct) = content_type {
        channel_domain::validate_content_type(ct)?;
    }

    let db = state.db.lock().await;
    Ok(queries::get_channel_groups(
        &db,
        playlist_id,
        content_type.as_deref(),
    )?)
}

#[tauri::command]
pub async fn search_channels(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<Channel>, AppError> {
    // Validate search query
    channel_domain::validate_search_query(&query)?;

    let db = state.db.lock().await;
    Ok(queries::search_channels(&db, &query)?)
}

#[tauri::command]
pub async fn toggle_favorite(state: State<'_, AppState>, channel_id: i64) -> Result<(), AppError> {
    // Validate channel ID
    channel_domain::validate_channel_id(channel_id)?;

    let db = state.db.lock().await;
    Ok(mutations::toggle_favorite(&db, channel_id)?)
}

#[tauri::command]
pub async fn get_favorites(state: State<'_, AppState>) -> Result<Vec<Channel>, AppError> {
    let db = state.db.lock().await;
    Ok(queries::get_favorites(&db)?)
}
