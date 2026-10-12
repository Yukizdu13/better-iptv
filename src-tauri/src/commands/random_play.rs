use crate::db::models::RandomPlayPreset;
use crate::db::{mutations, queries};
use crate::error::AppError;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_random_play_presets(
    state: State<'_, AppState>,
    playlist_id: i64,
) -> Result<Vec<RandomPlayPreset>, AppError> {
    let conn = state.pool.get()?;
    Ok(queries::get_random_play_presets(&conn, playlist_id)?)
}

#[tauri::command]
pub async fn create_random_play_preset(
    state: State<'_, AppState>,
    playlist_id: i64,
    name: String,
    series_channel_ids: Vec<i64>,
) -> Result<i64, AppError> {
    let conn = state.pool.get()?;
    Ok(mutations::create_random_play_preset(
        &conn,
        playlist_id,
        &name,
        &series_channel_ids,
    )?)
}

#[tauri::command]
pub async fn update_random_play_preset(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    series_channel_ids: Vec<i64>,
) -> Result<(), AppError> {
    let conn = state.pool.get()?;
    mutations::update_random_play_preset(&conn, id, &name, &series_channel_ids)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_random_play_preset(
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), AppError> {
    let conn = state.pool.get()?;
    mutations::delete_random_play_preset(&conn, id)?;
    Ok(())
}
