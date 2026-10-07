use function_name::named;
use tauri::AppHandle;

use crate::{
    core::{
        format_command_error, AchievementStatus, ActivityImportReport, ActivityServiceImpl,
        ActivitySummary, TActivityService, COLON_SEPERATOR,
    },
    database::BlenderVersionTime,
    AppState,
};

#[named]
#[tauri::command]
pub async fn cmd_import_activity(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<ActivityImportReport, String> {
    match ActivityServiceImpl.import_activity(app, state).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_fetch_blender_version_time(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    since: Option<String>,
) -> Result<Vec<BlenderVersionTime>, String> {
    match ActivityServiceImpl.fetch_blender_version_time(app, state, since).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_fetch_activity_summary(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    since: Option<String>,
    today_since: Option<String>,
    week_since: Option<String>,
) -> Result<ActivitySummary, String> {
    match ActivityServiceImpl
        .fetch_activity_summary(app, state, since, today_since, week_since)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_fetch_achievements(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AchievementStatus>, String> {
    match ActivityServiceImpl.fetch_achievements(app, state).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_mark_achievements_seen(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    match ActivityServiceImpl.mark_achievements_seen(app, state).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
