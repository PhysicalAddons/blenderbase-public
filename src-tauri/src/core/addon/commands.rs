use function_name::named;
use tauri::AppHandle;

use crate::{
    core::{format_command_error, AddonServiceImpl, TAddonService, COLON_SEPERATOR},
    AppState,
};

#[named]
#[tauri::command]
async fn cmd_insert_addon(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    match AddonServiceImpl.insert_addon(app, state).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
async fn cmd_insert_and_refresh_addons(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    match AddonServiceImpl.insert_and_refresh_addons(app, state).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
async fn cmd_update_addon(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    match AddonServiceImpl.update_addon(app, state).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
async fn cmd_fetch_addons(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    match AddonServiceImpl.fetch_addons(app, state).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
async fn cmd_delete_addon(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    match AddonServiceImpl.delete_addon(app, state).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
async fn cmd_toggle_addon(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    match AddonServiceImpl.toggle_addon(app, state).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
async fn cmd_symlink_addon(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    match AddonServiceImpl.symlink_addon(app, state).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
async fn cmd_reveal_addon_in_local_file_system(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    match AddonServiceImpl
        .reveal_addon_in_local_file_system(app, state)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
async fn cmd_update_addon_filter(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    match AddonServiceImpl.update_addon_filter(app, state).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
