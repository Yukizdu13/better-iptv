use crate::db::{models::*, operations::*};
use crate::error::AppError;
use crate::mpv::player::MpvPlayer;
use crate::playlist::{
    fetch_series_info, fetch_xtream_channels_with_progress, parse_m3u, SeriesInfo,
    XtreamCredentials,
};
use crate::state::AppState;
use log::{debug, error, info};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

// Episode data for playlist playback
#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistEpisode {
    pub id: String,
    pub title: String,
    pub extension: String,
}

// ========== Helper Functions ==========

/// Get language settings from database (audio and subtitle languages)
/// This is used by multiple commands to avoid code duplication
fn get_language_settings(db: &Connection) -> Result<(Option<String>, Option<String>), AppError> {
    let settings =
        crate::db::operations::get_multiple_settings(db, &["audio_language", "subtitle_language"])?;

    let audio = settings
        .get("audio_language")
        .filter(|s| !s.is_empty())
        .cloned();
    let subtitle = settings
        .get("subtitle_language")
        .filter(|s| !s.is_empty())
        .cloned();

    Ok((audio, subtitle))
}

// ========== MPV Commands ==========

#[tauri::command]
pub async fn check_mpv_installed() -> Result<bool, AppError> {
    Ok(MpvPlayer::check_installed())
}

#[tauri::command]
pub async fn play_channel(state: State<'_, AppState>, channel: Channel) -> Result<(), AppError> {
    // Set current channel using lightweight struct (avoids full clone)
    {
        let mut current = state.current_channel.write().await;
        *current = Some(crate::state::CurrentChannel::from_channel(&channel));
    }

    // Retrieve language settings from database
    let (audio_lang, subtitle_lang) = {
        let db = state.db.lock().await;
        get_language_settings(&db)?
    };

    // Play the stream with title and language preferences
    let mut player = state.mpv_player.lock().await;
    player
        .play_with_title(
            &channel.url,
            Some(&channel.name),
            audio_lang.as_deref(),
            subtitle_lang.as_deref(),
        )
        .map_err(|e| AppError::Mpv(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn stop_playback(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut player = state.mpv_player.lock().await;
    player.stop().map_err(|e| AppError::Mpv(e.to_string()))?;

    // Clear current channel
    let mut current = state.current_channel.write().await;
    *current = None;

    Ok(())
}

#[tauri::command]
pub async fn is_playing(state: State<'_, AppState>) -> Result<bool, AppError> {
    let mut player = state.mpv_player.lock().await;
    Ok(player.is_playing())
}

// ========== Playlist Commands ==========

#[tauri::command]
pub async fn import_playlist(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> Result<Playlist, AppError> {
    // Validate input
    if name.trim().is_empty() {
        return Err(AppError::InvalidInput("Playlist name cannot be empty".to_string()));
    }
    if source.trim().is_empty() {
        return Err(AppError::InvalidInput("Playlist source cannot be empty".to_string()));
    }

    // Parse M3U file
    let channels = parse_m3u(&source)
        .await
        .map_err(|e| AppError::Parse(e.to_string()))?;

    let db = state.db.lock().await;

    // Determine if it's a URL or file path
    let (url, file_path) = if source.starts_with("http://") || source.starts_with("https://") {
        (Some(source.clone()), None)
    } else {
        (None, Some(source.clone()))
    };

    // Create playlist (no Xtream credentials for M3U)
    let playlist = Playlist {
        id: None,
        name,
        url,
        file_path,
        last_updated: None,
        auto_refresh: false,
        xtream_username: None,
        xtream_password: None,
        created_at: None,
    };

    let playlist_id = create_playlist(&db, &playlist)?;

    // Set playlist_id for all channels
    let channels_with_playlist: Vec<Channel> = channels
        .into_iter()
        .map(|mut c| {
            c.playlist_id = playlist_id;
            c
        })
        .collect();

    // Insert channels in batches
    const BATCH_SIZE: usize = 1000;
    for chunk in channels_with_playlist.chunks(BATCH_SIZE) {
        create_channels_batch(&db, chunk)?;
    }

    // Return playlist with ID
    let mut result = playlist;
    result.id = Some(playlist_id);
    Ok(result)
}

#[tauri::command]
pub async fn get_playlists(state: State<'_, AppState>) -> Result<Vec<Playlist>, AppError> {
    let db = state.db.lock().await;
    Ok(crate::db::operations::get_playlists(&db)?)
}

#[tauri::command]
pub async fn delete_playlist(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let db = state.db.lock().await;
    Ok(crate::db::operations::delete_playlist(&db, id)?)
}

// ========== Channel Commands ==========

#[tauri::command]
pub async fn get_channels(
    state: State<'_, AppState>,
    playlist_id: Option<i64>,
) -> Result<Vec<Channel>, AppError> {
    let db = state.db.lock().await;
    Ok(crate::db::operations::get_channels(&db, playlist_id)?)
}

#[tauri::command]
pub async fn get_channel_groups(
    state: State<'_, AppState>,
    playlist_id: i64,
    content_type: Option<String>,
) -> Result<Vec<String>, AppError> {
    let db = state.db.lock().await;
    Ok(crate::db::operations::get_channel_groups(
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
    let db = state.db.lock().await;
    Ok(crate::db::operations::search_channels(&db, &query)?)
}

#[tauri::command]
pub async fn toggle_favorite(state: State<'_, AppState>, channel_id: i64) -> Result<(), AppError> {
    let db = state.db.lock().await;
    Ok(crate::db::operations::toggle_favorite(&db, channel_id)?)
}

#[tauri::command]
pub async fn get_favorites(state: State<'_, AppState>) -> Result<Vec<Channel>, AppError> {
    let db = state.db.lock().await;
    Ok(crate::db::operations::get_favorites(&db)?)
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
    // Validate input
    if name.trim().is_empty() {
        return Err(AppError::InvalidInput("Profile name cannot be empty".to_string()));
    }
    if server_url.trim().is_empty() {
        return Err(AppError::InvalidInput("Server URL cannot be empty".to_string()));
    }
    if username.trim().is_empty() {
        return Err(AppError::InvalidInput("Username cannot be empty".to_string()));
    }

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

    // Create playlist with Xtream credentials
    let playlist = Playlist {
        id: None,
        name,
        url: Some(server_url),
        file_path: None,
        last_updated: None,
        auto_refresh: false,
        xtream_username: Some(username),
        xtream_password: Some(password),
        created_at: None,
    };

    let playlist_id = create_playlist(&db, &playlist).map_err(|e| {
        error!("Failed to create playlist: {}", e);
        e
    })?;

    debug!("Created playlist with ID: {}", playlist_id);

    // Set playlist_id for all channels
    let channels_with_playlist: Vec<Channel> = channels
        .into_iter()
        .map(|mut c| {
            c.playlist_id = playlist_id;
            c
        })
        .collect();

    // Insert channels in batches for better performance
    const BATCH_SIZE: usize = 1000;
    let total_channels = channels_with_playlist.len();
    debug!(
        "Inserting {} channels in batches of {}...",
        total_channels, BATCH_SIZE
    );

    for (batch_num, chunk) in channels_with_playlist.chunks(BATCH_SIZE).enumerate() {
        create_channels_batch(&db, chunk).map_err(|e| {
            error!("Failed to insert channel batch: {}", e);
            e
        })?;
        debug!(
            "Inserted batch {}/{}",
            batch_num + 1,
            total_channels.div_ceil(BATCH_SIZE)
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

// ========== Series Commands ==========

#[tauri::command]
pub async fn get_series_info(
    server_url: String,
    username: String,
    password: String,
    series_id: i64,
) -> Result<SeriesInfo, AppError> {
    let creds = XtreamCredentials {
        server_url,
        username,
        password,
    };

    fetch_series_info(&creds, series_id)
        .await
        .map_err(|e| AppError::Http(e.to_string()))
}

#[tauri::command]
pub async fn play_episode_with_season(
    state: State<'_, AppState>,
    server_url: String,
    username: String,
    password: String,
    episodes: Vec<PlaylistEpisode>,
) -> Result<(), AppError> {
    if episodes.is_empty() {
        return Err(AppError::InvalidInput("No episodes provided".to_string()));
    }

    // Build URLs for all episodes
    let urls: Vec<String> = episodes
        .iter()
        .map(|episode| {
            format!(
                "{}/series/{}/{}/{}.{}",
                server_url.trim_end_matches('/'),
                username,
                password,
                episode.id,
                episode.extension
            )
        })
        .collect();

    // Use first episode's title for the window
    let first_title = &episodes[0].title;

    // Set current channel (use first episode info with lightweight struct)
    {
        let mut current = state.current_channel.write().await;
        *current = Some(crate::state::CurrentChannel {
            id: None,
            name: first_title.clone(),
            url: urls[0].clone(),
            content_type: "series".to_string(),
        });
    }

    // Retrieve language settings from database
    let (audio_lang, subtitle_lang) = {
        let db = state.db.lock().await;
        get_language_settings(&db)?
    };

    // Play playlist with language preferences
    let mut player = state.mpv_player.lock().await;
    player
        .play_with_playlist(
            &urls,
            Some(first_title),
            audio_lang.as_deref(),
            subtitle_lang.as_deref(),
        )
        .map_err(|e| AppError::Mpv(e.to_string()))?;

    Ok(())
}

// ========== Settings Commands ==========

#[tauri::command]
pub async fn get_setting(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<String>, AppError> {
    let db = state.db.lock().await;
    Ok(crate::db::operations::get_setting(&db, &key)?)
}

#[tauri::command]
pub async fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    let db = state.db.lock().await;
    Ok(crate::db::operations::set_setting(&db, &key, &value)?)
}

// ========== Profile Management Commands ==========

#[tauri::command]
pub async fn get_active_profile_id(state: State<'_, AppState>) -> Result<Option<i64>, AppError> {
    let db = state.db.lock().await;

    let active_id_str = crate::db::operations::get_setting(&db, "active_profile_id")?;

    // Parse string to i64, log warning if parsing fails
    let active_id = active_id_str.and_then(|s| {
        s.parse::<i64>().ok().or_else(|| {
            log::warn!("Invalid profile ID in database: {}", s);
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

    crate::db::operations::set_setting(&db, "active_profile_id", &profile_id.to_string())?;

    info!("Active profile changed to ID: {}", profile_id);

    Ok(())
}

#[tauri::command]
pub async fn rename_playlist(
    state: State<'_, AppState>,
    playlist_id: i64,
    new_name: String,
) -> Result<(), AppError> {
    // Validate input
    if new_name.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Playlist name cannot be empty".to_string(),
        ));
    }

    let db = state.db.lock().await;

    crate::db::operations::rename_playlist(&db, playlist_id, &new_name)?;

    info!("Playlist ID {} renamed to: {}", playlist_id, new_name);

    Ok(())
}

// ========== EPG Commands ==========

#[tauri::command]
pub async fn fetch_epg_data(state: State<'_, AppState>, epg_url: String) -> Result<usize, AppError> {
    // Validate input
    if epg_url.trim().is_empty() {
        return Err(AppError::InvalidInput("EPG URL cannot be empty".to_string()));
    }

    // Fetch and parse EPG data (async, no database lock)
    let programs = crate::epg::fetch_and_parse_epg(&epg_url)
        .await
        .map_err(|e| AppError::Epg(e.to_string()))?;

    // Store in database (sync, with lock)
    let db = state.db.lock().await;

    // First, update EPG IDs for channels that don't have them
    let updated = crate::db::operations::update_channel_epg_ids(&db)?;

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
    let db = state.db.lock().await;

    let current = crate::epg::get_current_program(&db, &channel_epg_id)?;
    let next = crate::epg::get_next_program(&db, &channel_epg_id)?;

    Ok((current, next))
}
