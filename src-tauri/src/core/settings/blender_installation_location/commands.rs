use function_name::named;
use tauri::AppHandle;

use crate::{
    core::{
        format_command_error, BlenderInstallationLocationServiceImpl,
        TBlenderInstallationLocationService, COLON_SEPERATOR,
    },
    database::BlenderInstallationLocation,
    AppState,
};

#[named]
#[tauri::command]
pub async fn cmd_insert_blender_installation_location(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    match BlenderInstallationLocationServiceImpl
        .insert_blender_installation_location(app, state)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_set_blender_installation_location_as_default(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: Option<String>,
    is_default: bool,
) -> Result<(), String> {
    match BlenderInstallationLocationServiceImpl
        .set_blender_installation_location_as_default(app, state, id, is_default)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_fetch_blender_installation_locations(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: Option<String>,
    limit: Option<i64>,
    directory_path: Option<String>,
    is_default: Option<bool>,
) -> Result<Vec<BlenderInstallationLocation>, String> {
    match BlenderInstallationLocationServiceImpl
        .fetch_blender_installation_locations(app, state, id, limit, directory_path, is_default)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_delete_blender_installation_location(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    match BlenderInstallationLocationServiceImpl
        .delete_blender_installation_location(app, state, id)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
