use crate::db::{models::*, queries, mutations};
use crate::error::AppError;
use crate::playlist::{fetch_xtream_channels_with_progress, parse_m3u, get_xtream_epg_url, XtreamCredentials};
use crate::playlist_domain;
use crate::state::AppState;
use log::{debug, error, info};
use tauri::{AppHandle, Emitter, State};

// ========== Playlist Commands ==========

#[tauri::command]
pub async fn import_playlist(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> Result<Playlist, AppError> {
    // Validate input using business logic
    playlist_domain::validate_playlist_name(&name)?;
    playlist_domain::validate_playlist_source(&source)?;

    // Parse M3U file
    let channels = parse_m3u(&source)
        .await
        .map_err(|e| AppError::Parse(e.to_string()))?;

    let db = state.db.lock().await;

    // Build playlist using business logic
    let playlist = playlist_domain::build_m3u_playlist(name, source)?;

    let playlist_id = mutations::create_playlist(&db, &playlist)?;

    // Assign playlist ID using business logic
    let channels_with_playlist = playlist_domain::assign_playlist_id_to_channels(channels, playlist_id);

    // Insert channels in batches using business logic batching
    let batches = playlist_domain::batch_channels(channels_with_playlist, playlist_domain::DEFAULT_BATCH_SIZE);
    for batch in batches {
        mutations::create_channels_batch(&db, &batch)?;
    }

    // Return playlist with ID
    let mut result = playlist;
    result.id = Some(playlist_id);
    Ok(result)
}

#[tauri::command]
pub async fn import_xtream_playlist(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    server_url: String,
    username: String,
    password: String,
) -> Result<Playlist, AppError> {
    // Validate input using business logic
    playlist_domain::validate_playlist_name(&name)?;
    playlist_domain::validate_xtream_credentials(&server_url, &username)?;

    info!(
        "Xtream import started: server={}, username={}",
        server_url, username
    );

    // Create credentials
    let creds = XtreamCredentials {
        server_url: server_url.clone(),
        username: username.clone(),
        password: password.clone(),
    };

    // Fetch channels from Xtream API with progress updates
    debug!("Fetching channels from Xtream API: {}", server_url);
    let channels = fetch_xtream_channels_with_progress(&creds, |progress| {
        let _ = app.emit("import-progress", progress);
    })
    .await
    .map_err(|e| {
        let err_msg = format!("Failed to fetch Xtream channels: {}", e);
        error!("{}", err_msg);
        AppError::Http(err_msg)
    })?;

    info!("Fetched {} channels from Xtream API", channels.len());

    let db = state.db.lock().await;

    // Build playlist using business logic
    let playlist = playlist_domain::build_xtream_playlist(
        name,
        server_url,
        username,
        password,
    )?;

    let playlist_id = mutations::create_playlist(&db, &playlist).map_err(|e| {
        error!("Failed to create playlist: {}", e);
        e
    })?;

    debug!("Created playlist with ID: {}", playlist_id);

    // Assign playlist ID using business logic
    let channels_with_playlist = playlist_domain::assign_playlist_id_to_channels(channels, playlist_id);

    // Insert channels in batches using business logic batching
    let total_channels = channels_with_playlist.len();
    debug!(
        "Inserting {} channels in batches of {}...",
        total_channels, playlist_domain::DEFAULT_BATCH_SIZE
    );

    let batches = playlist_domain::batch_channels(channels_with_playlist, playlist_domain::DEFAULT_BATCH_SIZE);
    for (batch_num, batch) in batches.iter().enumerate() {
        mutations::create_channels_batch(&db, batch).map_err(|e| {
            error!("Failed to insert channel batch: {}", e);
            e
        })?;
        debug!(
            "Inserted batch {}/{}",
            batch_num + 1,
            batches.len()
        );
    }

    info!(
        "Xtream import completed: {} channels imported",
        total_channels
    );

    // Auto-populate EPG URL from Xtream provider if not already set
    let existing_epg_url = queries::get_setting(&db, "epg_url")?;
    let has_epg_url = existing_epg_url
        .as_ref()
        .map(|u| !u.trim().is_empty())
        .unwrap_or(false);

    if !has_epg_url {
        let epg_url = get_xtream_epg_url(&creds);
        mutations::set_setting(&db, "epg_url", &epg_url)?;
        info!(
            "Auto-populated EPG URL from Xtream provider: {}",
            crate::utils::mask_credentials(&epg_url)
        );
    } else {
        debug!("EPG URL already configured, skipping auto-population");
    }

    // Return playlist with ID
    let mut result = playlist;
    result.id = Some(playlist_id);
    Ok(result)
}

#[tauri::command]
pub async fn refresh_playlist(
    app: AppHandle,
    state: State<'_, AppState>,
    playlist_id: i64,
) -> Result<MergeResult, AppError> {
    let db = state.db.lock().await;

    // Get the playlist
    let playlist = queries::get_playlist_by_id(&db, playlist_id)?
        .ok_or(AppError::PlaylistNotFound(playlist_id))?;

    let is_xtream = playlist.xtream_username.is_some();

    // Drop db lock before async fetch
    drop(db);

    // Fetch fresh channels
    let fresh_channels = if is_xtream {
        let creds = XtreamCredentials {
            server_url: playlist.url.clone().unwrap_or_default(),
            username: playlist.xtream_username.clone().unwrap_or_default(),
            password: playlist.xtream_password.clone().unwrap_or_default(),
        };

        info!("Refreshing Xtream playlist '{}' (ID {})", playlist.name, playlist_id);
        fetch_xtream_channels_with_progress(&creds, |progress| {
            let _ = app.emit("refresh-progress", progress);
        })
        .await
        .map_err(|e| {
            error!("Failed to fetch Xtream channels for refresh: {}", e);
            AppError::Http(e.to_string())
        })?
    } else {
        let source = playlist.url.as_deref().or(playlist.file_path.as_deref())
            .ok_or_else(|| AppError::InvalidInput("Playlist has no URL or file path".to_string()))?;

        info!("Refreshing M3U playlist '{}' (ID {})", playlist.name, playlist_id);
        parse_m3u(source)
            .await
            .map_err(|e| {
                error!("Failed to parse M3U for refresh: {}", e);
                AppError::Parse(e.to_string())
            })?
    };

    // Re-acquire db lock for merge
    let db = state.db.lock().await;

    let result = mutations::merge_channels(&db, playlist_id, &fresh_channels, is_xtream)?;

    // Update last_updated timestamp
    mutations::update_playlist_last_updated(&db, playlist_id)?;

    info!(
        "Playlist '{}' refreshed: {} added, {} updated, {} removed ({} total)",
        playlist.name, result.added, result.updated, result.removed, result.total
    );

    Ok(result)
}

#[tauri::command]
pub async fn get_stale_playlist_ids(
    state: State<'_, AppState>,
) -> Result<Vec<i64>, AppError> {
    let db = state.db.lock().await;
    let stale = queries::get_stale_playlists(&db, 7)?;
    Ok(stale.iter().filter_map(|p| p.id).collect())
}

#[tauri::command]
pub async fn get_playlists(state: State<'_, AppState>) -> Result<Vec<Playlist>, AppError> {
    let db = state.db.lock().await;
    Ok(queries::get_playlists(&db)?)
}

#[tauri::command]
pub async fn delete_playlist(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let db = state.db.lock().await;
    Ok(mutations::delete_playlist(&db, id)?)
}

#[tauri::command]
pub async fn rename_playlist(
    state: State<'_, AppState>,
    playlist_id: i64,
    new_name: String,
) -> Result<(), AppError> {
    // Validate input using business logic
    playlist_domain::validate_playlist_name(&new_name)?;

    let db = state.db.lock().await;

    mutations::rename_playlist(&db, playlist_id, &new_name)?;

    info!("Playlist ID {} renamed to: {}", playlist_id, new_name);

    Ok(())
}
