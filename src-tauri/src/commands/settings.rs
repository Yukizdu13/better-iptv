use crate::db::{queries, mutations};
use crate::error::AppError;
use crate::state::AppState;
use log::{info, warn};
use tauri::State;

// ========== Settings Commands ==========

#[tauri::command]
pub async fn get_setting(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<String>, AppError> {
    let db = state.db.lock().await;
    Ok(queries::get_setting(&db, &key)?)
}

#[tauri::command]
pub async fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    let db = state.db.lock().await;
    Ok(mutations::set_setting(&db, &key, &value)?)
}

// ========== Profile Management Commands ==========

#[tauri::command]
pub async fn get_active_profile_id(state: State<'_, AppState>) -> Result<Option<i64>, AppError> {
    let db = state.db.lock().await;

    let active_id_str = queries::get_setting(&db, "active_profile_id")?;

    // Parse string to i64, log warning if parsing fails
    let active_id = active_id_str.and_then(|s| {
        s.parse::<i64>().ok().or_else(|| {
            warn!("Invalid profile ID in database: {}", s);
            None
        })
    });

    Ok(active_id)
}

#[tauri::command]
pub async fn set_active_profile_id(
    state: State<'_, AppState>,
    profile_id: i64,
) -> Result<(), AppError> {
    let db = state.db.lock().await;

    mutations::set_setting(&db, "active_profile_id", &profile_id.to_string())?;

    info!("Active profile changed to ID: {}", profile_id);

    Ok(())
}
