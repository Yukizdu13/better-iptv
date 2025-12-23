use crate::error::AppError;
use crate::epg_domain;
use crate::state::AppState;
use log::debug;
use tauri::State;

// ========== EPG Commands ==========

#[tauri::command]
pub async fn fetch_epg_data(state: State<'_, AppState>, epg_url: String) -> Result<usize, AppError> {
    // Validate and normalize input
    let normalized_url = epg_domain::normalize_epg_url(&epg_url);
    epg_domain::validate_epg_url(&normalized_url)?;

    // Fetch and parse EPG data (async, no database lock)
    let programs = crate::epg::fetch_and_parse_epg(&normalized_url)
        .await
        .map_err(|e| AppError::Epg(e.to_string()))?;

    // Store in database (sync, with lock)
    let db = state.db.lock().await;

    // First, update EPG IDs for channels that don't have them
    let updated = crate::db::mutations::update_channel_epg_ids(&db)?;

    if updated > 0 {
        debug!("Updated EPG IDs for {} channels", updated);
    }

    let count = crate::epg::store_epg_programs(&db, &programs)?;

    Ok(count)
}

#[tauri::command]
pub async fn get_channel_epg(
    state: State<'_, AppState>,
    channel_epg_id: String,
) -> Result<(Option<String>, Option<String>), AppError> {
    // Validate channel EPG ID
    epg_domain::validate_channel_epg_id(&channel_epg_id)?;

    let db = state.db.lock().await;

    let current = crate::epg::get_current_program(&db, &channel_epg_id)?;
    let next = crate::epg::get_next_program(&db, &channel_epg_id)?;

    Ok((current, next))
}
