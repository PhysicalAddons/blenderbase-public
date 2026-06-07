use function_name::named;
use tauri::AppHandle;

use crate::{
    core::{format_command_error, BlendFileServiceImpl, TBlendFileService, COLON_SEPERATOR},
    database::{BlendFile, BlenderSeries, BlenderVersion},
    AppState,
};
#[named]
#[tauri::command]
pub async fn cmd_refresh_blend_files(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    match BlendFileServiceImpl.refresh_blend_files(app, state).await {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_fetch_blend_files(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: Option<String>,
    limit: Option<i64>,
    file_path: Option<String>,
    blender_series_id: Option<String>,
    order: &str,
) -> Result<Vec<BlendFile>, String> {
    match BlendFileServiceImpl
        .refresh_blend_files(app.clone(), state.clone())
        .await
    {
        Ok(_) => {}
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
    match BlendFileServiceImpl
        .fetch_blend_files(app, state, id, limit, file_path, blender_series_id, order)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_fetch_blender_series(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: Option<String>,
    limit: Option<i64>,
    blender_config_directory: Option<String>,
    is_mapped: Option<bool>,
    order: &str,
) -> Result<Vec<BlenderSeries>, String> {
    match BlendFileServiceImpl
        .refresh_blend_files(app.clone(), state.clone())
        .await
    {
        Ok(_) => {}
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
    match BlendFileServiceImpl
        .fetch_blender_series(
            app,
            state,
            id,
            limit,
            blender_config_directory,
            is_mapped,
            order,
        )
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_open_blend_file(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    blend_file: BlendFile,
    blender_version: BlenderVersion,
) -> Result<(), String> {
    match BlendFileServiceImpl
        .open_blend_file(app, state, blend_file, blender_version)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_reveal_in_file_explorer(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    file_path: std::path::PathBuf,
) -> Result<(), String> {
    match BlendFileServiceImpl
        .reveal_in_file_explorer(app, state, file_path)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_delete_blend_file(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: Option<String>,
) -> Result<(), String> {
    match BlendFileServiceImpl.delete_blend_file(app, state, id).await {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_update_blender_series(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    blender_series: BlenderSeries,
) -> Result<(), String> {
    match BlendFileServiceImpl
        .update_blender_series(app, state, blender_series)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
