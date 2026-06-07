use function_name::named;
use tauri::AppHandle;

use crate::{
    core::{format_command_error, AppSettingsServiceImpl, TAppSettingsService, COLON_SEPERATOR},
    database::{AppSetting, AppSettingType, InputValueType},
    AppState,
};

#[named]
#[tauri::command]
pub async fn cmd_app_settings_init(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    match AppSettingsServiceImpl.app_settings_init(app, state).await {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}

#[named]
#[tauri::command]
pub async fn cmd_fetch_app_setting(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: Option<i64>,
    limit: Option<i64>,
    code: Option<String>,
    is_read_on_app_launch: Option<bool>,
    app_setting_type_id: Option<i64>,
) -> Result<Vec<AppSetting>, String> {
    match AppSettingsServiceImpl
        .fetch_app_setting(
            app,
            state,
            id,
            limit,
            code,
            is_read_on_app_launch,
            app_setting_type_id,
        )
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_fetch_app_setting_type(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: Option<i64>,
    limit: Option<i64>,
    code: Option<String>,
) -> Result<Vec<AppSettingType>, String> {
    match AppSettingsServiceImpl
        .fetch_app_setting_type(app, state, id, limit, code)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_fetch_input_value_type(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: Option<i64>,
    limit: Option<i64>,
    code: Option<String>,
) -> Result<Vec<InputValueType>, String> {
    match AppSettingsServiceImpl
        .fetch_input_value_type(app, state, id, limit, code)
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_handle_setting(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    app_setting: AppSetting,
) -> Result<(), String> {
    match AppSettingsServiceImpl
        .handle_setting(app, state, app_setting)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
