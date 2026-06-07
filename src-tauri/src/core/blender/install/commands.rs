
use function_name::named;
use tauri::AppHandle;

use crate::{
    core::{
        format_command_error, BlenderInstallServiceImpl, DownloadableBlenderVersion,
        TBlenderInstallService, COLON_SEPERATOR,
    },
    database::{BlenderInstallationLocation, BlenderVersion},
    AppState,
};

#[named]
#[tauri::command]
pub async fn cmd_init_blender_version(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    downloadable_blender_version: DownloadableBlenderVersion,
    blender_installation_location: BlenderInstallationLocation,
) -> Result<BlenderVersion, String> {
    match BlenderInstallServiceImpl
        .init_blender_version(
            app,
            state,
            downloadable_blender_version,
            blender_installation_location,
        )
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_install_blender_version(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: Option<String>,
    archive_file_path: std::path::PathBuf,
) -> Result<(), String> {
    match BlenderInstallServiceImpl
        .install_blender_version(app, state, id, archive_file_path)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_write_blender_version_download_data(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    downloadable_blender_version: DownloadableBlenderVersion,
    directory_path: std::path::PathBuf,
) -> Result<(), String> {
    match BlenderInstallServiceImpl
        .write_blender_version_download_data(
            app,
            state,
            downloadable_blender_version,
            directory_path,
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_update_blender_version(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    blender_version: BlenderVersion,
) -> Result<(), String> {
    match BlenderInstallServiceImpl
        .update_blender_version(app, state, blender_version)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_set_blender_version_as_default(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: Option<String>,
) -> Result<(), String> {
    match BlenderInstallServiceImpl
        .set_blender_version_as_default(app, state, id)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_update_blender_version_download_status_type(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    blender_version: BlenderVersion,
    download_status_type: &str,
) -> Result<(), String> {
    match BlenderInstallServiceImpl
        .update_blender_version_download_status_type(
            app,
            state,
            blender_version,
            download_status_type,
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_fetch_blender_versions(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: Option<String>,
    limit: Option<i64>,
    is_default: Option<bool>,
    executable_file_path: Option<String>,
    series: Option<String>,
    order: &str,
    download_status_types: Option<Vec<String>>,
) -> Result<Vec<BlenderVersion>, String> {
    match BlenderInstallServiceImpl
        .refresh_blender_versions(app.clone(), state.clone())
        .await
    {
        Ok(_) => {}
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
    match BlenderInstallServiceImpl
        .set_default_blender_version(app.clone(), state.clone())
        .await
    {
        Ok(_) => {}
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
    match BlenderInstallServiceImpl
        .fetch_blender_versions(
            app,
            state,
            id,
            limit,
            is_default,
            executable_file_path,
            series,
            order,
            download_status_types,
        )
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_update_install_blender_build_type(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    code: String,
) -> Result<(), String> {
    match BlenderInstallServiceImpl
        .update_install_blender_build_type(app, state, code)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_launch_blender_version(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    match BlenderInstallServiceImpl
        .launch_blender_version(app, state, id)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_delete_blender_version(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    match BlenderInstallServiceImpl
        .delete_blender_version(app, state, id)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
