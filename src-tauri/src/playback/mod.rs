/// MPV is only used on desktop platforms. iOS uses HTML5 <video> in WKWebView.
#[cfg(not(target_os = "ios"))]
pub mod mpv;

#[cfg(not(target_os = "ios"))]
use crate::db::models::Channel;
#[cfg(not(target_os = "ios"))]
use crate::error::AppError;
#[cfg(not(target_os = "ios"))]
use crate::state::CurrentChannel;
#[cfg(not(target_os = "ios"))]
use tokio::sync::RwLock;

#[cfg(not(target_os = "ios"))]
pub async fn play_channel(
    mpv: &mut mpv::MpvPlayer,
    current: &RwLock<Option<CurrentChannel>>,
    channel: &Channel,
    audio_lang: Option<&str>,
    subtitle_lang: Option<&str>,
) -> Result<(), AppError> {
    mpv.play_with_title(&channel.url, Some(&channel.name), audio_lang, subtitle_lang)
        .map_err(|e| AppError::Mpv(e.to_string()))?;
    let mut curr = current.write().await;
    *curr = Some(CurrentChannel::from_channel(channel));
    Ok(())
}

#[cfg(not(target_os = "ios"))]
pub async fn stop(
    mpv: &mut mpv::MpvPlayer,
    current: &RwLock<Option<CurrentChannel>>,
) -> Result<(), AppError> {
    mpv.stop().map_err(|e| AppError::Mpv(e.to_string()))?;
    let mut curr = current.write().await;
    *curr = None;
    Ok(())
}

#[cfg(not(target_os = "ios"))]
pub fn is_playing(mpv: &mut mpv::MpvPlayer) -> bool {
    mpv.is_playing()
}

/// Run a blocking IPC call against the current MPV session without holding the player lock
/// (the call may wait several seconds for MPV to finish loading the stream).
#[cfg(not(target_os = "ios"))]
async fn with_ipc<T, F>(player: &tokio::sync::Mutex<mpv::MpvPlayer>, f: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce(&std::path::Path) -> anyhow::Result<T> + Send + 'static,
{
    let path = player
        .lock()
        .await
        .ipc_path()
        .ok_or_else(|| AppError::Mpv("No active MPV session".to_string()))?;
    tokio::task::spawn_blocking(move || f(&path))
        .await
        .map_err(|e| AppError::Mpv(e.to_string()))?
        .map_err(|e| AppError::Mpv(format!("{:#}", e)))
}

#[cfg(not(target_os = "ios"))]
pub async fn seek(player: &tokio::sync::Mutex<mpv::MpvPlayer>, seconds: f64) -> Result<(), AppError> {
    with_ipc(player, move |path| mpv::seek_relative(path, seconds)).await
}

#[cfg(not(target_os = "ios"))]
pub async fn show_text(
    player: &tokio::sync::Mutex<mpv::MpvPlayer>,
    text: String,
    duration_ms: u64,
) -> Result<(), AppError> {
    with_ipc(player, move |path| mpv::show_text(path, &text, duration_ms)).await
}

#[cfg(not(target_os = "ios"))]
pub async fn toggle_pause(player: &tokio::sync::Mutex<mpv::MpvPlayer>) -> Result<(), AppError> {
    with_ipc(player, mpv::toggle_pause).await
}

#[cfg(not(target_os = "ios"))]
pub async fn playlist_next(player: &tokio::sync::Mutex<mpv::MpvPlayer>) -> Result<(), AppError> {
    with_ipc(player, mpv::playlist_next).await
}

#[cfg(not(target_os = "ios"))]
pub async fn playback_status(
    player: &tokio::sync::Mutex<mpv::MpvPlayer>,
) -> Result<mpv::PlaybackStatus, AppError> {
    with_ipc(player, mpv::playback_status).await
}

#[cfg(not(target_os = "ios"))]
pub fn check_mpv_installed() -> bool {
    mpv::MpvPlayer::check_installed()
}
