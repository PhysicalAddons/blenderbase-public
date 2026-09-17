use function_name::named;
use tauri::AppHandle;

use crate::{
    core::{
        format_command_error, SeriesApplyReport, SetupApplyOptions, SetupBundleInfo, SetupExportOptions,
        SetupServiceImpl, TSetupService, COLON_SEPERATOR,
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
