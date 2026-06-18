use crate::error::AppError;

#[tauri::command]
pub async fn get_platform() -> Result<String, AppError> {
    let p = if cfg!(target_os = "ios") {
        "ios"
    } else if cfg!(target_os = "android") {
        "android"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };
    Ok(p.to_string())
}
