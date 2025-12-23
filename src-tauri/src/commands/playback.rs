use crate::db::{queries, models::Channel};
use crate::error::AppError;
use crate::playback;
use crate::state::AppState;
use tauri::State;

// ========== MPV Commands ==========

#[tauri::command]
pub async fn check_mpv_installed() -> Result<bool, AppError> {
    Ok(playback::check_mpv_installed())
}

#[tauri::command]
pub async fn play_channel(state: State<'_, AppState>, channel: Channel) -> Result<(), AppError> {
    // Get language settings from database
    let (audio_lang, subtitle_lang) = {
        let db = state.db.lock().await;
        let settings = queries::get_multiple_settings(&db, &["audio_language", "subtitle_language"])?;

        let audio = settings
            .get("audio_language")
            .filter(|s| !s.is_empty())
            .cloned();
        let subtitle = settings
            .get("subtitle_language")
            .filter(|s| !s.is_empty())
            .cloned();

        (audio, subtitle)
    };

    // Play using playback domain
    let mut player = state.mpv_player.lock().await;
    playback::play_channel(
        &mut player,
        &state.current_channel,
        &channel,
        audio_lang.as_deref(),
        subtitle_lang.as_deref(),
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn stop_playback(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut player = state.mpv_player.lock().await;
    playback::stop(&mut player, &state.current_channel).await?;

    Ok(())
}

#[tauri::command]
pub async fn is_playing(state: State<'_, AppState>) -> Result<bool, AppError> {
    let mut player = state.mpv_player.lock().await;
    Ok(playback::is_playing(&mut player))
}
