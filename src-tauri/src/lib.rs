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
    // BLENDERBASE_DATA_DIR points a build at another data folder, so a development build
    // can run beside the installed app without sharing (or touching) its database.
    let mut base_dir = match std::env::var("BLENDERBASE_DATA_DIR") {
        Ok(v) if !v.trim().is_empty() => std::path::PathBuf::from(v.trim()),
        _ => match dirs::data_dir() {
            Some(v) => v.join(COM_PHYSICALADDONS_BLENDERBASE),
            None => return Err(String::from("Could not determine the user data directory.")),
        },
    };
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
    let pool = open_pool(options, &base_dir).await?;
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

/// Why the database could not be used, as [`open_pool`] sees it.
enum Refusal {
    /// SQLite refuses the file as not a usable database (SQLITE_CORRUPT, SQLITE_NOTADB or
    /// a failed integrity check); setting files aside may heal that.
    Unusable(String),
    /// Anything else, reported as it is.
    Other(String),
}

/// SQLITE_CORRUPT (11) and SQLITE_NOTADB (26) both mean the file is not a usable database.
fn is_unusable_database(e: &sqlx::Error) -> bool {
    match e {
        sqlx::Error::Database(d) => {
            matches!(d.code().as_deref(), Some("11") | Some("26"))
                || d.message().contains("malformed")
                || d.message().contains("not a database")
        }
        _ => false,
    }
}

/// Opens the database and runs SQLite's quick integrity check. A file SQLite refuses is
/// healed in two steps, each keeping what it sets aside in a dated `corrupt-<timestamp>`
/// folder next to the database, so nothing is lost that a recovery tool could still read.
/// First only the WAL sidecars go: a fresh connection replays the WAL before anything else,
/// and a damaged WAL makes every new connection fail while the main file is sound (seen on
/// 2026-09-30). Then, if the main file itself is refused, it goes too and a fresh database
/// is started; everything in it can be rebuilt (locations are re-added, versions rescanned,
/// caches refilled). A damaged database must not stop the app.
async fn open_pool(
    options: SqliteConnectOptions,
    db_path: &std::path::Path,
) -> Result<sqlx::SqlitePool, String> {
    let dir = db_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let aside = dir.join(format!("corrupt-{}", chrono::Local::now().format("%Y%m%d-%H%M%S")));
    let mut last = String::new();
    for stage in 0..3 {
        match stage {
            1 => {
                if move_aside(db_path, &aside, &["-wal", "-shm", "-journal"])? > 0 {
                    eprintln!(
                        "The database could not be used ({}); its WAL sidecars have been moved to {} and the main file is tried on its own.",
                        last,
                        aside.display()
                    );
                }
            }
            2 => {
                move_aside(db_path, &aside, &[""])?;
                eprintln!(
                    "The database file is unusable ({}); it has been moved to {} and a fresh one is started.",
                    last,
                    aside.display()
                );
            }
            _ => {}
        }
        // A read-only look first. A read-write connection that is refused can, as it closes,
        // checkpoint a damaged WAL into the main file and ruin it; a read-only one never
        // checkpoints, so a refusal here costs nothing.
        match probe_read_only(&options, db_path).await {
            Ok(()) => {}
            Err(Refusal::Unusable(detail)) => {
                last = detail;
                continue;
            }
            Err(Refusal::Other(message)) => {
                eprintln!("Could not look at the database read-only ({}); opening it directly.", message);
            }
        }
        match try_open_and_check(&options, db_path).await {
            Ok(pool) => return Ok(pool),
            Err(Refusal::Unusable(detail)) => last = detail,
            Err(Refusal::Other(message)) => return Err(message),
        }
    }
    Err(format!("Could not open the database {}: {}", db_path.display(), last))
}

/// Connects read-only and runs `PRAGMA quick_check`, without the power to change anything.
/// A missing file passes (a fresh database is about to be created); read-only quirks are
/// reported as `Other` and not held against the database.
async fn probe_read_only(
    options: &SqliteConnectOptions,
    db_path: &std::path::Path,
) -> Result<(), Refusal> {
    if !db_path.exists() {
        return Ok(());
    }
    let probe = options.clone().read_only(true).create_if_missing(false);
    let pool = match SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(probe)
        .await
    {
        Ok(v) => v,
        Err(e) if is_unusable_database(&e) => return Err(Refusal::Unusable(e.to_string())),
        Err(e) => return Err(Refusal::Other(e.to_string())),
    };
    let verdict: Result<String, sqlx::Error> = sqlx::query_scalar("PRAGMA quick_check")
        .fetch_one(&pool)
        .await;
    pool.close().await;
    match verdict {
        Ok(v) if v == "ok" => Ok(()),
        Ok(v) => Err(Refusal::Unusable(format!("quick_check: {}", v))),
        Err(e) if is_unusable_database(&e) => Err(Refusal::Unusable(e.to_string())),
        Err(e) => Err(Refusal::Other(e.to_string())),
    }
}

/// One attempt: connect, then `PRAGMA quick_check`. The pool is closed again on a refusal.
async fn try_open_and_check(
    options: &SqliteConnectOptions,
    db_path: &std::path::Path,
) -> Result<sqlx::SqlitePool, Refusal> {
    let pool = match SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options.clone())
        .await
    {
        Ok(v) => v,
        Err(e) if is_unusable_database(&e) => return Err(Refusal::Unusable(e.to_string())),
        Err(e) => {
            return Err(Refusal::Other(format!(
                "Could not open the database {}: {}",
                db_path.display(),
                e
            )))
        }
    };
    let verdict: Result<String, sqlx::Error> = sqlx::query_scalar("PRAGMA quick_check")
        .fetch_one(&pool)
        .await;
    let refusal = match verdict {
        Ok(v) if v == "ok" => return Ok(pool),
        Ok(v) => Refusal::Unusable(format!("quick_check: {}", v)),
        Err(e) if is_unusable_database(&e) => Refusal::Unusable(e.to_string()),
        Err(e) => Refusal::Other(format!("Could not check the database: {}", e)),
    };
    pool.close().await;
    Err(refusal)
}

/// Moves the database files with the given suffixes into `folder` (created when the first
/// file goes in) and returns how many were moved. A file another process still holds makes
/// this fail rather than heal halfway.
fn move_aside(
    db_path: &std::path::Path,
    folder: &std::path::Path,
    suffixes: &[&str],
) -> Result<usize, String> {
    let dir = db_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let name = db_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("test.db"));
    let mut moved = 0;
    for suffix in suffixes {
        let from = dir.join(format!("{}{}", name, suffix));
        if !from.exists() {
            continue;
        }
        std::fs::create_dir_all(folder)
            .map_err(|e| format!("Could not create {}: {}", folder.display(), e))?;
        let to = folder.join(format!("{}{}", name, suffix));
        // The connection that was just refused may still be letting go of its handle, and
        // on Windows an open handle blocks a rename; a few short retries cover that. A file
        // another process holds (a second running instance) stays blocked and is reported.
        let mut attempt = 0;
        loop {
            match std::fs::rename(&from, &to) {
                Ok(()) => break,
                Err(_) if attempt < 40 => {
                    attempt += 1;
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(e) => {
                    // Nothing went in: leave no empty folder behind.
                    if moved == 0 {
                        let _ = std::fs::remove_dir(folder);
                    }
                    return Err(format!("Could not move {} aside: {}", from.display(), e));
                }
            }
        }
        moved += 1;
    }
    Ok(moved)
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
            cmd_apply_addon,
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
            cmd_lan_receive,
            cmd_sweep_blender_installations
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

    /// The connection options the app uses, for a database at `db_path`.
    fn app_options(db_path: &std::path::Path) -> SqliteConnectOptions {
        let url = format!("sqlite://{}", db_path.to_string_lossy());
        SqliteConnectOptions::from_str(&url)
            .unwrap()
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true)
    }

    /// The dated folders that [`open_pool`] sets damaged files aside in.
    fn quarantine_folders(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("corrupt-"))
            .map(|e| e.path())
            .collect()
    }

    /// A file that is not a database must be set aside, not fatal: the app
    /// continues on a fresh database and the damaged file is kept.
    #[tokio::test]
    async fn quarantines_a_damaged_database_and_starts_fresh() {
        let dir = std::env::temp_dir().join(format!("blenderbase-corrupt-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("test.db");
        let garbage = b"this is definitely not a sqlite database, just bytes\n".repeat(40);
        std::fs::write(&db_path, &garbage).unwrap();

        let pool = open_pool(app_options(&db_path), &db_path).await.unwrap();
        let quarantined = quarantine_folders(&dir);
        assert_eq!(quarantined.len(), 1, "one dated quarantine folder");
        assert_eq!(
            std::fs::read(quarantined[0].join("test.db")).unwrap(),
            garbage,
            "the damaged file is kept inside it"
        );
        assert!(db_path.exists(), "a fresh database took its place");
        sqlx::migrate!().run(&pool).await.unwrap();
        let ok: String = sqlx::query_scalar("PRAGMA quick_check").fetch_one(&pool).await.unwrap();
        assert_eq!(ok, "ok");
        pool.close().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A WAL file whose single frame replaces page 1 with garbage. Its header and frame
    /// checksums are right, so SQLite replays it, and then finds no database header: that is
    /// how a damaged WAL presents itself to every fresh connection.
    fn poisoned_wal(page_size: u32, page_count: u32) -> Vec<u8> {
        // SQLite's WAL checksum: pairs of 32-bit words, s0 += x0 + s1; s1 += x1 + s0.
        fn checksum(bytes: &[u8], (mut s0, mut s1): (u32, u32)) -> (u32, u32) {
            for chunk in bytes.chunks_exact(8) {
                let x0 = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                let x1 = u32::from_be_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                s0 = s0.wrapping_add(x0.wrapping_add(s1));
                s1 = s1.wrapping_add(x1.wrapping_add(s0));
            }
            (s0, s1)
        }
        let (salt1, salt2) = (0x1111_1111u32, 0x2222_2222u32);
        let mut header = Vec::new();
        header.extend_from_slice(&0x377f_0683u32.to_be_bytes()); // magic: big-endian checksums
        header.extend_from_slice(&3_007_000u32.to_be_bytes()); // format
        header.extend_from_slice(&page_size.to_be_bytes());
        header.extend_from_slice(&0u32.to_be_bytes()); // checkpoint sequence
        header.extend_from_slice(&salt1.to_be_bytes());
        header.extend_from_slice(&salt2.to_be_bytes());
        let (h0, h1) = checksum(&header, (0, 0));
        header.extend_from_slice(&h0.to_be_bytes());
        header.extend_from_slice(&h1.to_be_bytes());
        let page = vec![0xAAu8; page_size as usize];
        let mut frame = Vec::new();
        frame.extend_from_slice(&1u32.to_be_bytes()); // page number
        frame.extend_from_slice(&page_count.to_be_bytes()); // commit frame: database size after it
        frame.extend_from_slice(&salt1.to_be_bytes());
        frame.extend_from_slice(&salt2.to_be_bytes());
        let (f0, f1) = checksum(&page, checksum(&frame[..8], (h0, h1)));
        frame.extend_from_slice(&f0.to_be_bytes());
        frame.extend_from_slice(&f1.to_be_bytes());
        let mut wal = header;
        wal.extend_from_slice(&frame);
        wal.extend_from_slice(&page);
        wal
    }

    /// A sound database with a damaged WAL beside it must open with its data intact: only
    /// the sidecars are set aside, the main file stays.
    #[tokio::test]
    async fn heals_a_database_whose_wal_is_unreadable() {
        let dir = std::env::temp_dir().join(format!("blenderbase-wal-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("test.db");
        let pool = open_pool(app_options(&db_path), &db_path).await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        sqlx::query("INSERT INTO blender_series (id, series, config_directory_path) VALUES ('s1', '5.2', 'x')")
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await; // checkpoints and drops the WAL
        let _ = std::fs::remove_file(dir.join("test.db-shm"));
        assert!(quarantine_folders(&dir).is_empty(), "a sound database sets nothing aside");

        let main = std::fs::read(&db_path).unwrap();
        let page_size = match u16::from_be_bytes([main[16], main[17]]) as u32 {
            1 => 65_536,
            v => v,
        };
        let page_count = (main.len() as u32) / page_size;
        std::fs::write(dir.join("test.db-wal"), poisoned_wal(page_size, page_count)).unwrap();
        // Read-only, like the heal's own look: a read-write connection closing after this
        // refusal would checkpoint the garbage page into the main file.
        let refused = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(app_options(&db_path).read_only(true))
            .await;
        match refused {
            Err(ref e) if is_unusable_database(e) => {}
            other => panic!("a plain connection must be refused as unusable, got {:?}", other.map(|_| ())),
        }
        assert!(dir.join("test.db-wal").exists(), "the read-only look leaves the WAL in place");

        let pool = open_pool(app_options(&db_path), &db_path).await.unwrap();
        let series: String = sqlx::query_scalar("SELECT series FROM blender_series WHERE id = 's1'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(series, "5.2", "the data in the main file survives");
        pool.close().await;
        let quarantined = quarantine_folders(&dir);
        assert_eq!(quarantined.len(), 1, "one dated quarantine folder");
        assert!(quarantined[0].join("test.db-wal").exists(), "the damaged WAL is kept");
        assert!(!quarantined[0].join("test.db").exists(), "the main file was not touched");
        assert!(db_path.exists());
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
