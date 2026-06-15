use crate::error::AppError;
use crate::playlist::{fetch_vod_info, VodInfo, XtreamCredentials};
use crate::series_domain;

#[tauri::command]
pub async fn get_vod_info(
    server_url: String,
    username: String,
    password: String,
    vod_id: i64,
) -> Result<VodInfo, AppError> {
    series_domain::validate_server_url(&server_url)?;
    series_domain::validate_credentials(&username, &password)?;

    let creds = XtreamCredentials {
        server_url,
        username,
        password,
    };

    fetch_vod_info(&creds, vod_id)
        .await
        .map_err(|e| AppError::Http(e.to_string()))
}
