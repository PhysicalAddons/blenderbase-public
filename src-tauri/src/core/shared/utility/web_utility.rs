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

/// Like `http_get_as_string`, but with a cloned client so it can run inside spawned tasks.
pub async fn http_get_as_string_with_client(
    client: reqwest::Client,
    url: String,
) -> Result<String, String> {
    // Mirrors rate-limit by IP: a burst of directory listings gets "429 Too
    // Many Requests". Those (and 503) are retried with growing pauses, honouring
    // Retry-After when the server sends one; anything else fails at once.
    const ATTEMPTS: u32 = 4;
    let mut attempt = 0u32;
    loop {
        attempt += 1;
        let response = match client.get(&url).send().await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed http get as string: {}", e)),
        };
        let status = response.status();
        if status.is_success() {
            return match response.text().await {
                Ok(v) => Ok(v),
                Err(e) => Err(format!("Failed http get as string: {}", e)),
            };
        }
        let retryable = status == reqwest::StatusCode::TOO_MANY_REQUESTS
            || status == reqwest::StatusCode::SERVICE_UNAVAILABLE;
        if !retryable || attempt >= ATTEMPTS {
            return Err(format!("Failed http get as string: {} returned {}", url, status));
        }
        let retry_after = response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .map(|s| s.min(30));
        let wait = retry_after.unwrap_or(2u64.pow(attempt));
        tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
    }
}
