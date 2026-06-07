use crate::{
    core::{
        check_internet_connection, format_command_error, instance_popup_window, COLON_SEPERATOR,
    },
    AppState,
};
use function_name::named;
use tauri::AppHandle;

#[named]
#[tauri::command]
pub async fn cmd_check_internet_connection(
    state: tauri::State<'_, AppState>,
    is_override: Option<bool>,
) -> Result<Option<bool>, String> {
    match check_internet_connection(state, is_override).await {
        Ok(v) => Ok(v),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
#[named]
#[tauri::command]
pub async fn cmd_instance_popup_window(
    app: AppHandle,
    label: String,
    title: String,
    url_path: String,
    width: Option<f64>,
    height: Option<f64>,
    resizeable: Option<bool>,
    always_on_top: Option<bool>,
    focused: Option<bool>,
    skip_taskbar: Option<bool>,
) -> Result<(), String> {
    match instance_popup_window(
        app,
        label,
        title,
        url_path,
        width,
        height,
        resizeable,
        always_on_top,
        focused,
        skip_taskbar,
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
