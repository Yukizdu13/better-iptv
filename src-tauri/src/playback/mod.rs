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

#[cfg(not(target_os = "ios"))]
pub fn check_mpv_installed() -> bool {
    mpv::MpvPlayer::check_installed()
}
