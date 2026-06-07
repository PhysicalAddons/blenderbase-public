use std::sync::Mutex;

use tauri::Manager;
mod core;
mod database;
mod infrastructure;
use crate::{core::*, database::*};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let mut base_dir = dirs::data_dir().expect("Failed to get data dir.");
    base_dir.push(COM_PHYSICALADDONS_BLENDERBASE);
    std::fs::create_dir_all(&base_dir).expect("Failed to create app data directory.");
    base_dir.push(TEST_DB);
    if !base_dir.exists() {
        std::fs::File::create(&base_dir).expect("Failed to create database file.");
    }
    let db_url = format!("{}{}", SQLITE_PREFIX, base_dir.to_string_lossy());
    let pool = sqlx::SqlitePool::connect(&db_url)
        .await
        .expect("Failed to connect to database.");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations.");
    let http_client = reqwest::Client::new();
    let action_timeouts = ActionTimestamp::default();
    let app_state = AppState {
        pool,
        http_client,
        action_timeouts: Mutex::new(action_timeouts),
    };
    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_sql::Builder::new().build()) // TODO use only defauilt sqlx or the tauri one.
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_upload::init())
        .setup(|app| {
            // Opens the developer tools when run in debug.
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd_app_db_init,
            cmd_app_settings_init,
            cmd_fetch_app_setting_type,
            cmd_fetch_app_setting,
            cmd_fetch_input_value_type,
            cmd_handle_setting,
            cmd_insert_blender_installation_location,
            cmd_set_blender_installation_location_as_default,
            cmd_fetch_blender_installation_locations,
            cmd_delete_blender_installation_location,
            cmd_fetch_blender_version_build_types,
            cmd_update_download_blender_build_type,
            cmd_get_downloadable_blender_version_data,
            cmd_refresh_blend_files,
            cmd_fetch_blend_files,
            cmd_fetch_blender_series,
            cmd_instance_popup_window,
            cmd_check_internet_connection,
            cmd_init_blender_version,
            cmd_update_blender_version_download_status_type,
            cmd_install_blender_version,
            cmd_fetch_blender_versions,
            cmd_fetch_download_status_type,
            cmd_launch_blender_version,
            cmd_update_blender_series,
            cmd_open_blend_file,
            cmd_write_blender_version_download_data,
            cmd_set_blender_version_as_default,
            cmd_delete_blender_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running Blenderbase application");
}
