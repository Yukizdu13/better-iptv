use crate::error::AppError;
use crate::epg_domain;
use crate::state::AppState;
use log::{debug, info, warn};
use serde::Serialize;
use tauri::State;

/// EPG status information returned to frontend
#[derive(Debug, Serialize)]
pub struct EpgStatus {
    /// Whether an EPG URL is configured
    pub has_url: bool,
    /// Last successful EPG fetch timestamp (RFC3339 format)
    pub last_fetched: Option<String>,
    /// Number of programs currently in database
    pub program_count: usize,
}

/// Result of an EPG refresh operation
#[derive(Debug, Serialize)]
pub struct EpgRefreshResult {
    /// Whether the refresh was successful
    pub success: bool,
    /// Number of programs loaded
    pub programs_loaded: usize,
    /// Timestamp of the refresh (RFC3339 format)
    pub timestamp: String,
    /// Error message if failed
    pub error: Option<String>,
}

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

/// Get EPG status information (URL configured, last fetch time, program count)
#[tauri::command]
pub async fn get_epg_status(state: State<'_, AppState>) -> Result<EpgStatus, AppError> {
    let db = state.db.lock().await;

    // Check if EPG URL is configured
    let epg_url = crate::db::queries::get_setting(&db, "epg_url")?;
    let has_url = epg_url.map(|u| !u.trim().is_empty()).unwrap_or(false);

    // Get last fetched timestamp
    let last_fetched = crate::db::queries::get_setting(&db, "epg_last_fetched")?;

    // Get program count
    let program_count = crate::db::queries::get_epg_program_count(&db)?;

    Ok(EpgStatus {
        has_url,
        last_fetched,
        program_count,
    })
}

/// Force refresh EPG data from configured URL
#[tauri::command]
pub async fn force_refresh_epg(state: State<'_, AppState>) -> Result<EpgRefreshResult, AppError> {
    // Get EPG URL from settings
    let epg_url = {
        let db = state.db.lock().await;
        crate::db::queries::get_setting(&db, "epg_url")?
    };

    // Check if URL is configured
    let epg_url = match epg_url {
        Some(url) if !url.trim().is_empty() => url,
        _ => {
            return Ok(EpgRefreshResult {
                success: false,
                programs_loaded: 0,
                timestamp: chrono::Utc::now().to_rfc3339(),
                error: Some("No EPG URL configured".to_string()),
            });
        }
    };

    info!("Force refreshing EPG from: {}", epg_url);

    // Validate and normalize URL
    let normalized_url = epg_domain::normalize_epg_url(&epg_url);
    if let Err(e) = epg_domain::validate_epg_url(&normalized_url) {
        warn!("Invalid EPG URL: {}", e);
        return Ok(EpgRefreshResult {
            success: false,
            programs_loaded: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            error: Some(format!("Invalid EPG URL: {}", e)),
        });
    }

    // Fetch and parse EPG data (async, no database lock)
    let programs = match crate::epg::fetch_and_parse_epg(&normalized_url).await {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to fetch EPG: {}", e);
            return Ok(EpgRefreshResult {
                success: false,
                programs_loaded: 0,
                timestamp: chrono::Utc::now().to_rfc3339(),
                error: Some(format!("Failed to fetch EPG: {}", e)),
            });
        }
    };

    // Store in database (sync, with lock)
    let db = state.db.lock().await;

    // Update EPG IDs for channels that don't have them
    let updated = crate::db::mutations::update_channel_epg_ids(&db)?;
    if updated > 0 {
        debug!("Updated EPG IDs for {} channels", updated);
    }

    let count = crate::epg::store_epg_programs(&db, &programs)?;

    // Store last fetched timestamp
    let timestamp = chrono::Utc::now().to_rfc3339();
    crate::db::mutations::set_setting(&db, "epg_last_fetched", &timestamp)?;

    info!("EPG refresh completed: {} programs loaded", count);

    Ok(EpgRefreshResult {
        success: true,
        programs_loaded: count,
        timestamp,
        error: None,
    })
}
