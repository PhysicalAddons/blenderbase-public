use std::{str::FromStr, sync::Mutex};

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
mod core;
mod database;
mod infrastructure;
use crate::{core::*, database::*};

/// Prepares the application data directory and database.
///
/// Every failure is returned as a message instead of panicking, so the caller
/// can show it to the user. In release builds the process has no console
/// (see `windows_subsystem` in main.rs), so a panic here would otherwise exit
/// silently with nothing to diagnose.
async fn init_app_state() -> Result<AppState, String> {
    let mut base_dir = match dirs::data_dir() {
        Some(v) => v,
        None => return Err(String::from("Could not determine the user data directory.")),
    };
    base_dir.push(COM_PHYSICALADDONS_BLENDERBASE);
    if let Err(e) = std::fs::create_dir_all(&base_dir) {
        return Err(format!(
            "Could not create the app data directory {}: {}",
            base_dir.display(),
            e
        ));
    }
    base_dir.push(TEST_DB);
    let db_url = format!("{}{}", SQLITE_PREFIX, base_dir.to_string_lossy());
    // WAL lets reads proceed while a write is in flight; the busy timeout
    // covers the short lock handoffs between them instead of failing at once.
    let options = match SqliteConnectOptions::from_str(&db_url) {
        Ok(v) => v
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_secs(5)),
        Err(e) => {
            return Err(format!(
                "Could not parse the database location {}: {}",
                base_dir.display(),
                e
            ))
        }
    };
    let pool = match SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Could not open the database {}: {}",
                base_dir.display(),
                e
            ))
        }
    };
    if let Err(e) = sqlx::migrate!().run(&pool).await {
        return Err(format!(
            "Could not update the database {}: {}",
            base_dir.display(),
            e
        ));
    }
    let http_client = reqwest::Client::new();
    let action_timeouts = ActionTimestamp::default();
    Ok(AppState {
        pool,
        http_client,
        action_timeouts: Mutex::new(action_timeouts),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let app_state = init_app_state().await;
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_upload::init())
        .setup(move |app| {
            // The state is registered here rather than via `Builder::manage`
            // so that a startup failure can be reported through a native
            // dialog (which needs an initialized app) before exiting.
            match app_state {
                Ok(state) => {
                    app.manage(state);
                }
                Err(message) => {
                    app.dialog()
                        .message(format!("Blenderbase could not start.\n\n{}", message))
                        .title("Blenderbase")
                        .kind(MessageDialogKind::Error)
                        .buttons(MessageDialogButtons::Ok)
                        .blocking_show();
                    std::process::exit(1);
                }
            }
            // DevTools are not opened at startup: an attached DevTools makes the WebView draw a
            // "W × H" size overlay while the window is resized. F12 still opens them in debug.
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
            cmd_refresh_blender_versions,
            cmd_fetch_download_status_type,
            cmd_launch_blender_version,
            cmd_update_blender_series,
            cmd_open_blend_file,
            cmd_write_blender_version_download_data,
            cmd_set_blender_version_as_default,
            cmd_delete_blender_version,
            cmd_fetch_addons,
            cmd_refresh_addons,
            cmd_toggle_addon,
            cmd_install_addon,
            cmd_symlink_addon,
            cmd_delete_addon,
            cmd_reveal_addon_in_file_explorer,
            cmd_refresh_blender_version_details,
            cmd_confirm_blender_installation_location
        ])
        .run(tauri::generate_context!());
    if let Err(e) = app {
        eprintln!("Error while running Blenderbase application: {}", e);
        std::process::exit(1);
    }
}
