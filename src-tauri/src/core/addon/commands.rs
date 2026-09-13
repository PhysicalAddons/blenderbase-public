use function_name::named;
use tauri::AppHandle;

use crate::{
    core::{format_command_error, AddonServiceImpl, TAddonService, COLON_SEPERATOR},
    database::Addon,
    AppState,
};

#[named]
#[tauri::command]
pub async fn cmd_fetch_addons(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    blender_version_id: String,
) -> Result<Vec<Addon>, String> {
    match AddonServiceImpl
        .fetch_addons(app, state, blender_version_id)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_refresh_addons(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    blender_version_id: String,
) -> Result<Vec<Addon>, String> {
    match AddonServiceImpl
        .refresh_addons(app, state, blender_version_id)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_toggle_addon(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: String,
    is_enabled: bool,
) -> Result<Addon, String> {
    match AddonServiceImpl.toggle_addon(app, state, id, is_enabled).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_install_addon(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    blender_version_id: String,
    file_path: String,
) -> Result<Vec<Addon>, String> {
    match AddonServiceImpl
        .install_addon(app, state, blender_version_id, file_path)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_symlink_addon(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    blender_version_id: String,
    directory_path: String,
) -> Result<Vec<Addon>, String> {
    match AddonServiceImpl
        .symlink_addon(app, state, blender_version_id, directory_path)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_delete_addon(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Vec<Addon>, String> {
    match AddonServiceImpl.delete_addon(app, state, id).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_reveal_addon_in_file_explorer(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    match AddonServiceImpl
        .reveal_addon_in_file_explorer(app, state, id)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
