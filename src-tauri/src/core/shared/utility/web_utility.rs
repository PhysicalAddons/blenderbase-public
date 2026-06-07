use chrono::{DateTime, Duration, Utc};
use serde::de::DeserializeOwned;

use crate::{
    core::{AppSettingCodeKind, NETWORKCHECK_KDE_ORG},
    AppState,
};

const HTTP_TIMEOUT: u64 = 3;

pub async fn http_get_as_json<T: DeserializeOwned>(
    state: tauri::State<'_, AppState>,
    url: String,
) -> Result<T, String> {
    // Vec<DownloadableBlenderVersion>
    let response = match state
        .http_client
        .get(url)
        // .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT))
        .send()
        .await
    {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed http get as JSON: {}", e)),
    };
    match response.json::<T>().await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format!("Failed http get as JSON: {}", e)),
    }
}

pub async fn http_get_as_string(
    state: tauri::State<'_, AppState>,
    url: String,
) -> Result<String, String> {
    let response = match state
        .http_client
        .get(url)
        // .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT))
        .send()
        .await
    {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed http get as string: {}", e)),
    };
    if response.status().is_success() {
        match response.text().await {
            Ok(response_string) => Ok(response_string),
            Err(e) => return Err(format!("Failed http get as string: {}", e)),
        }
    } else {
        Err(format!("Failed http get as string: {}", response.status()))
    }
}

pub async fn check_internet_connection(
    state: tauri::State<'_, AppState>,
    is_override: Option<bool>,
) -> Result<Option<bool>, String> {
    let r = state.app_setting_repository();
    let mut app_settings = match r
        .fetch(
            None,
            None,
            Some(AppSettingCodeKind::SetCheckInternetConnectionTimeout.to_string()),
            None,
            None,
        )
        .await
    {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed check internet connection: {}", e)),
    };
    if app_settings.is_empty() {
        return Err(format!(
            "Failed check internet connection no app setting with code: {:?}",
            AppSettingCodeKind::SetCheckInternetConnectionTimeout.to_string()
        ));
    }
    let app_setting = app_settings.remove(0);
    let should_check = {
        let mut timeouts: std::sync::MutexGuard<'_, crate::core::ActionTimestamp> =
            match state.action_timeouts.lock() {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed check internet connection: {}", e)),
            };
        let now = chrono::Utc::now();
        let last_check: DateTime<Utc> = match timeouts.fs_utility_cmd_check_internet_connection {
            Some(v) => v,
            None => now,
        };
        let should = timeouts.fs_utility_cmd_check_internet_connection.is_none() || 
            match is_override { Some(v) => v, None => false, } == true || // User is asking for a check.
            (now - last_check) > Duration::seconds(match app_setting.int_value {
                Some(v) => v,
                None => return Err(format!("Failed check internet connection"))
            }); // 5 seconds have passed since last check.
        if should {
            timeouts.fs_utility_cmd_check_internet_connection = Some(now);
        }
        should
    };
    if should_check {
        let result = state
            .http_client
            .get(NETWORKCHECK_KDE_ORG)
            .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT))
            .send()
            .await;

        let is_connected = match result {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        };
        return Ok(Some(is_connected));
    }
    return Ok(None);
}
