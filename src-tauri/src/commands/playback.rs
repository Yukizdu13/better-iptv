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
    let (audio_lang, subtitle_lang, quality) = {
        let conn = state.pool.get()?;
        let settings = queries::get_multiple_settings(
            &conn,
            &["audio_language", "subtitle_language", "video_quality"],
        )?;
        let quality = playback::mpv::VideoQuality::from_setting(
            settings.get("video_quality").map(String::as_str),
        );
        let audio = settings
            .get("audio_language")
            .filter(|s| !s.is_empty())
            .cloned();
        let subtitle = settings
            .get("subtitle_language")
            .filter(|s| !s.is_empty())
            .cloned();
        (audio, subtitle, quality)
    };

    let mut player = state.mpv_player.lock().await;
    player.set_video_quality(quality);
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

#[cfg(not(target_os = "ios"))]
#[tauri::command]
pub async fn mpv_seek(state: State<'_, AppState>, seconds: f64) -> Result<(), AppError> {
    playback::seek(&state.mpv_player, seconds).await
}

#[cfg(not(target_os = "ios"))]
#[tauri::command]
pub async fn mpv_show_text(
    state: State<'_, AppState>,
    text: String,
    duration_ms: u64,
) -> Result<(), AppError> {
    playback::show_text(&state.mpv_player, text, duration_ms).await
}

#[cfg(not(target_os = "ios"))]
#[tauri::command]
pub async fn mpv_toggle_pause(state: State<'_, AppState>) -> Result<(), AppError> {
    playback::toggle_pause(&state.mpv_player).await
}

#[cfg(not(target_os = "ios"))]
#[tauri::command]
pub async fn mpv_playlist_next(state: State<'_, AppState>) -> Result<(), AppError> {
    playback::playlist_next(&state.mpv_player).await
}

#[cfg(not(target_os = "ios"))]
#[tauri::command]
pub async fn mpv_get_playback_status(
    state: State<'_, AppState>,
) -> Result<playback::mpv::PlaybackStatus, AppError> {
    playback::playback_status(&state.mpv_player).await
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

#[cfg(target_os = "ios")]
#[tauri::command]
pub async fn mpv_seek(_seconds: f64) -> Result<(), AppError> {
    Ok(())
}

#[cfg(target_os = "ios")]
#[tauri::command]
pub async fn mpv_show_text(_text: String, _duration_ms: u64) -> Result<(), AppError> {
    Ok(())
}

#[cfg(target_os = "ios")]
#[tauri::command]
pub async fn mpv_toggle_pause() -> Result<(), AppError> {
    Ok(())
}

#[cfg(target_os = "ios")]
#[tauri::command]
pub async fn mpv_playlist_next() -> Result<(), AppError> {
    Ok(())
}

#[cfg(target_os = "ios")]
#[tauri::command]
pub async fn mpv_get_playback_status() -> Result<(), AppError> {
    Err(AppError::Mpv("MPV is not available on iOS".to_string()))
}
