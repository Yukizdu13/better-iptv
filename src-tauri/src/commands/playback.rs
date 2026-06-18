use crate::error::AppError;

// ── Desktop (non-iOS) — real MPV commands ────────────────────────────────────

#[cfg(not(target_os = "ios"))]
use crate::db::queries;
#[cfg(not(target_os = "ios"))]
use crate::db::models::Channel;
#[cfg(not(target_os = "ios"))]
use crate::playback;
#[cfg(not(target_os = "ios"))]
use crate::state::AppState;
#[cfg(not(target_os = "ios"))]
use log::info;
#[cfg(not(target_os = "ios"))]
use tauri::State;

#[cfg(not(target_os = "ios"))]
#[tauri::command]
pub async fn check_mpv_installed() -> Result<bool, AppError> {
    Ok(playback::check_mpv_installed())
}

#[cfg(not(target_os = "ios"))]
#[tauri::command]
pub async fn play_channel(state: State<'_, AppState>, channel: Channel) -> Result<(), AppError> {
    let (audio_lang, subtitle_lang) = {
        let conn = state.pool.get()?;
        let settings =
            queries::get_multiple_settings(&conn, &["audio_language", "subtitle_language"])?;
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

    let mut player = state.mpv_player.lock().await;
    playback::play_channel(
        &mut player,
        &state.current_channel,
        &channel,
        audio_lang.as_deref(),
        subtitle_lang.as_deref(),
    )
    .await?;

    info!("Playing channel: {} ({})", channel.name, channel.content_type);
    Ok(())
}

#[cfg(not(target_os = "ios"))]
#[tauri::command]
pub async fn stop_playback(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut player = state.mpv_player.lock().await;
    playback::stop(&mut player, &state.current_channel).await?;
    info!("Playback stopped");
    Ok(())
}

#[cfg(not(target_os = "ios"))]
#[tauri::command]
pub async fn is_playing(state: State<'_, AppState>) -> Result<bool, AppError> {
    let mut player = state.mpv_player.lock().await;
    Ok(playback::is_playing(&mut player))
}

// ── iOS stubs — frontend handles playback via HTML5 <video> ──────────────────

#[cfg(target_os = "ios")]
#[tauri::command]
pub async fn check_mpv_installed() -> Result<bool, AppError> {
    Ok(false)
}

#[cfg(target_os = "ios")]
#[tauri::command]
pub async fn play_channel(_channel: crate::db::models::Channel) -> Result<(), AppError> {
    Ok(())
}

#[cfg(target_os = "ios")]
#[tauri::command]
pub async fn stop_playback() -> Result<(), AppError> {
    Ok(())
}

#[cfg(target_os = "ios")]
#[tauri::command]
pub async fn is_playing() -> Result<bool, AppError> {
    Ok(false)
}
