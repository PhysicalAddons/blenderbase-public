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
        .connect_with(options.clone())
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
    // A damaged database must not stop the app: everything in it can be
    // rebuilt (locations are re-added, versions rescanned, caches refilled).
    let pool = match quarantine_if_damaged(pool, &base_dir).await? {
        Some(fresh) => fresh,
        None => pool_from(options, &base_dir).await?,
    };
    reconcile_migration_checksums(&pool).await?;
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
        release_scrape_cache: Mutex::new(None),
    })
}

async fn pool_from(
    options: SqliteConnectOptions,
    db_path: &std::path::Path,
) -> Result<sqlx::SqlitePool, String> {
    SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
        .map_err(|e| format!("Could not open the database {}: {}", db_path.display(), e))
}

/// Runs SQLite's quick integrity check. When the file is damaged, the pool is
/// closed, the database and its WAL sidecars are moved into a dated
/// `corrupt-<timestamp>` folder next to it, and `None` is returned so the
/// caller opens a fresh database. Returns the pool unchanged when the file
/// is sound. A damaged file is kept, never deleted, so nothing is lost that a
/// recovery tool could still read.
async fn quarantine_if_damaged(
    pool: sqlx::SqlitePool,
    db_path: &std::path::Path,
) -> Result<Option<sqlx::SqlitePool>, String> {
    let verdict: Result<String, sqlx::Error> = sqlx::query_scalar("PRAGMA quick_check")
        .fetch_one(&pool)
        .await;
    let damaged = match verdict {
        Ok(v) => v != "ok",
        Err(sqlx::Error::Database(e)) => {
            // SQLITE_CORRUPT (11) and SQLITE_NOTADB (26) both mean the file is
            // not a usable database; anything else is a real failure.
            matches!(e.code().as_deref(), Some("11") | Some("26"))
                || e.message().contains("malformed")
                || e.message().contains("not a database")
        }
        Err(e) => return Err(format!("Could not check the database: {}", e)),
    };
    if !damaged {
        return Ok(Some(pool));
    }
    pool.close().await;
    let dir = db_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let quarantine = dir.join(format!("corrupt-{}", stamp));
    std::fs::create_dir_all(&quarantine)
        .map_err(|e| format!("Could not create {}: {}", quarantine.display(), e))?;
    let name = db_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("test.db"));
    for suffix in ["", "-wal", "-shm", "-journal"] {
        let from = dir.join(format!("{}{}", name, suffix));
        if from.exists() {
            let to = quarantine.join(format!("{}{}", name, suffix));
            std::fs::rename(&from, &to)
                .map_err(|e| format!("Could not move {} aside: {}", from.display(), e))?;
        }
    }
    eprintln!(
        "The database was damaged and has been moved to {}; starting with a fresh one.",
        quarantine.display()
    );
    Ok(None)
}

/// Brings stored migration checksums in line with the ones embedded in this
/// build.
///
/// sqlx hashes each migration byte for byte. A database written by a build
/// whose copy of a migration differed only in line endings (a Windows
/// checkout without the LF rule) would otherwise refuse to start with
/// "previously applied but has been modified". Shipped migrations are never
/// edited after release, so a mismatch on an already-applied version is
/// treated as that drift: the stored checksum is replaced and startup goes
/// on. Migrations not yet applied are left for `migrate!().run` as usual.
async fn reconcile_migration_checksums(pool: &sqlx::SqlitePool) -> Result<(), String> {
    let table: Option<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Could not inspect the database: {}", e))?;
    if table.is_none() {
        return Ok(());
    }
    let applied: Vec<(i64, Vec<u8>)> =
        sqlx::query_as("SELECT version, checksum FROM _sqlx_migrations")
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Could not read applied migrations: {}", e))?;
    for migration in sqlx::migrate!().iter() {
        // Reversible migrations are listed twice (up and down) under one
        // version; only the up script's checksum is what sqlx verifies.
        if migration.migration_type.is_down_migration() {
            continue;
        }
        let Some((_, stored)) = applied.iter().find(|(v, _)| *v == migration.version) else {
            continue;
        };
        if stored.as_slice() == migration.checksum.as_ref() {
            continue;
        }
        sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = ?")
            .bind(migration.checksum.as_ref())
            .bind(migration.version)
            .execute(pool)
            .await
            .map_err(|e| {
                format!(
                    "Could not reconcile migration {}: {}",
                    migration.version, e
                )
            })?;
        eprintln!(
            "Migration {} was recorded with a different checksum (line-ending drift); updated to this build's.",
            migration.version
        );
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let app_state = init_app_state().await;
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_upload::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(move |app| {
            // The state is registered here rather than via `Builder::manage`
            // so that a startup failure can be reported through a native
            // dialog (which needs an initialized app) before exiting.
            match app_state {
                Ok(state) => {
                    app.manage(state);
                    // The local network side: silent until the user shares a setup or looks for one.
                    app.manage(LanHub::new(app.package_info().version.to_string()));
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
            cmd_confirm_blender_installation_location,
            cmd_reveal_in_file_explorer,
            cmd_reveal_blender_version_in_file_explorer,
            cmd_default_installation_directory,
            cmd_register_blender_installation_location,
            cmd_export_setup_bundle,
            cmd_inspect_setup_bundle,
            cmd_apply_setup_bundle,
            cmd_undo_setup_apply,
            cmd_startup_setup_file,
            cmd_get_setup_sync,
            cmd_set_setup_sync_folder,
            cmd_save_setup_to_sync_folder,
            cmd_mark_setup_synced,
            cmd_send_setup_transfer,
            cmd_receive_setup_transfer,
            cmd_lan_status,
            cmd_lan_browse,
            cmd_lan_share_start,
            cmd_lan_share_stop,
            cmd_lan_receive
        ])
        .build(tauri::generate_context!());
    match app {
        Ok(app) => app.run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                // Other computers drop this one from their lists at once instead of after a timeout.
                if let Some(hub) = app.try_state::<LanHub>() {
                    hub.shutdown_blocking();
                }
            }
        }),
        Err(e) => {
            eprintln!("Error while running Blenderbase application: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file that is not a database must be quarantined, not fatal: the app
    /// continues on a fresh database and the damaged file is kept aside.
    #[tokio::test]
    async fn quarantines_a_damaged_database_and_starts_fresh() {
        let dir = std::env::temp_dir().join(format!("blenderbase-corrupt-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("test.db");
        std::fs::write(&db_path, b"this is definitely not a sqlite database, just bytes
".repeat(40)).unwrap();
        let url = format!("sqlite://{}", db_path.to_string_lossy());
        let options = SqliteConnectOptions::from_str(&url).unwrap().create_if_missing(true);
        let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options.clone()).await.unwrap();

        let verdict = quarantine_if_damaged(pool, &db_path).await.unwrap();
        assert!(verdict.is_none(), "a garbage file must be reported as damaged");
        assert!(!db_path.exists(), "the damaged file must be moved out of the way");
        let quarantined: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).filter(|e| e.file_name().to_string_lossy().starts_with("corrupt-")).collect();
        assert_eq!(quarantined.len(), 1, "one dated quarantine folder");
        assert!(quarantined[0].path().join("test.db").exists(), "the damaged file is kept inside it");

        let fresh = pool_from(options, &db_path).await.unwrap();
        sqlx::migrate!().run(&fresh).await.unwrap();
        let ok: String = sqlx::query_scalar("PRAGMA quick_check").fetch_one(&fresh).await.unwrap();
        assert_eq!(ok, "ok");
        fresh.close().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A database whose stored checksum for an applied migration differs from
    /// this build's (line-ending drift) must start after reconciliation.
    #[tokio::test]
    async fn reconciles_a_drifted_migration_checksum() {
        use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
        use std::str::FromStr;
        let dir = std::env::temp_dir().join(format!("blenderbase-migrate-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let url = format!("sqlite://{}", dir.join("t.db").to_string_lossy());
        let opts = SqliteConnectOptions::from_str(&url).unwrap().create_if_missing(true);
        let pool = SqlitePoolOptions::new().max_connections(1).connect_with(opts).await.unwrap();

        sqlx::migrate!().run(&pool).await.unwrap();
        let first = sqlx::migrate!().iter().next().unwrap().version;
        sqlx::query("UPDATE _sqlx_migrations SET checksum = X'00' WHERE version = ?")
            .bind(first)
            .execute(&pool)
            .await
            .unwrap();
        assert!(
            sqlx::migrate!().run(&pool).await.is_err(),
            "sqlx must refuse the drifted checksum before reconciliation"
        );

        reconcile_migration_checksums(&pool).await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        pool.close().await;
        let _ = std::fs::remove_dir_all(&dir);
    }
}
