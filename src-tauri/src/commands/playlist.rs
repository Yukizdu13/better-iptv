use crate::db::{models::*, queries, mutations};
use crate::error::AppError;
use crate::playlist::{fetch_xtream_channels_with_progress, parse_m3u, XtreamCredentials};
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

    // Return playlist with ID
    let mut result = playlist;
    result.id = Some(playlist_id);
    Ok(result)
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
