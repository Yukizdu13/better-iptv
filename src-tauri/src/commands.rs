use crate::db::{models::*, operations::*};
use crate::mpv::player::MpvPlayer;
use crate::playlist::{parse_m3u, fetch_xtream_channels_with_progress, fetch_series_info, XtreamCredentials, SeriesInfo, FetchProgress};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{State, AppHandle, Emitter};

// Episode data for playlist playback
#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistEpisode {
    pub id: String,
    pub title: String,
    pub extension: String,
}

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

    // Retrieve language settings from database
    let (audio_lang, subtitle_lang) = {
        let db = state.db.lock().await;
        let audio = crate::db::operations::get_setting(&db, "audio_language")
            .map_err(|e| format!("Failed to get audio language setting: {}", e))?
            .filter(|s| !s.is_empty());
        let subtitle = crate::db::operations::get_setting(&db, "subtitle_language")
            .map_err(|e| format!("Failed to get subtitle language setting: {}", e))?
            .filter(|s| !s.is_empty());
        (audio, subtitle)
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

    let playlist_id = create_playlist(&db, &playlist)
        .map_err(|e| format!("Failed to create playlist: {}", e))?;

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
        create_channels_batch(&db, chunk)
            .map_err(|e| format!("Failed to insert channel batch: {}", e))?;
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

#[tauri::command]
pub async fn import_xtream_playlist(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    server_url: String,
    username: String,
    password: String,
) -> Result<Playlist, String> {
    println!("=== Xtream Import Started ===");
    println!("Server: {}", server_url);
    println!("Username: {}", username);

    // Create credentials
    let creds = XtreamCredentials {
        server_url: server_url.clone(),
        username: username.clone(),
        password: password.clone(),
    };

    // Fetch channels from Xtream API with progress updates
    println!("Fetching channels from Xtream API...");
    let channels = fetch_xtream_channels_with_progress(&creds, |progress| {
        let _ = app.emit("import-progress", progress);
    })
        .await
        .map_err(|e| {
            let err_msg = format!("Failed to fetch Xtream channels: {}", e);
            eprintln!("ERROR: {}", err_msg);
            err_msg
        })?;

    println!("Fetched {} channels", channels.len());

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

    let playlist_id = create_playlist(&db, &playlist)
        .map_err(|e| {
            let err_msg = format!("Failed to create playlist: {}", e);
            eprintln!("ERROR: {}", err_msg);
            err_msg
        })?;

    println!("Created playlist with ID: {}", playlist_id);

    // Set playlist_id for all channels
    let mut channels_with_playlist: Vec<Channel> = channels
        .into_iter()
        .map(|mut c| {
            c.playlist_id = playlist_id;
            c
        })
        .collect();

    // Insert channels in batches for better performance
    const BATCH_SIZE: usize = 1000;
    let total_channels = channels_with_playlist.len();
    println!("Inserting {} channels in batches of {}...", total_channels, BATCH_SIZE);

    for (batch_num, chunk) in channels_with_playlist.chunks(BATCH_SIZE).enumerate() {
        create_channels_batch(&db, chunk)
            .map_err(|e| {
                let err_msg = format!("Failed to insert channel batch: {}", e);
                eprintln!("ERROR: {}", err_msg);
                err_msg
            })?;
        println!("Inserted batch {}/{}", batch_num + 1, (total_channels + BATCH_SIZE - 1) / BATCH_SIZE);
    }

    println!("=== Xtream Import Completed ===");

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
) -> Result<SeriesInfo, String> {
    let creds = XtreamCredentials {
        server_url,
        username,
        password,
    };

    fetch_series_info(&creds, series_id)
        .await
        .map_err(|e| format!("Failed to fetch series info: {}", e))
}

#[tauri::command]
pub async fn play_episode_with_season(
    state: State<'_, AppState>,
    server_url: String,
    username: String,
    password: String,
    episodes: Vec<PlaylistEpisode>,
) -> Result<(), String> {
    if episodes.is_empty() {
        return Err("No episodes provided".to_string());
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

    // Set current channel (use first episode info)
    {
        let mut current = state.current_channel.write().await;
        *current = Some(Channel {
            playlist_id: 0,
            name: first_title.clone(),
            url: urls[0].clone(),
            content_type: "series".to_string(),
            is_favorite: false,
            sort_order: 0,
            id: None,
            logo: None,
            group_name: None,
            epg_id: None,
            tvg_name: None,
            created_at: None,
        });
    }

    // Retrieve language settings from database
    let (audio_lang, subtitle_lang) = {
        let db = state.db.lock().await;
        let audio = crate::db::operations::get_setting(&db, "audio_language")
            .map_err(|e| format!("Failed to get audio language setting: {}", e))?
            .filter(|s| !s.is_empty());
        let subtitle = crate::db::operations::get_setting(&db, "subtitle_language")
            .map_err(|e| format!("Failed to get subtitle language setting: {}", e))?
            .filter(|s| !s.is_empty());
        (audio, subtitle)
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
        .map_err(|e| format!("Failed to play episode playlist: {}", e))?;

    Ok(())
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

// ========== EPG Commands ==========

#[tauri::command]
pub async fn fetch_epg_data(state: State<'_, AppState>, epg_url: String) -> Result<usize, String> {
    // Fetch and parse EPG data (async, no database lock)
    let programs = crate::epg::fetch_and_parse_epg(&epg_url)
        .await
        .map_err(|e| format!("Failed to fetch EPG: {}", e))?;

    // Store in database (sync, with lock)
    let db = state.db.lock().await;

    // First, update EPG IDs for channels that don't have them
    let updated = crate::db::operations::update_channel_epg_ids(&db)
        .map_err(|e| format!("Failed to update channel EPG IDs: {}", e))?;

    if updated > 0 {
        println!("Updated EPG IDs for {} channels", updated);
    }

    let count = crate::epg::store_epg_programs(&db, &programs)
        .map_err(|e| format!("Failed to store EPG: {}", e))?;

    Ok(count)
}

#[tauri::command]
pub async fn get_channel_epg(
    state: State<'_, AppState>,
    channel_epg_id: String,
) -> Result<(Option<String>, Option<String>), String> {
    let db = state.db.lock().await;

    let current = crate::epg::get_current_program(&db, &channel_epg_id)
        .map_err(|e| format!("Failed to get current program: {}", e))?;

    let next = crate::epg::get_next_program(&db, &channel_epg_id)
        .map_err(|e| format!("Failed to get next program: {}", e))?;

    Ok((current, next))
}
