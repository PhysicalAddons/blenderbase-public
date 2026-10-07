//! The achievement catalogue and its evaluation. A rule is a metric, a threshold and a scope;
//! the catalogue is one JSON file bundled with the build, so adding a badge is adding an entry.
//! The progress a locked row shows comes from the same metric the rule reads, so the two never
//! disagree.

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Local, SecondsFormat, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::{
    core::parse_log_time,
    database::{
        AchievementUnlockRepository, ActivityCounterRepository, BlenderSessionRepository,
        InstalledBuild,
    },
};

const CATALOGUE_JSON: &str = include_str!("achievements.json");

/// One rule of the catalogue.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AchievementDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    /// What is measured: `open_hours`, `cube_deleted`, `sessions`, ... (see [`metric_value`]).
    pub metric: String,
    pub threshold: f64,
    /// `total` over everything, `series_max` the largest single series, `version_max` the
    /// largest single installed version.
    #[serde(default = "total_scope")]
    pub scope: String,
    /// A name the frontend maps to an icon; no UI detail lives in the catalogue.
    pub icon: String,
}

fn total_scope() -> String {
    String::from("total")
}

/// A rule with where the user stands on it.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AchievementStatus {
    pub id: String,
    pub name: String,
    pub description: String,
    pub metric: String,
    pub threshold: f64,
    pub scope: String,
    pub icon: String,
    /// The metric as it stands, floored to whole units (hours or counts).
    pub value: i64,
    pub unlocked_at: Option<String>,
    pub is_seen: bool,
}

/// The bundled catalogue; empty, with a line in the log, should the JSON ever be broken.
pub fn achievement_catalogue() -> Vec<AchievementDefinition> {
    match serde_json::from_str::<Vec<AchievementDefinition>>(CATALOGUE_JSON) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("The bundled achievements.json is not valid: {}", e);
            Vec::new()
        }
    }
}

/// Everything the rules read, fetched once per evaluation.
#[derive(Default, Debug, Clone)]
pub struct AchievementMetrics {
    pub open_seconds: i64,
    pub active_seconds: i64,
    pub longest_session_seconds: i64,
    pub finished_sessions: i64,
    pub unclean_exits: i64,
    pub alpha_sessions: i64,
    pub series_open_max_seconds: i64,
    pub version_open_max_seconds: i64,
    pub counters: BTreeMap<String, i64>,
    pub versions_in_week: i64,
    pub night_sessions: i64,
    /// `ended_at` of the newest finished session, else now: the moment a crossed rule is dated.
    pub latest_end: String,
    /// What Blenderbase itself counted: addons installed, setups shared and applied, Blender
    /// versions installed (`activity_counter`).
    pub app_counters: BTreeMap<String, i64>,
    /// Different addons and extensions installed right now, each counted once across series.
    pub addons_installed: i64,
    pub extensions_installed: i64,
    /// Blender versions installed right now, and how many of LTS, Stable, Beta and Alpha are among them.
    pub blender_versions_installed: i64,
    pub blender_variants_installed: i64,
}

/// Series Blender maintains as long-term support releases; mirrors the frontend's list.
const LTS_SERIES: [&str; 6] = ["2.83", "2.93", "3.3", "3.6", "4.2", "4.5"];
/// Words that land in a release archive's variant slot instead of a cycle.
const PLATFORM_WORDS: [&str; 9] = ["windows", "win", "darwin", "macos", "mac", "linux", "x64", "x86", "arm64"];

/// `lts`, `stable`, `candidate`, `beta`, `alpha` or empty: the same reading of a build the
/// Blender column's tags use.
pub fn build_variant(release_cycle: Option<&str>, risk_id: Option<&str>, series: Option<&str>) -> &'static str {
    let raw = release_cycle
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| risk_id.map(str::trim).filter(|s| !s.is_empty()))
        .unwrap_or("");
    if raw.is_empty() {
        return "";
    }
    let value = raw.to_lowercase();
    if PLATFORM_WORDS.contains(&value.as_str()) {
        // Release archives carry only the platform in their name: a release build.
        return match series.map(str::trim) {
            Some(s) if !s.is_empty() => {
                if LTS_SERIES.contains(&s) {
                    "lts"
                } else {
                    "stable"
                }
            }
            _ => "",
        };
    }
    match value.as_str() {
        "lts" => "lts",
        "stable" | "release" => "stable",
        "candidate" | "rc" => "candidate",
        "beta" => "beta",
        "alpha" => "alpha",
        _ => "",
    }
}

/// How many of LTS, Stable, Beta and Alpha are installed at once: Full Spectrum wants all four.
pub fn count_build_variants(builds: &[InstalledBuild]) -> i64 {
    ["lts", "stable", "beta", "alpha"]
        .iter()
        .filter(|wanted| {
            builds.iter().any(|b| {
                build_variant(b.release_cycle.as_deref(), b.risk_id.as_deref(), b.series.as_deref()) == **wanted
            })
        })
        .count() as i64
}

pub async fn load_achievement_metrics(
    pool: &SqlitePool,
    now: DateTime<Utc>,
) -> Result<AchievementMetrics, String> {
    let sessions = BlenderSessionRepository::new(pool);
    let failed = |e: sqlx::Error| format!("Failed load_achievement_metrics: {:?}", e);
    let totals = sessions.fetch_totals("").await.map_err(failed)?;
    let week_ago = (now - Duration::days(7)).to_rfc3339_opts(SecondsFormat::Secs, false);
    let spans = sessions.fetch_spans().await.map_err(failed)?;
    let app = ActivityCounterRepository::new(pool);
    let builds = app.fetch_installed_builds().await.map_err(failed)?;
    Ok(AchievementMetrics {
        app_counters: app.fetch_all().await.map_err(failed)?.into_iter().collect(),
        addons_installed: app.count_distinct_addons("addon").await.map_err(failed)?,
        extensions_installed: app.count_distinct_addons("extension").await.map_err(failed)?,
        blender_versions_installed: builds.len() as i64,
        blender_variants_installed: count_build_variants(&builds),
        open_seconds: totals.open_seconds,
        active_seconds: totals.active_seconds,
        longest_session_seconds: totals.longest_session_seconds,
        finished_sessions: totals.finished_sessions,
        unclean_exits: totals.unclean_exits,
        alpha_sessions: totals.alpha_sessions,
        series_open_max_seconds: sessions.fetch_series_open_max().await.map_err(failed)?,
        version_open_max_seconds: sessions.fetch_version_open_max().await.map_err(failed)?,
        counters: sessions.fetch_counter_totals("").await.map_err(failed)?.into_iter().collect(),
        versions_in_week: sessions.fetch_distinct_versions_since(&week_ago).await.map_err(failed)?,
        night_sessions: spans
            .iter()
            .filter(|(start, end)| spans_hour_in(&Local, start, end, 2))
            .count() as i64,
        latest_end: if totals.latest_end.is_empty() {
            now.to_rfc3339_opts(SecondsFormat::Secs, false)
        } else {
            totals.latest_end
        },
    })
}

/// Whether a session running from `started_at` to `ended_at` was open at `hour` o'clock (local
/// time in `tz`) on any day. A session longer than two days has seen every hour.
pub fn spans_hour_in<Tz: TimeZone>(tz: &Tz, started_at: &str, ended_at: &str, hour: u32) -> bool {
    let (Some(start), Some(end)) = (parse_log_time(started_at), parse_log_time(ended_at)) else {
        return false;
    };
    if end <= start {
        return false;
    }
    if end - start >= Duration::hours(48) {
        return true;
    }
    let start_local = start.with_timezone(tz);
    let end_local = end.with_timezone(tz);
    let mut day = start_local.date_naive();
    for _ in 0..3 {
        if let Some(moment) = day
            .and_hms_opt(hour, 0, 0)
            .and_then(|naive| tz.from_local_datetime(&naive).earliest())
        {
            if moment >= start_local && moment <= end_local {
                return true;
            }
        }
        day = match day.succ_opt() {
            Some(next) => next,
            None => return false,
        };
    }
    false
}

/// What a rule's metric is worth right now, in the rule's units.
pub fn metric_value(m: &AchievementMetrics, metric: &str, scope: &str) -> f64 {
    let hours = |seconds: i64| seconds as f64 / 3600.0;
    let counter = |kind: &str| *m.counters.get(kind).unwrap_or(&0) as f64;
    let app = |kind: &str| *m.app_counters.get(kind).unwrap_or(&0) as f64;
    match metric {
        "open_hours" => match scope {
            "series_max" => hours(m.series_open_max_seconds),
            "version_max" => hours(m.version_open_max_seconds),
            _ => hours(m.open_seconds),
        },
        "active_hours" => hours(m.active_seconds),
        "longest_session_hours" => hours(m.longest_session_seconds),
        "sessions" => m.finished_sessions as f64,
        "render_hours" => hours(*m.counters.get("render_seconds").unwrap_or(&0)),
        "versions_in_week" => m.versions_in_week as f64,
        "night_sessions" => m.night_sessions as f64,
        "alpha_sessions" => m.alpha_sessions as f64,
        "unclean_exits" => m.unclean_exits as f64,
        // The viewport and the objects: the log carries thousandths of a unit and tenths of a degree.
        "view_orbit_turns" => counter("view_orbit_ddeg") / 3600.0,
        "view_pan_km" => counter("view_pan_mm") / 1_000_000.0,
        "view_zoom_doublings" => counter("view_zoom_mz") / 1000.0,
        "object_move_km" => counter("object_move_mm") / 1_000_000.0,
        "object_turns" => counter("object_turn_ddeg") / 3600.0,
        "camera_move_km" => counter("camera_move_mm") / 1_000_000.0,
        // Addons: enabled inside Blender plus installed through Blenderbase, and what is there now.
        "addon_installs" => counter("addon_enabled") + app("app_addon_installs"),
        "app_addon_installs" => app("app_addon_installs"),
        "addons_installed" => m.addons_installed as f64,
        "extensions_installed" => m.extensions_installed as f64,
        // Sync and Blender installs, counted by Blenderbase itself.
        "setup_shared" => app("app_setup_shared"),
        "setup_applied" => app("app_setup_applied"),
        "blender_installs" => app("app_blender_installs"),
        "blender_versions_installed" => m.blender_versions_installed as f64,
        "blender_variants_installed" => m.blender_variants_installed as f64,
        other => counter(other),
    }
}

/// Compares every rule not yet unlocked with the totals and records the ones crossed. Returns
/// those, for the status line.
pub async fn evaluate_achievements(
    pool: &SqlitePool,
    now: DateTime<Utc>,
) -> Result<Vec<AchievementDefinition>, String> {
    let catalogue = achievement_catalogue();
    if catalogue.is_empty() {
        return Ok(Vec::new());
    }
    let unlocks = AchievementUnlockRepository::new(pool);
    let have: Vec<String> = unlocks
        .fetch_all()
        .await
        .map_err(|e| format!("Failed evaluate_achievements: {:?}", e))?
        .into_iter()
        .map(|u| u.achievement_id)
        .collect();
    let metrics = load_achievement_metrics(pool, now).await?;
    let mut fresh = Vec::new();
    for definition in catalogue {
        if have.contains(&definition.id) {
            continue;
        }
        if metric_value(&metrics, &definition.metric, &definition.scope) >= definition.threshold {
            unlocks
                .insert(&definition.id, &metrics.latest_end, None)
                .await
                .map_err(|e| format!("Failed evaluate_achievements: {:?}", e))?;
            fresh.push(definition);
        }
    }
    Ok(fresh)
}

/// The whole catalogue with progress and unlock dates, in catalogue order.
pub async fn fetch_achievements(
    pool: &SqlitePool,
    now: DateTime<Utc>,
) -> Result<Vec<AchievementStatus>, String> {
    let metrics = load_achievement_metrics(pool, now).await?;
    let unlocks = AchievementUnlockRepository::new(pool)
        .fetch_all()
        .await
        .map_err(|e| format!("Failed fetch_achievements: {:?}", e))?;
    Ok(achievement_catalogue()
        .into_iter()
        .map(|d| {
            let unlock = unlocks.iter().find(|u| u.achievement_id == d.id);
            AchievementStatus {
                value: metric_value(&metrics, &d.metric, &d.scope).floor() as i64,
                unlocked_at: unlock.map(|u| u.unlocked_at.clone()),
                is_seen: unlock.map(|u| u.is_seen).unwrap_or(false),
                id: d.id,
                name: d.name,
                description: d.description,
                metric: d.metric,
                threshold: d.threshold,
                scope: d.scope,
                icon: d.icon,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    #[test]
    fn the_catalogue_parses_with_unique_ids_and_known_metrics() {
        let catalogue = achievement_catalogue();
        assert!(catalogue.len() >= 33, "the starting set plus the viewport, addon, sync and install badges");
        let mut ids: Vec<&str> = catalogue.iter().map(|d| d.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), catalogue.len(), "ids are unique");
        let empty = AchievementMetrics::default();
        for d in &catalogue {
            assert!(d.threshold > 0.0, "{} has a threshold", d.id);
            assert!(!d.icon.is_empty(), "{} has an icon", d.id);
            assert_eq!(metric_value(&empty, &d.metric, &d.scope), 0.0, "{} reads zero on an empty table", d.id);
        }
    }

    #[test]
    fn metrics_map_to_their_units() {
        let mut m = AchievementMetrics::default();
        m.open_seconds = 7200;
        m.series_open_max_seconds = 3600;
        m.counters.insert("cube_deleted".into(), 5);
        m.counters.insert("render_seconds".into(), 5400);
        m.finished_sessions = 3;
        assert_eq!(metric_value(&m, "open_hours", "total"), 2.0);
        assert_eq!(metric_value(&m, "open_hours", "series_max"), 1.0);
        assert_eq!(metric_value(&m, "open_hours", "version_max"), 0.0);
        assert_eq!(metric_value(&m, "cube_deleted", "total"), 5.0);
        assert_eq!(metric_value(&m, "render_hours", "total"), 1.5);
        assert_eq!(metric_value(&m, "sessions", "total"), 3.0);
        assert_eq!(metric_value(&m, "nothing_like_this", "total"), 0.0);

        m.counters.insert("view_orbit_ddeg".into(), 7200);
        m.counters.insert("view_pan_mm".into(), 2_500_000);
        m.counters.insert("view_zoom_mz".into(), 1500);
        m.counters.insert("object_move_mm".into(), 10_000_000);
        m.counters.insert("object_turn_ddeg".into(), 3600);
        m.counters.insert("camera_move_mm".into(), 500_000);
        m.counters.insert("addon_enabled".into(), 3);
        m.app_counters.insert("app_addon_installs".into(), 4);
        m.app_counters.insert("app_setup_shared".into(), 2);
        m.app_counters.insert("app_blender_installs".into(), 6);
        m.addons_installed = 12;
        m.extensions_installed = 5;
        m.blender_versions_installed = 20;
        m.blender_variants_installed = 3;
        assert_eq!(metric_value(&m, "view_orbit_turns", "total"), 2.0);
        assert_eq!(metric_value(&m, "view_pan_km", "total"), 2.5);
        assert_eq!(metric_value(&m, "view_zoom_doublings", "total"), 1.5);
        assert_eq!(metric_value(&m, "object_move_km", "total"), 10.0);
        assert_eq!(metric_value(&m, "object_turns", "total"), 1.0);
        assert_eq!(metric_value(&m, "camera_move_km", "total"), 0.5);
        assert_eq!(metric_value(&m, "addon_installs", "total"), 7.0, "enabled in Blender plus installed through the app");
        assert_eq!(metric_value(&m, "app_addon_installs", "total"), 4.0);
        assert_eq!(metric_value(&m, "setup_shared", "total"), 2.0);
        assert_eq!(metric_value(&m, "setup_applied", "total"), 0.0);
        assert_eq!(metric_value(&m, "blender_installs", "total"), 6.0);
        assert_eq!(metric_value(&m, "addons_installed", "total"), 12.0);
        assert_eq!(metric_value(&m, "extensions_installed", "total"), 5.0);
        assert_eq!(metric_value(&m, "blender_versions_installed", "total"), 20.0);
        assert_eq!(metric_value(&m, "blender_variants_installed", "total"), 3.0);
    }

    #[test]
    fn build_variants_read_like_the_blender_column_tags() {
        assert_eq!(build_variant(Some("windows"), None, Some("4.5")), "lts");
        assert_eq!(build_variant(Some("windows"), None, Some("5.1")), "stable");
        assert_eq!(build_variant(None, Some("x64"), Some("5.2")), "stable", "the risk id is the fallback");
        assert_eq!(build_variant(Some("windows"), None, None), "", "a platform word without a series says nothing");
        assert_eq!(build_variant(Some("Alpha"), None, Some("5.3")), "alpha");
        assert_eq!(build_variant(Some("beta"), None, Some("5.2")), "beta");
        assert_eq!(build_variant(Some("rc"), None, Some("5.2")), "candidate");
        assert_eq!(build_variant(Some("release"), None, Some("5.2")), "stable");
        assert_eq!(build_variant(Some("lts"), None, Some("5.2")), "lts");
        assert_eq!(build_variant(Some("experimental"), None, Some("5.2")), "");
        assert_eq!(build_variant(None, None, Some("5.2")), "");
        let build = |cycle: &str, series: &str| InstalledBuild { release_cycle: Some(cycle.into()), risk_id: None, series: Some(series.into()) };
        assert_eq!(count_build_variants(&[build("windows", "4.5"), build("windows", "5.1"), build("alpha", "5.3")]), 3);
        assert_eq!(count_build_variants(&[build("windows", "4.5"), build("windows", "5.1"), build("alpha", "5.3"), build("beta", "5.2"), build("rc", "5.2")]), 4, "a candidate adds nothing beyond the four");
        assert_eq!(count_build_variants(&[]), 0);
    }

    /// The badges Blenderbase counts by itself: an addon installed through the app, a setup
    /// shared, a Blender version installed, and the four build kinds installed at once.
    #[tokio::test]
    async fn app_side_counters_and_installed_builds_unlock_their_badges() {
        let (pool, dir) = test_pool().await;
        let now = Utc::now();
        let app = ActivityCounterRepository::new(&pool);
        app.add("app_addon_installs", 1).await.unwrap();
        app.add("app_setup_shared", 1).await.unwrap();
        app.add("app_blender_installs", 1).await.unwrap();
        app.add("app_blender_installs", 2).await.unwrap();
        for (id, cycle, series) in [("b1", "windows", "4.5"), ("b2", "windows", "5.1"), ("b3", "beta", "5.2"), ("b4", "alpha", "5.3")] {
            sqlx::query("INSERT INTO blender_version (id, release_cycle, series, file_size, installation_directory_path, blender_installation_location_id, download_status_type_id) VALUES (?, ?, ?, 0, 'x', 'loc', 4)")
                .bind(id)
                .bind(cycle)
                .bind(series)
                .execute(&pool)
                .await
                .unwrap();
        }
        // A pending download is not installed.
        sqlx::query("INSERT INTO blender_version (id, release_cycle, series, file_size, installation_directory_path, blender_installation_location_id, download_status_type_id) VALUES ('b5', 'rc', '5.2', 0, 'x', 'loc', 1)")
            .execute(&pool)
            .await
            .unwrap();
        for i in 0..3 {
            sqlx::query("INSERT INTO addon (id, main_python_file_path, installation_directory, variant_type, functional_name, parent_blender_version_id) VALUES (?, ?, 'd', 'addon', ?, 'b1')")
                .bind(format!("a{}", i))
                .bind(format!("p{}", i))
                .bind(if i == 2 { "shared_tool".to_string() } else { format!("tool_{}", i) })
                .execute(&pool)
                .await
                .unwrap();
        }
        sqlx::query("INSERT INTO addon (id, main_python_file_path, installation_directory, variant_type, functional_name, parent_blender_version_id) VALUES ('a9', 'p9', 'd', 'addon', 'shared_tool', 'b2')")
            .execute(&pool)
            .await
            .unwrap();

        let metrics = load_achievement_metrics(&pool, now).await.unwrap();
        assert_eq!(metrics.app_counters.get("app_blender_installs"), Some(&3));
        assert_eq!(metrics.blender_versions_installed, 4);
        assert_eq!(metrics.blender_variants_installed, 4);
        assert_eq!(metrics.addons_installed, 3, "the same addon in two series counts once");
        assert_eq!(metrics.extensions_installed, 0);

        let fresh = evaluate_achievements(&pool, now).await.unwrap();
        let mut ids: Vec<&str> = fresh.iter().map(|d| d.id.as_str()).collect();
        ids.sort();
        assert_eq!(ids, vec!["first_download", "full_spectrum", "plugged_in", "well_connected"]);
        pool.close().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn night_shift_is_judged_in_local_time() {
        let plus_one = FixedOffset::east_opt(3600).unwrap();
        // 00:30 to 03:00 local (+01:00) = 23:30 to 02:00 UTC: includes 02:00 local.
        assert!(spans_hour_in(&plus_one, "2026-10-05T23:30:00+00:00", "2026-10-06T02:00:00+00:00", 2));
        // 22:00 to 23:30 local: no.
        assert!(!spans_hour_in(&plus_one, "2026-10-05T21:00:00+00:00", "2026-10-05T22:30:00+00:00", 2));
        // The same UTC span read in UTC: 23:30 to 02:00 UTC includes 02:00 UTC exactly at the end.
        assert!(spans_hour_in(&Utc, "2026-10-05T23:30:00+00:00", "2026-10-06T02:00:00+00:00", 2));
        // In UTC-5 that span is 18:30 to 21:00: no.
        let minus_five = FixedOffset::west_opt(5 * 3600).unwrap();
        assert!(!spans_hour_in(&minus_five, "2026-10-05T23:30:00+00:00", "2026-10-06T02:00:00+00:00", 2));
        // Three days straight: yes whatever the zone.
        assert!(spans_hour_in(&minus_five, "2026-10-01T12:00:00+00:00", "2026-10-04T12:00:00+00:00", 2));
        assert!(!spans_hour_in(&Utc, "garbage", "2026-10-04T12:00:00+00:00", 2));
    }

    async fn test_pool() -> (SqlitePool, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("blenderbase-achievements-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let url = format!("sqlite://{}", dir.join("t.db").to_string_lossy());
        let options = SqliteConnectOptions::from_str(&url)
            .unwrap()
            .create_if_missing(true)
            .foreign_keys(false);
        let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        (pool, dir)
    }

    async fn insert_session(pool: &SqlitePool, id: &str, version: &str, series: &str, started: &str, ended: &str, open: i64, clean: bool) {
        sqlx::query(
            "INSERT INTO blender_session (id, blender_version_id, version, series, build_hash, binary_path, pid, started_at, ended_at, open_seconds, active_seconds, is_finished, is_clean_exit, log_file_name, import_offset)
             VALUES (?, ?, ?, ?, '', 'x', 1, ?, ?, ?, ?, 1, ?, 'f', 0)",
        )
        .bind(id)
        .bind(format!("v-{}", version))
        .bind(version)
        .bind(series)
        .bind(started)
        .bind(ended)
        .bind(open)
        .bind(open / 2)
        .bind(clean)
        .execute(pool)
        .await
        .unwrap();
    }

    /// Rules crossed are recorded once, dated by the newest session, and show up with their
    /// progress; a second evaluation finds nothing new; marking seen clears the badge.
    #[tokio::test]
    async fn rules_unlock_once_and_report_progress() {
        let (pool, dir) = test_pool().await;
        let now = Utc::now();
        insert_session(&pool, "s1", "5.2.1 LTS", "5.2", "2026-10-05T18:00:00+00:00", "2026-10-05T20:03:05+00:00", 7385, true).await;
        insert_session(&pool, "s2", "5.3.0 Alpha", "5.3", "2026-10-06T08:00:00+00:00", "2026-10-06T08:35:00+00:00", 2100, false).await;
        sqlx::query("INSERT INTO blender_session_counter (blender_session_id, kind, value) VALUES ('s1', 'cube_deleted', 1), ('s1', 'render_seconds', 150), ('s2', 'undo', 12)")
            .execute(&pool)
            .await
            .unwrap();

        let fresh = evaluate_achievements(&pool, now).await.unwrap();
        let mut ids: Vec<&str> = fresh.iter().map(|d| d.id.as_str()).collect();
        ids.sort();
        assert_eq!(ids, vec!["crash_survivor", "cube_curious", "early_adopter", "first_steps"]);
        assert!(evaluate_achievements(&pool, now).await.unwrap().is_empty(), "nothing new the second time");

        let statuses = fetch_achievements(&pool, now).await.unwrap();
        let by_id = |id: &str| statuses.iter().find(|s| s.id == id).unwrap();
        assert_eq!(by_id("first_steps").unlocked_at.as_deref(), Some("2026-10-06T08:35:00+00:00"), "dated by the newest finished session");
        assert!(!by_id("first_steps").is_seen);
        assert_eq!(by_id("centurion").value, 2, "two whole hours so far");
        assert!(by_id("centurion").unlocked_at.is_none());
        assert_eq!(by_id("cube_slayer").value, 1);
        assert_eq!(by_id("patient").value, 0, "150 render seconds floor to zero hours");
        assert_eq!(by_id("undo_champion").value, 12);

        let unlocks = AchievementUnlockRepository::new(&pool);
        assert_eq!(unlocks.count_unseen().await.unwrap(), 4);
        unlocks.mark_all_seen().await.unwrap();
        assert_eq!(unlocks.count_unseen().await.unwrap(), 0);
        assert!(fetch_achievements(&pool, now).await.unwrap().iter().filter(|s| s.unlocked_at.is_some()).all(|s| s.is_seen));
        pool.close().await;
        let _ = std::fs::remove_dir_all(&dir);
    }
}
