use function_name::named;
use tauri::AppHandle;

use crate::{
    core::{
        format_command_error, setup_file_argument, SeriesApplyReport, SetupApplyOptions, SetupBundleInfo,
        SetupExportOptions, SetupServiceImpl, SetupSyncStatus, TSetupService, COLON_SEPERATOR,
    },
    AppState,
};

#[named]
#[tauri::command]
pub async fn cmd_export_setup_bundle(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    file_path: String,
    options: Option<SetupExportOptions>,
) -> Result<SetupBundleInfo, String> {
    match SetupServiceImpl
        .export_setup_bundle(app, state, file_path, options.unwrap_or_default())
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_inspect_setup_bundle(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    file_path: String,
) -> Result<SetupBundleInfo, String> {
    match SetupServiceImpl
        .inspect_setup_bundle(app, state, file_path)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_apply_setup_bundle(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    file_path: String,
    options: Option<SetupApplyOptions>,
) -> Result<Vec<SeriesApplyReport>, String> {
    match SetupServiceImpl
        .apply_setup_bundle(app, state, file_path, options.unwrap_or_default())
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_undo_setup_apply(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    series: String,
) -> Result<usize, String> {
    match SetupServiceImpl.undo_setup_apply(app, state, series).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

/// The setup file this process was started with, when a `.bbsetup` was opened with the app
/// (double-click once the installer registers the extension, or "Open with"). The frontend
/// asks once at startup and shows the file in the restore view.
#[tauri::command]
pub async fn cmd_startup_setup_file() -> Result<Option<String>, String> {
    Ok(setup_file_argument(std::env::args().skip(1)))
}

#[named]
#[tauri::command]
pub async fn cmd_get_setup_sync(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<SetupSyncStatus, String> {
    match SetupServiceImpl.get_setup_sync(app, state).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_set_setup_sync_folder(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    folder_path: Option<String>,
) -> Result<SetupSyncStatus, String> {
    match SetupServiceImpl.set_setup_sync_folder(app, state, folder_path).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_save_setup_to_sync_folder(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    options: Option<SetupExportOptions>,
) -> Result<SetupBundleInfo, String> {
    match SetupServiceImpl
        .save_setup_to_sync_folder(app, state, options.unwrap_or_default())
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_mark_setup_synced(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    content_hash: String,
) -> Result<SetupSyncStatus, String> {
    match SetupServiceImpl.mark_setup_synced(app, state, content_hash).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
