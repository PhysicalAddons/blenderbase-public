use function_name::named;
use tauri::AppHandle;

use crate::{
    core::{
        format_command_error, BlenderDownloadServiceImpl, BlenderVersionBuildTypeDTO,
        DownloadableBlenderVersion, TBlenderDownloadService, COLON_SEPERATOR,
    },
    database::DownloadStatusType,
    AppState,
};

#[named]
#[tauri::command]
pub async fn cmd_get_downloadable_blender_version_data(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    build: &str,
    order: &str,
) -> Result<Vec<DownloadableBlenderVersion>, String> {
    match BlenderDownloadServiceImpl
        .get_downloadable_blender_version_data(app, state, build, order)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_fetch_blender_version_build_types(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: Option<i64>,
    limit: Option<i64>,
    code: Option<String>,
) -> Result<Vec<BlenderVersionBuildTypeDTO>, String> {
    match BlenderDownloadServiceImpl
        .fetch_blender_version_build_types(app, state, id, limit, code)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_update_download_blender_build_type(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    code: String,
) -> Result<(), String> {
    match BlenderDownloadServiceImpl
        .update_download_blender_build_type(app, state, code)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_fetch_download_status_type(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: Option<i64>,
    limit: Option<i64>,
    code: Option<Vec<String>>,
) -> Result<Vec<DownloadStatusType>, String> {
    match BlenderDownloadServiceImpl
        .fetch_download_status_type(app, state, id, limit, code)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
