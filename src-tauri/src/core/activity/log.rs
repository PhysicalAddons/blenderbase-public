//! Reading the activity log the startup script writes: one JSON object per line, folded into
//! what a session row needs. Pure functions over bytes, so the importer can be tested without
//! Blender or a database.

use std::collections::BTreeMap;

use serde::Deserialize;

/// The script writes one `hb` line this often; an active minute counts as this many seconds.
pub const HEARTBEAT_SECONDS: i64 = 60;

/// The counter kinds a session may carry. Distances are in thousandths of a Blender unit
/// (`_mm` in a metric scene), angles in tenths of a degree (`_ddeg`), zoom in thousandths of
/// a doubling (`_mz`): integers in the table, small enough deltas to add up without loss.
pub const COUNTER_KINDS: [&str; 18] = [
    "file_opened",
    "new_file",
    "file_saved",
    "render_started",
    "render_finished",
    "render_cancelled",
    "render_seconds",
    "undo",
    "redo",
    "cube_deleted",
    "suzanne_added",
    "view_pan_mm",
    "view_orbit_ddeg",
    "view_zoom_mz",
    "object_move_mm",
    "object_turn_ddeg",
    "camera_move_mm",
    "addon_enabled",
];

/// The log's first line.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LogStart {
    pub session: String,
    pub pid: i64,
    pub version: String,
    pub series: String,
    pub build_hash: String,
    pub binary: String,
    pub script: i64,
    pub at: String,
}

/// Everything a stretch of log lines adds up to.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LogFold {
    pub start: Option<LogStart>,
    /// `at` of the last line read; empty when none carried one.
    pub last_at: String,
    /// The largest `up` seen: seconds since the session started, on the monotonic clock.
    pub last_up: i64,
    /// 60 per heartbeat that reported activity.
    pub active_seconds: i64,
    pub counters: BTreeMap<String, i64>,
    /// An `end` line was read.
    pub clean_exit: bool,
    pub lines: usize,
    /// Lines that were not JSON or not a kind this build knows.
    pub skipped: usize,
    /// Bytes consumed: up to and including the last newline. A partial last line (Blender
    /// crashed mid-write, or is writing right now) is left for the next read.
    pub consumed: usize,
}

#[derive(Deserialize)]
struct RawLine {
    t: String,
    at: Option<String>,
    up: Option<i64>,
    session: Option<String>,
    pid: Option<i64>,
    version: Option<String>,
    version_tuple: Option<Vec<i64>>,
    build_hash: Option<String>,
    binary: Option<String>,
    script: Option<i64>,
    active: Option<bool>,
    undo: Option<i64>,
    redo: Option<i64>,
    seconds: Option<i64>,
    count: Option<i64>,
    // Script version 2: what the viewport and the objects did since the last heartbeat.
    pan: Option<f64>,
    orbit: Option<f64>,
    zoom: Option<f64>,
    moved: Option<f64>,
    turned: Option<f64>,
    cam_moved: Option<f64>,
    addons: Option<i64>,
}

/// A float from the log as a whole number of thousandths or tenths, so it adds up in an integer column.
fn scaled(value: Option<f64>, scale: f64) -> i64 {
    match value {
        Some(v) if v.is_finite() && v > 0.0 => (v * scale).round() as i64,
        _ => 0,
    }
}

/// Folds every complete line in `bytes`.
pub fn fold_log(bytes: &[u8]) -> LogFold {
    let mut fold = LogFold::default();
    let mut pos = 0;
    while let Some(newline) = bytes[pos..].iter().position(|b| *b == b'\n') {
        let line = &bytes[pos..pos + newline];
        pos += newline + 1;
        fold.consumed = pos;
        let text = String::from_utf8_lossy(line);
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        let raw: RawLine = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => {
                fold.skipped += 1;
                continue;
            }
        };
        fold.lines += 1;
        if let Some(at) = &raw.at {
            if !at.is_empty() {
                fold.last_at = at.clone();
            }
        }
        if let Some(up) = raw.up {
            fold.last_up = fold.last_up.max(up);
        }
        match raw.t.as_str() {
            "start" => {
                fold.start = Some(LogStart {
                    session: raw.session.clone().unwrap_or_default(),
                    pid: raw.pid.unwrap_or(0),
                    version: raw.version.clone().unwrap_or_default(),
                    series: series_of_version(&raw.version_tuple, &raw.version),
                    build_hash: raw.build_hash.clone().unwrap_or_default(),
                    binary: raw.binary.clone().unwrap_or_default(),
                    script: raw.script.unwrap_or(0),
                    at: raw.at.clone().unwrap_or_default(),
                });
            }
            "hb" => {
                if raw.active == Some(true) {
                    fold.active_seconds += HEARTBEAT_SECONDS;
                }
                bump(&mut fold, "undo", raw.undo.unwrap_or(0));
                bump(&mut fold, "redo", raw.redo.unwrap_or(0));
                bump(&mut fold, "view_pan_mm", scaled(raw.pan, 1000.0));
                bump(&mut fold, "view_orbit_ddeg", scaled(raw.orbit, 10.0));
                bump(&mut fold, "view_zoom_mz", scaled(raw.zoom, 1000.0));
                bump(&mut fold, "object_move_mm", scaled(raw.moved, 1000.0));
                bump(&mut fold, "object_turn_ddeg", scaled(raw.turned, 10.0));
                bump(&mut fold, "camera_move_mm", scaled(raw.cam_moved, 1000.0));
                bump(&mut fold, "addon_enabled", raw.addons.unwrap_or(0).max(0));
            }
            "end" => fold.clean_exit = true,
            "render_finished" | "render_cancelled" => {
                bump(&mut fold, &raw.t, 1);
                bump(&mut fold, "render_seconds", raw.seconds.unwrap_or(0));
            }
            "suzanne_added" => bump(&mut fold, "suzanne_added", raw.count.unwrap_or(1)),
            "file_opened" | "new_file" | "file_saved" | "render_started" | "cube_deleted" => {
                bump(&mut fold, &raw.t, 1)
            }
            _ => fold.skipped += 1,
        }
    }
    fold
}

fn bump(fold: &mut LogFold, kind: &str, by: i64) {
    if by == 0 {
        return;
    }
    *fold.counters.entry(kind.to_string()).or_insert(0) += by;
}

/// "5.2" from `[5, 2, 1]`, else from "5.2.1"; empty when neither is usable.
pub fn series_of_version(version_tuple: &Option<Vec<i64>>, version: &Option<String>) -> String {
    if let Some(t) = version_tuple {
        if t.len() >= 2 {
            return format!("{}.{}", t[0], t[1]);
        }
    }
    let Some(v) = version else {
        return String::new();
    };
    let mut parts = v.trim().split('.');
    match (parts.next(), parts.next()) {
        (Some(a), Some(b)) if !a.is_empty() && !b.is_empty() => format!("{}.{}", a, b),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = concat!(
        "{\"t\":\"start\",\"at\":\"2026-10-06T09:15:02+00:00\",\"up\":0,\"session\":\"3f9c2a7e\",\"pid\":18240,\"version\":\"5.2.1\",\"version_tuple\":[5,2,1],\"build_hash\":\"9e2066a\",\"binary\":\"D:\\\\Blender\\\\5.2.1\\\\blender.exe\",\"script\":1}\n",
        "{\"t\":\"new_file\",\"at\":\"2026-10-06T09:15:03+00:00\",\"up\":1}\n",
        "{\"t\":\"cube_deleted\",\"at\":\"2026-10-06T09:15:09+00:00\",\"up\":7}\n",
        "{\"t\":\"hb\",\"at\":\"2026-10-06T09:16:02+00:00\",\"up\":60,\"active\":true,\"undo\":3,\"redo\":0}\n",
        "{\"t\":\"render_started\",\"at\":\"2026-10-06T09:40:11+00:00\",\"up\":1509}\n",
        "{\"t\":\"render_finished\",\"at\":\"2026-10-06T09:41:40+00:00\",\"up\":1598,\"seconds\":89}\n",
        "{\"t\":\"suzanne_added\",\"at\":\"2026-10-06T09:50:00+00:00\",\"up\":2098,\"count\":2}\n",
        "{\"t\":\"hb\",\"at\":\"2026-10-06T10:02:02+00:00\",\"up\":2820,\"active\":false,\"undo\":0,\"redo\":1}\n",
        "{\"t\":\"end\",\"at\":\"2026-10-06T10:02:31+00:00\",\"up\":2849}\n",
    );

    #[test]
    fn folds_a_whole_session() {
        let fold = fold_log(SAMPLE.as_bytes());
        let start = fold.start.as_ref().expect("the start line");
        assert_eq!(start.session, "3f9c2a7e");
        assert_eq!(start.pid, 18240);
        assert_eq!(start.version, "5.2.1");
        assert_eq!(start.series, "5.2");
        assert_eq!(start.build_hash, "9e2066a");
        assert_eq!(start.binary, "D:\\Blender\\5.2.1\\blender.exe");
        assert_eq!(start.at, "2026-10-06T09:15:02+00:00");
        assert_eq!(fold.last_at, "2026-10-06T10:02:31+00:00");
        assert_eq!(fold.last_up, 2849);
        assert_eq!(fold.active_seconds, 60, "one active heartbeat");
        assert!(fold.clean_exit);
        assert_eq!(fold.lines, 9);
        assert_eq!(fold.skipped, 0);
        assert_eq!(fold.consumed, SAMPLE.len());
        assert_eq!(fold.counters.get("new_file"), Some(&1));
        assert_eq!(fold.counters.get("cube_deleted"), Some(&1));
        assert_eq!(fold.counters.get("render_started"), Some(&1));
        assert_eq!(fold.counters.get("render_finished"), Some(&1));
        assert_eq!(fold.counters.get("render_seconds"), Some(&89));
        assert_eq!(fold.counters.get("suzanne_added"), Some(&2));
        assert_eq!(fold.counters.get("undo"), Some(&3));
        assert_eq!(fold.counters.get("redo"), Some(&1));
        assert!(fold.counters.get("file_saved").is_none(), "zero counters are not reported");
    }

    /// A line still being written has no newline yet: it waits, and the next read picks it
    /// up from the consumed offset without counting the earlier lines again.
    #[test]
    fn leaves_a_partial_last_line_for_the_next_read() {
        let cut = SAMPLE.len() - 20;
        let first = fold_log(&SAMPLE.as_bytes()[..cut]);
        assert!(!first.clean_exit);
        assert!(first.consumed < cut, "the torn end line is not consumed");
        assert_eq!(first.last_up, 2820);
        let second = fold_log(&SAMPLE.as_bytes()[first.consumed..]);
        assert!(second.clean_exit);
        assert_eq!(second.lines, 1);
        assert!(second.start.is_none());
        assert_eq!(second.last_up, 2849);
        assert!(second.counters.is_empty());
    }

    /// Version 2 heartbeats carry what the viewport and the objects did; the fold scales them
    /// into whole thousandths and tenths so an integer column can add them up.
    #[test]
    fn heartbeats_carry_viewport_and_object_motion() {
        let text = concat!(
            "{\"t\":\"hb\",\"at\":\"2026-10-06T09:16:02+00:00\",\"up\":60,\"active\":true,\"undo\":0,\"redo\":0,\"pan\":12.3456,\"orbit\":270.04,\"zoom\":1.5,\"moved\":0.0004,\"turned\":90.0,\"cam_moved\":2.5,\"addons\":1}\n",
            "{\"t\":\"hb\",\"at\":\"2026-10-06T09:17:02+00:00\",\"up\":120,\"active\":false,\"undo\":0,\"redo\":0,\"pan\":0.0,\"orbit\":-5.0,\"addons\":0}\n",
        );
        let fold = fold_log(text.as_bytes());
        assert_eq!(fold.counters.get("view_pan_mm"), Some(&12346));
        assert_eq!(fold.counters.get("view_orbit_ddeg"), Some(&2700), "a negative value is ignored, not subtracted");
        assert_eq!(fold.counters.get("view_zoom_mz"), Some(&1500));
        assert!(fold.counters.get("object_move_mm").is_none(), "under half a thousandth rounds to nothing");
        assert_eq!(fold.counters.get("object_turn_ddeg"), Some(&900));
        assert_eq!(fold.counters.get("camera_move_mm"), Some(&2500));
        assert_eq!(fold.counters.get("addon_enabled"), Some(&1));
        // A version 1 heartbeat without the fields still folds.
        let old = fold_log(b"{\"t\":\"hb\",\"at\":\"2026-10-06T09:16:02+00:00\",\"up\":60,\"active\":true,\"undo\":2,\"redo\":0}\n");
        assert_eq!(old.counters.get("undo"), Some(&2));
        assert!(old.counters.get("view_pan_mm").is_none());
    }

    #[test]
    fn unknown_or_broken_lines_are_skipped_but_consumed() {
        let text = "not json at all\n{\"t\":\"teleport\",\"at\":\"2026-10-06T09:15:03+00:00\",\"up\":5}\n{\"t\":\"file_saved\",\"up\":9}\n";
        let fold = fold_log(text.as_bytes());
        assert_eq!(fold.skipped, 2);
        assert_eq!(fold.lines, 2, "the unknown kind is still a parsed line");
        assert_eq!(fold.consumed, text.len());
        assert_eq!(fold.last_up, 9);
        assert_eq!(fold.last_at, "2026-10-06T09:15:03+00:00", "a line without `at` keeps the last one");
        assert_eq!(fold.counters.get("file_saved"), Some(&1));
    }

    #[test]
    fn series_comes_from_the_tuple_then_the_string() {
        assert_eq!(series_of_version(&Some(vec![5, 2, 1]), &None), "5.2");
        assert_eq!(series_of_version(&Some(vec![4]), &Some("4.5.1".into())), "4.5");
        assert_eq!(series_of_version(&None, &Some("5.3.0 Alpha".into())), "5.3");
        assert_eq!(series_of_version(&None, &Some("odd".into())), "");
        assert_eq!(series_of_version(&None, &None), "");
    }
}
