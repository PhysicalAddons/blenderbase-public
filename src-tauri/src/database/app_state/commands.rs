use function_name::named;
use tauri::AppHandle;

use crate::{
    core::{format_command_error, COLON_SEPERATOR},
    database::{AppState, AppStateImpl, TAppState},
};

#[named]
#[tauri::command]
pub async fn cmd_app_db_init(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    match AppStateImpl.app_db_init(app, state).await {
        Ok(_) => Ok(()),
        Err(e) => return Err(format_command_error(function_name!(), COLON_SEPERATOR, e).await),
    }
}
