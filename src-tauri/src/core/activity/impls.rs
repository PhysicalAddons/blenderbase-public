//! Stats, phase 1: counting. The startup script in each Blender series writes a log per
//! session; this service puts that script in place, reads the logs into `blender_session` and
//! answers the hours per version. Nothing here talks to the network.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::AppHandle;

use crate::{
    core::{
        blender_config_root, evaluate_achievements, fetch_achievements, fold_log, install_script,
        remove_script, AchievementDefinition, AchievementStatus, HEARTBEAT_SECONDS,
        COM_PHYSICALADDONS_BLENDERBASE, COUNT_BLENDER_ACTIVITY_UPPERCASE,
    },
    database::{
        AchievementUnlockRepository, ActivityCounterRepository, AppSettingRepository,
        BlenderSeriesRepository, BlenderSessionProgress, BlenderSessionRepository,
        BlenderSessionStart, BlenderVersion, BlenderVersionRepository, BlenderVersionTime,
    },
    AppState,
};

/// The counters Blenderbase keeps by itself (`activity_counter`), bumped from the commands.
pub const APP_COUNTER_ADDON_INSTALLS: &str = "app_addon_installs";
pub const APP_COUNTER_SETUP_SHARED: &str = "app_setup_shared";
pub const APP_COUNTER_SETUP_APPLIED: &str = "app_setup_applied";
pub const APP_COUNTER_BLENDER_INSTALLS: &str = "app_blender_installs";

/// Adds one to a counter Blenderbase keeps for the achievements. Never fails the action it
/// counts: a miss is logged and the action's result stands.
pub async fn bump_app_counter(pool: &SqlitePool, kind: &str) {
    if let Err(e) = ActivityCounterRepository::new(pool).add(kind, 1).await {
        eprintln!("Activity counter {} not updated: {:?}", kind, e);
    }
}

/// The folder under the app data directory the script writes its session logs into.
pub const ACTIVITY_DIRECTORY: &str = "activity";
/// Three missed heartbeats: the session is over unless its process is still alive.
pub const STALE_AFTER_SECONDS: i64 = 3 * HEARTBEAT_SECONDS;
/// After a day without a line the session is over whatever a reused pid says.
pub const STALE_FOR_SURE_SECONDS: i64 = 24 * 3600;

/// What one import run did.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivityImportReport {
    /// The Settings switch is on.
    pub is_counting: bool,
    /// Scripts written or refreshed in series folders on this run.
    pub scripts_written: usize,
    pub files_seen: usize,
    pub sessions_updated: usize,
    pub sessions_finished: usize,
    /// Achievements this run crossed, for the status line.
    pub unlocked: Vec<AchievementDefinition>,
    /// Unlocks nobody has looked at yet, for the title-bar badge.
    pub unseen_unlocks: i64,
}

/// The figures row of the Stats view, over the sessions started since a date.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActivitySummary {
    pub open_seconds: i64,
    pub active_seconds: i64,
    pub longest_session_seconds: i64,
    pub sessions: i64,
    /// Open time of the sessions started since the caller's local midnight, whatever the range.
    pub today_open_seconds: i64,
    /// Open time of the sessions started since the caller's start of the week, whatever the range.
    pub week_open_seconds: i64,
    /// Every counter kind summed over the range.
    pub counters: BTreeMap<String, i64>,
    pub unseen_unlocks: i64,
}

pub trait TActivityService {
    /// Reads whatever is new in the log folder. Runs at app start, on window focus and when
    /// the Stats view opens; with nothing new it costs one directory listing.
    async fn import_activity(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<ActivityImportReport, String>;
    /// Time per version over the sessions started at or after `since` (UTC ISO 8601; None for all time).
    async fn fetch_blender_version_time(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        since: Option<String>,
    ) -> Result<Vec<BlenderVersionTime>, String>;
    /// `since` bounds the range; `today_since` and `week_since` are the caller's local midnight
    /// and start of the week as UTC ISO 8601, for the two figures that ignore the range.
    async fn fetch_activity_summary(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        since: Option<String>,
        today_since: Option<String>,
        week_since: Option<String>,
    ) -> Result<ActivitySummary, String>;
    async fn fetch_achievements(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<Vec<AchievementStatus>, String>;
    /// The Stats view was opened: the badge goes.
    async fn mark_achievements_seen(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
}

pub struct ActivityServiceImpl;

impl TActivityService for ActivityServiceImpl {
    async fn import_activity(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<ActivityImportReport, String> {
        // Held for the whole run: a second caller waits and then finds the files gone or the
        // offsets moved, so no line is counted twice.
        let _one_at_a_time = state.activity_import_lock.lock().await;
        let pool = &state.pool;
        let mut report = ActivityImportReport {
            is_counting: is_counting(pool).await?,
            ..Default::default()
        };
        if report.is_counting {
            // A series installed since the switch went on, or a script older than this build's,
            // gets the current file. Logged, not fatal: the logs are still read.
            match ensure_scripts(pool).await {
                Ok(n) => report.scripts_written = n,
                Err(e) => eprintln!("Activity script install skipped: {}", e),
            }
        }
        let Some(log_dir) = activity_log_dir() else {
            return Err(String::from("Failed import_activity: could not determine the app data directory"));
        };
        if !log_dir.is_dir() {
            return Ok(report);
        }
        let versions = state
            .blender_version_repository()
            .fetch(None, None, None, None, None)
            .await
            .map_err(|e| format!("Failed import_activity: {:?}", e))?;
        let now = Utc::now();
        import_logs(pool, &log_dir, &versions, now, &process_is_alive, &mut report).await?;
        rematch_sessions(pool, &versions).await?;
        // Every run, not only one with new sessions: a build that brings new rules, or the
        // first start after an upgrade, must catch up on what the table already holds.
        report.unlocked = evaluate_achievements(pool, now).await?;
        report.unseen_unlocks = AchievementUnlockRepository::new(pool)
            .count_unseen()
            .await
            .map_err(|e| format!("Failed import_activity: {:?}", e))?;
        Ok(report)
    }

    async fn fetch_blender_version_time(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        since: Option<String>,
    ) -> Result<Vec<BlenderVersionTime>, String> {
        BlenderSessionRepository::new(&state.pool)
            .fetch_version_time(since.as_deref().unwrap_or(""))
            .await
            .map_err(|e| format!("Failed fetch_blender_version_time: {:?}", e))
    }

    async fn fetch_activity_summary(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        since: Option<String>,
        today_since: Option<String>,
        week_since: Option<String>,
    ) -> Result<ActivitySummary, String> {
        let sessions = BlenderSessionRepository::new(&state.pool);
        let failed = |e: sqlx::Error| format!("Failed fetch_activity_summary: {:?}", e);
        let since = since.unwrap_or_default();
        let totals = sessions.fetch_totals(&since).await.map_err(failed)?;
        // Without the caller's local boundaries, a UTC day and the last seven days stand in.
        let now = Utc::now();
        let today_since = today_since.unwrap_or_else(|| {
            format!("{}T00:00:00+00:00", now.format("%Y-%m-%d"))
        });
        let week_since = week_since
            .unwrap_or_else(|| (now - Duration::days(7)).to_rfc3339_opts(SecondsFormat::Secs, false));
        let today = sessions.fetch_totals(&today_since).await.map_err(failed)?;
        let week = sessions.fetch_totals(&week_since).await.map_err(failed)?;
        Ok(ActivitySummary {
            open_seconds: totals.open_seconds,
            active_seconds: totals.active_seconds,
            longest_session_seconds: totals.longest_session_seconds,
            sessions: totals.sessions,
            today_open_seconds: today.open_seconds,
            week_open_seconds: week.open_seconds,
            counters: sessions.fetch_counter_totals(&since).await.map_err(failed)?.into_iter().collect(),
            unseen_unlocks: AchievementUnlockRepository::new(&state.pool)
                .count_unseen()
                .await
                .map_err(failed)?,
        })
    }

    async fn fetch_achievements(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<Vec<AchievementStatus>, String> {
        fetch_achievements(&state.pool, Utc::now()).await
    }

    async fn mark_achievements_seen(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        AchievementUnlockRepository::new(&state.pool)
            .mark_all_seen()
            .await
            .map_err(|e| format!("Failed mark_achievements_seen: {:?}", e))
    }
}

/// The app data folder, as `init_app_state` resolves it (`BLENDERBASE_DATA_DIR` for a
/// development build beside the installed app).
pub fn app_data_dir() -> Option<PathBuf> {
    match std::env::var("BLENDERBASE_DATA_DIR") {
        Ok(v) if !v.trim().is_empty() => Some(PathBuf::from(v.trim())),
        _ => dirs::data_dir().map(|d| d.join(COM_PHYSICALADDONS_BLENDERBASE)),
    }
}

pub fn activity_log_dir() -> Option<PathBuf> {
    app_data_dir().map(|d| d.join(ACTIVITY_DIRECTORY))
}

/// Whether the Settings switch is on.
pub async fn is_counting(pool: &SqlitePool) -> Result<bool, String> {
    let rows = AppSettingRepository::new(pool)
        .fetch(None, None, Some(COUNT_BLENDER_ACTIVITY_UPPERCASE.to_string()), None, None)
        .await
        .map_err(|e| format!("Failed is_counting: {:?}", e))?;
    Ok(rows
        .first()
        .map(|s| s.int_value.unwrap_or(0) != 0)
        .unwrap_or(false))
}

/// The series folder for a stored `blender_series.config_directory_path`: that column holds
/// the `config` folder inside the series folder, so its parent is the series folder.
fn series_folder_of_config(config_directory_path: &str) -> PathBuf {
    let path = Path::new(config_directory_path.trim());
    let is_config = path
        .file_name()
        .map(|n| n.to_string_lossy().eq_ignore_ascii_case("config"))
        .unwrap_or(false);
    match path.parent() {
        Some(parent) if is_config && !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => path.to_path_buf(),
    }
}

/// The oldest series the script runs in: 2.80 brought `depsgraph_update_post` and Python 3.7.
/// Older series stay untouched rather than print an error at every launch.
const OLDEST_SERIES: (u32, u32) = (2, 80);

/// Whether a series name like "4.5" or "5.2" is one the script is written for.
fn is_series_folder_name(name: &str) -> bool {
    let mut parts = name.split('.');
    let (Some(major), Some(minor), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    match (major.parse::<u32>(), minor.parse::<u32>()) {
        (Ok(major), Ok(minor)) => (major, minor) >= OLDEST_SERIES,
        _ => false,
    }
}

/// Every series folder the script belongs in: one per installed version's series (through the
/// stored config folder, else the platform default), plus every series folder already under the
/// config root, so a Blender the user runs outside Blenderbase is counted too.
pub async fn series_directories(pool: &SqlitePool) -> Result<Vec<PathBuf>, String> {
    let versions = BlenderVersionRepository::new(pool)
        .fetch(None, None, None, None, None)
        .await
        .map_err(|e| format!("Failed series_directories: {:?}", e))?;
    let series_rows = BlenderSeriesRepository::new(pool)
        .fetch(None, None, None, None)
        .await
        .map_err(|e| format!("Failed series_directories: {:?}", e))?;
    let root = blender_config_root();
    let mut out: Vec<PathBuf> = Vec::new();
    let mut push = |dir: PathBuf| {
        if !out.contains(&dir) {
            out.push(dir);
        }
    };
    for series in versions.iter().filter_map(|v| v.series.as_deref()) {
        let series = series.trim();
        if !is_series_folder_name(series) {
            continue;
        }
        let known = series_rows
            .iter()
            .find(|r| r.series == series && !r.config_directory_path.trim().is_empty())
            .map(|r| series_folder_of_config(&r.config_directory_path));
        match known.or_else(|| root.as_ref().map(|r| r.join(series))) {
            Some(dir) => push(dir),
            None => {}
        }
    }
    if let Some(root) = &root {
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if entry.path().is_dir() && is_series_folder_name(&name) {
                    push(entry.path());
                }
            }
        }
    }
    Ok(out)
}

/// Puts the current script into every series folder that lacks it. Returns how many were written.
pub async fn ensure_scripts(pool: &SqlitePool) -> Result<usize, String> {
    let Some(log_dir) = activity_log_dir() else {
        return Err(String::from("Could not determine the app data directory"));
    };
    let mut written = 0;
    let mut first_error: Option<String> = None;
    for dir in series_directories(pool).await? {
        match install_script(&dir, &log_dir) {
            Ok(true) => written += 1,
            Ok(false) => {}
            Err(e) => {
                if first_error.is_none() {
                    first_error = Some(e);
                }
            }
        }
    }
    match first_error {
        Some(e) => Err(e),
        None => Ok(written),
    }
}

/// Takes the script out of every series folder. Returns how many were removed.
pub async fn remove_scripts(pool: &SqlitePool) -> Result<usize, String> {
    let mut removed = 0;
    let mut first_error: Option<String> = None;
    for dir in series_directories(pool).await? {
        match remove_script(&dir) {
            Ok(true) => removed += 1,
            Ok(false) => {}
            Err(e) => {
                if first_error.is_none() {
                    first_error = Some(e);
                }
            }
        }
    }
    match first_error {
        Some(e) => Err(e),
        None => Ok(removed),
    }
}

/// The Settings switch changed. Rows are kept either way; only the script comes and goes.
pub async fn apply_activity_switch(pool: &SqlitePool, enabled: bool) -> Result<(), String> {
    if enabled {
        ensure_scripts(pool).await?;
    } else {
        remove_scripts(pool).await?;
    }
    Ok(())
}

/// The session id in a log file name `<UTC start>_<session>.jsonl`.
pub fn session_id_of(file_name: &str) -> Option<String> {
    let stem = file_name.strip_suffix(".jsonl")?;
    let (_, id) = stem.split_once('_')?;
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    Some(id.to_string())
}

/// Lower-cases (on Windows) and forward-slashes a path so two spellings of one place compare equal.
fn normalize_path(path: &str) -> String {
    let s = path.trim().replace('\\', "/");
    let s = s.trim_end_matches('/').to_string();
    if cfg!(windows) {
        s.to_lowercase()
    } else {
        s
    }
}

/// "5.2.1" from Blender's "5.2.1 LTS" or "5.3.0 Alpha": the number alone is what the stored
/// version carries.
fn version_number(version: &str) -> &str {
    version.trim().split_whitespace().next().unwrap_or("")
}

/// The installed version a session belongs to. First by place: the running binary sits inside
/// the version's folder (this covers `blender.exe` against the registered `blender-launcher.exe`
/// and the binary inside a macOS bundle). Then by the registered executable itself. Then by
/// build hash and version number, for a version moved since the log was written.
pub fn match_version<'a>(
    versions: &'a [BlenderVersion],
    binary: &str,
    build_hash: &str,
    version: &str,
) -> Option<&'a BlenderVersion> {
    let binary = normalize_path(binary);
    if binary.is_empty() {
        return None;
    }
    versions
        .iter()
        .find(|v| {
            let dir = normalize_path(&v.installation_directory_path);
            !dir.is_empty() && binary.starts_with(&format!("{}/", dir))
        })
        .or_else(|| {
            versions.iter().find(|v| {
                v.executable_file_path
                    .as_deref()
                    .map(normalize_path)
                    .map(|p| p == binary)
                    .unwrap_or(false)
            })
        })
        .or_else(|| {
            let hash = build_hash.trim().to_lowercase();
            let number = version_number(version);
            if hash.is_empty() || number.is_empty() {
                return None;
            }
            versions.iter().find(|v| {
                let stored = v.hash.as_deref().unwrap_or("").trim().to_lowercase();
                !stored.is_empty()
                    && (stored.starts_with(&hash) || hash.starts_with(&stored))
                    && v.version.as_deref().map(version_number) == Some(number)
            })
        })
}

/// A log timestamp (`at`, `started_at`, `ended_at`: RFC 3339) as UTC; None for anything else.
pub fn parse_log_time(at: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(at.trim())
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

/// Whether a session with no `end` line is over: three heartbeats missed and its process gone,
/// or a whole day silent. An unreadable timestamp is never judged stale.
pub fn is_stale(last_at: &str, now: DateTime<Utc>, pid: i64, alive: &(dyn Fn(i64) -> bool + Sync)) -> bool {
    let Some(at) = parse_log_time(last_at) else {
        return false;
    };
    let age = (now - at).num_seconds();
    if age < STALE_AFTER_SECONDS {
        return false;
    }
    if age >= STALE_FOR_SURE_SECONDS {
        return true;
    }
    pid <= 0 || !alive(pid)
}

/// Whether a process with this id is running right now.
pub fn process_is_alive(pid: i64) -> bool {
    use sysinfo::{Pid, ProcessesToUpdate, System};
    let Ok(pid) = u32::try_from(pid) else {
        return false;
    };
    let pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    system.process(pid).is_some()
}

/// Reads every log in `log_dir` from where the last run stopped, writes the sessions and
/// counters, and deletes the file of each finished session.
pub async fn import_logs(
    pool: &SqlitePool,
    log_dir: &Path,
    versions: &[BlenderVersion],
    now: DateTime<Utc>,
    alive: &(dyn Fn(i64) -> bool + Sync),
    report: &mut ActivityImportReport,
) -> Result<(), String> {
    let repository = BlenderSessionRepository::new(pool);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(log_dir)
        .map_err(|e| format!("Failed import_logs: could not read {}: {}", log_dir.display(), e))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "jsonl").unwrap_or(false))
        .collect();
    paths.sort();
    for path in paths {
        report.files_seen += 1;
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let Some(session_id) = session_id_of(&file_name) else {
            continue;
        };
        let existing = repository
            .fetch_by_id(&session_id)
            .await
            .map_err(|e| format!("Failed import_logs: {:?}", e))?;
        if let Some(s) = &existing {
            if s.is_finished {
                // A leftover from an interrupted run: the row has everything already.
                let _ = std::fs::remove_file(&path);
                continue;
            }
        }
        let bytes = match std::fs::read(&path) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Activity log {} could not be read: {}", path.display(), e);
                continue;
            }
        };
        let mut offset = existing
            .as_ref()
            .map(|s| usize::try_from(s.import_offset).unwrap_or(0))
            .unwrap_or(0);
        if offset > bytes.len() {
            // The file is shorter than what was read before: it was replaced. Start over.
            offset = 0;
        }
        let fold = fold_log(&bytes[offset..]);
        let start = match (&existing, &fold.start) {
            // Nothing to hang a row on yet: the start line is still being written.
            (None, None) => continue,
            (None, Some(s)) => Some(BlenderSessionStart {
                blender_version_id: match_version(versions, &s.binary, &s.build_hash, &s.version)
                    .map(|v| v.id.clone()),
                version: s.version.clone(),
                series: s.series.clone(),
                build_hash: s.build_hash.clone(),
                binary_path: s.binary.clone(),
                pid: s.pid,
                started_at: s.at.clone(),
                log_file_name: file_name.clone(),
            }),
            (Some(_), _) => None,
        };
        let pid = start
            .as_ref()
            .map(|s| s.pid)
            .or_else(|| existing.as_ref().map(|s| s.pid))
            .unwrap_or(0);
        let last_at = if !fold.last_at.is_empty() {
            fold.last_at.clone()
        } else {
            existing.as_ref().map(|s| s.ended_at.clone()).unwrap_or_default()
        };
        let clean = fold.clean_exit || existing.as_ref().map(|s| s.is_clean_exit).unwrap_or(false);
        let finished = clean || is_stale(&last_at, now, pid, alive);
        if fold.consumed == 0 && !finished && start.is_none() {
            continue;
        }
        let progress = BlenderSessionProgress {
            ended_at: last_at,
            open_seconds: fold
                .last_up
                .max(existing.as_ref().map(|s| s.open_seconds).unwrap_or(0)),
            active_seconds_delta: fold.active_seconds,
            is_finished: finished,
            is_clean_exit: clean,
            import_offset: (offset + fold.consumed) as i64,
            counters: fold.counters.into_iter().collect(),
        };
        repository
            .apply_import(&session_id, start.as_ref(), &progress)
            .await
            .map_err(|e| format!("Failed import_logs: {:?}", e))?;
        report.sessions_updated += 1;
        if finished {
            report.sessions_finished += 1;
            if let Err(e) = std::fs::remove_file(&path) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    eprintln!("Activity log {} could not be removed: {}", path.display(), e);
                }
            }
        }
    }
    Ok(())
}

/// Links sessions without a version to one again, so a reinstalled build gets its hours back.
pub async fn rematch_sessions(pool: &SqlitePool, versions: &[BlenderVersion]) -> Result<(), String> {
    let repository = BlenderSessionRepository::new(pool);
    let unmatched = repository
        .fetch_unmatched()
        .await
        .map_err(|e| format!("Failed rematch_sessions: {:?}", e))?;
    for session in unmatched {
        if let Some(v) = match_version(versions, &session.binary_path, &session.build_hash, &session.version) {
            repository
                .set_blender_version(&session.id, Some(&v.id))
                .await
                .map_err(|e| format!("Failed rematch_sessions: {:?}", e))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("blenderbase-import-{}-{}", tag, uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A fresh database with every migration applied. Foreign keys stay off so a version row
    /// needs no location row behind it.
    async fn test_pool(dir: &Path) -> SqlitePool {
        let url = format!("sqlite://{}", dir.join("t.db").to_string_lossy());
        let options = SqliteConnectOptions::from_str(&url)
            .unwrap()
            .create_if_missing(true)
            .foreign_keys(false);
        let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        pool
    }

    fn version(id: &str, dir: &str, exe: &str, hash: &str, number: &str) -> BlenderVersion {
        BlenderVersion {
            id: id.to_string(),
            version: Some(number.to_string()),
            series: Some(number.rsplit_once('.').map(|(s, _)| s.to_string()).unwrap_or_default()),
            hash: Some(hash.to_string()),
            installation_directory_path: dir.to_string(),
            executable_file_path: Some(exe.to_string()),
            ..Default::default()
        }
    }

    async fn insert_version(pool: &SqlitePool, v: &BlenderVersion) {
        sqlx::query(
            "INSERT INTO blender_version (id, version, series, hash, file_size, installation_directory_path, executable_file_path, blender_installation_location_id, download_status_type_id) VALUES (?, ?, ?, ?, 0, ?, ?, 'loc', 1)",
        )
        .bind(&v.id)
        .bind(&v.version)
        .bind(&v.series)
        .bind(&v.hash)
        .bind(&v.installation_directory_path)
        .bind(&v.executable_file_path)
        .execute(pool)
        .await
        .unwrap();
    }

    const T0: &str = "2026-10-06T09:15:02+00:00";

    fn line(kind: &str, up: i64, extra: &str) -> String {
        let at = Utc.with_ymd_and_hms(2026, 10, 6, 9, 15, 2).unwrap() + chrono::Duration::seconds(up);
        format!(
            "{{\"t\":\"{}\",\"at\":\"{}\",\"up\":{}{}}}\n",
            kind,
            at.to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
            up,
            extra
        )
    }

    fn start_line(binary: &str) -> String {
        format!(
            "{{\"t\":\"start\",\"at\":\"{}\",\"up\":0,\"session\":\"abc123\",\"pid\":4242,\"version\":\"5.2.1\",\"version_tuple\":[5,2,1],\"build_hash\":\"9e2066a1b2c3\",\"binary\":{},\"script\":1}}\n",
            T0,
            serde_json::to_string(binary).unwrap()
        )
    }

    #[test]
    fn session_ids_come_from_the_file_name() {
        assert_eq!(session_id_of("20261006T091502Z_3f9c2a7e.jsonl"), Some("3f9c2a7e".to_string()));
        assert_eq!(session_id_of("20261006T091502Z_.jsonl"), None);
        assert_eq!(session_id_of("nounderscore.jsonl"), None);
        assert_eq!(session_id_of("20261006T091502Z_3f9c.txt"), None);
        assert_eq!(session_id_of("x_../etc.jsonl"), None, "only alphanumerics make an id");
    }

    #[test]
    fn versions_match_by_folder_then_executable_then_hash() {
        let versions = vec![
            version("w", "C:\\blenderbaseapps\\5.2.1", "C:\\blenderbaseapps\\5.2.1\\blender-launcher.exe", "9e2066a", "5.2.1"),
            version("w2", "C:\\blenderbaseapps\\5.2.10", "C:\\blenderbaseapps\\5.2.10\\blender-launcher.exe", "1111111", "5.2.10"),
            version("m", "/Applications/Blender 4.5", "/Applications/Blender 4.5/Blender.app", "ab25eae", "4.5.2"),
        ];
        let by_folder = match_version(&versions, "c:/blenderbaseapps/5.2.1/blender.exe", "", "").unwrap();
        assert_eq!(by_folder.id, "w", "blender.exe beside the registered launcher");
        let longer = match_version(&versions, "C:\\blenderbaseapps\\5.2.10\\blender.exe", "", "").unwrap();
        assert_eq!(longer.id, "w2", "a folder that merely starts with another's name is not it");
        let mac = match_version(&versions, "/Applications/Blender 4.5/Blender.app/Contents/MacOS/Blender", "", "").unwrap();
        assert_eq!(mac.id, "m");
        let moved = match_version(&versions, "D:\\elsewhere\\blender.exe", "9E2066A1B2C3", "5.2.1 LTS").unwrap();
        assert_eq!(moved.id, "w", "a moved install still matches on hash prefix and version number");
        assert!(match_version(&versions, "D:\\elsewhere\\blender.exe", "9e2066a", "5.2.2").is_none(), "the version string must agree");
        assert!(match_version(&versions, "", "9e2066a", "5.2.1").is_none());
    }

    #[test]
    fn staleness_needs_missed_heartbeats_and_a_dead_process() {
        let now = Utc.with_ymd_and_hms(2026, 10, 6, 12, 0, 0).unwrap();
        let recent = (now - chrono::Duration::seconds(100)).to_rfc3339();
        let quiet = (now - chrono::Duration::seconds(400)).to_rfc3339();
        let yesterday = (now - chrono::Duration::hours(25)).to_rfc3339();
        let alive = |_: i64| true;
        let dead = |_: i64| false;
        assert!(!is_stale(&recent, now, 1, &dead), "within three heartbeats nothing is judged");
        assert!(!is_stale(&quiet, now, 1, &alive), "the process still runs");
        assert!(is_stale(&quiet, now, 1, &dead));
        assert!(is_stale(&quiet, now, 0, &alive), "no pid to check means gone");
        assert!(is_stale(&yesterday, now, 1, &alive), "a day of silence is over whatever the pid says");
        assert!(!is_stale("garbage", now, 0, &dead));
        assert!(!is_stale("", now, 0, &dead));
    }

    /// A running session is read in two passes: the second adds only the new lines, the end
    /// line finishes the session, and the file goes. Hours land on the matched version.
    #[tokio::test]
    async fn imports_a_session_across_two_reads_and_finishes_it() {
        let dir = temp_dir("db");
        let logs = temp_dir("logs");
        let pool = test_pool(&dir).await;
        let v = version("w", "C:\\blenderbaseapps\\5.2.1", "C:\\blenderbaseapps\\5.2.1\\blender-launcher.exe", "9e2066a", "5.2.1");
        insert_version(&pool, &v).await;
        let versions = vec![v];
        let path = logs.join("20261006T091502Z_abc123.jsonl");
        let mut text = start_line("C:\\blenderbaseapps\\5.2.1\\blender.exe");
        text += &line("new_file", 1, "");
        text += &line("cube_deleted", 7, "");
        text += &line("hb", 60, ",\"active\":true,\"undo\":3,\"redo\":0");
        text += &line("hb", 120, ",\"active\":true,\"undo\":2,\"redo\":1");
        text += "{\"t\":\"file_sa"; // torn: Blender is writing this line right now
        std::fs::write(&path, &text).unwrap();
        let now = Utc.with_ymd_and_hms(2026, 10, 6, 9, 17, 30).unwrap();
        let alive = |pid: i64| pid == 4242;

        let mut report = ActivityImportReport::default();
        import_logs(&pool, &logs, &versions, now, &alive, &mut report).await.unwrap();
        assert_eq!(report.files_seen, 1);
        assert_eq!(report.sessions_updated, 1);
        assert_eq!(report.sessions_finished, 0);
        assert!(path.exists(), "a running session keeps its file");
        let repository = BlenderSessionRepository::new(&pool);
        let session = repository.fetch_by_id("abc123").await.unwrap().expect("the row");
        assert_eq!(session.blender_version_id.as_deref(), Some("w"));
        assert_eq!(session.version, "5.2.1");
        assert_eq!(session.series, "5.2");
        assert_eq!(session.pid, 4242);
        assert_eq!(session.started_at, T0);
        assert_eq!(session.open_seconds, 120);
        assert_eq!(session.active_seconds, 120);
        assert!(!session.is_finished);
        assert_eq!(session.import_offset as usize, text.len() - "{\"t\":\"file_sa".len());

        // Nothing new: no row is touched, the report stays quiet.
        let mut again = ActivityImportReport::default();
        import_logs(&pool, &logs, &versions, now, &alive, &mut again).await.unwrap();
        assert_eq!(again.sessions_updated, 0);

        // Blender finishes the torn line, works on, and quits cleanly.
        let mut rest = String::from("ved\",\"at\":\"2026-10-06T09:17:10+00:00\",\"up\":128}\n");
        rest += &line("hb", 180, ",\"active\":false,\"undo\":0,\"redo\":0");
        rest += &line("render_finished", 200, ",\"seconds\":45");
        rest += &line("end", 230, "");
        std::fs::write(&path, format!("{}{}", text, rest)).unwrap();
        let mut last = ActivityImportReport::default();
        import_logs(&pool, &logs, &versions, now, &alive, &mut last).await.unwrap();
        assert_eq!(last.sessions_updated, 1);
        assert_eq!(last.sessions_finished, 1);
        assert!(!path.exists(), "a finished session's file is deleted");
        let session = repository.fetch_by_id("abc123").await.unwrap().unwrap();
        assert!(session.is_finished);
        assert!(session.is_clean_exit);
        assert_eq!(session.open_seconds, 230);
        assert_eq!(session.active_seconds, 120, "the quiet heartbeat adds nothing");
        assert_eq!(session.ended_at, "2026-10-06T09:18:52+00:00");

        let counters: Vec<(String, i64)> = sqlx::query_as("SELECT kind, value FROM blender_session_counter WHERE blender_session_id = 'abc123' ORDER BY kind")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(
            counters,
            vec![
                ("cube_deleted".to_string(), 1),
                ("file_saved".to_string(), 1),
                ("new_file".to_string(), 1),
                ("redo".to_string(), 1),
                ("render_finished".to_string(), 1),
                ("render_seconds".to_string(), 45),
                ("undo".to_string(), 5),
            ]
        );
        let time = repository.fetch_version_time("").await.unwrap();
        assert_eq!(time.len(), 1);
        assert_eq!(time[0].blender_version_id.as_deref(), Some("w"));
        assert_eq!(time[0].open_seconds, 230);
        assert_eq!(time[0].sessions, 1);
        assert_eq!(time[0].last_used, T0);
        pool.close().await;
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&logs);
    }

    /// No end line, three heartbeats missed, process gone: the session is finished unclean and
    /// its hours still count. A binary nobody knows leaves the version empty and is matched later
    /// when that version turns up.
    #[tokio::test]
    async fn a_silent_session_of_a_dead_process_is_finished_unclean_and_rematched_later() {
        let dir = temp_dir("db2");
        let logs = temp_dir("logs2");
        let pool = test_pool(&dir).await;
        let path = logs.join("20261006T091502Z_abc123.jsonl");
        let mut text = start_line("E:\\portable\\blender.exe");
        text += &line("hb", 60, ",\"active\":true,\"undo\":0,\"redo\":0");
        std::fs::write(&path, &text).unwrap();
        let now = Utc.with_ymd_and_hms(2026, 10, 6, 9, 30, 0).unwrap();
        let dead = |_: i64| false;

        let mut report = ActivityImportReport::default();
        import_logs(&pool, &logs, &[], now, &dead, &mut report).await.unwrap();
        assert_eq!(report.sessions_finished, 1);
        assert!(!path.exists());
        let repository = BlenderSessionRepository::new(&pool);
        let session = repository.fetch_by_id("abc123").await.unwrap().unwrap();
        assert!(session.is_finished);
        assert!(!session.is_clean_exit, "no end line: an unclean exit");
        assert!(session.blender_version_id.is_none());
        assert_eq!(session.open_seconds, 60);
        let time = repository.fetch_version_time("").await.unwrap();
        assert_eq!(time[0].blender_version_id, None, "counted under versions no longer installed");

        let v = version("p", "E:\\portable", "E:\\portable\\blender.exe", "9e2066a", "5.2.1");
        insert_version(&pool, &v).await;
        rematch_sessions(&pool, &[v]).await.unwrap();
        let session = repository.fetch_by_id("abc123").await.unwrap().unwrap();
        assert_eq!(session.blender_version_id.as_deref(), Some("p"));
        pool.close().await;
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&logs);
    }

    #[test]
    fn series_folders_are_recognised_by_name() {
        assert!(is_series_folder_name("4.5"));
        assert!(is_series_folder_name("5.2"));
        assert!(is_series_folder_name("2.80"), "the first series with the handlers the script needs");
        assert!(!is_series_folder_name("2.79"), "too old a Python for the script");
        assert!(!is_series_folder_name("2.76"));
        assert!(!is_series_folder_name("5.2.1"));
        assert!(!is_series_folder_name("config"));
        assert!(!is_series_folder_name("5."));
        assert_eq!(
            series_folder_of_config("C:\\Users\\me\\AppData\\Roaming\\Blender Foundation\\Blender\\5.2\\config"),
            PathBuf::from("C:\\Users\\me\\AppData\\Roaming\\Blender Foundation\\Blender\\5.2")
        );
        assert_eq!(series_folder_of_config("/home/me/.config/blender/4.5"), PathBuf::from("/home/me/.config/blender/4.5"));
    }
}
