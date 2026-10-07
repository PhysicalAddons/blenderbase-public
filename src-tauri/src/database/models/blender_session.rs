use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

/// One Blender run, as imported from the activity log the startup script writes. The version
/// text and series are kept on the row so the hours survive an uninstall; only the link to
/// `blender_version` is cleared then.
#[derive(Default, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct BlenderSession {
    pub id: String,
    pub blender_version_id: Option<String>,
    pub version: String,
    pub series: String,
    pub build_hash: String,
    pub binary_path: String,
    pub pid: i64,
    pub started_at: String,
    pub ended_at: String,
    pub open_seconds: i64,
    pub active_seconds: i64,
    pub is_finished: bool,
    pub is_clean_exit: bool,
    pub log_file_name: String,
    pub import_offset: i64,
    pub created: String,
    pub modified: String,
}

/// What the log's `start` line gives a session on first sight.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct BlenderSessionStart {
    pub blender_version_id: Option<String>,
    pub version: String,
    pub series: String,
    pub build_hash: String,
    pub binary_path: String,
    pub pid: i64,
    pub started_at: String,
    pub log_file_name: String,
}

/// What one read of the log adds to a session: absolute values for the clock fields,
/// deltas for everything that is summed.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct BlenderSessionProgress {
    pub ended_at: String,
    pub open_seconds: i64,
    pub active_seconds_delta: i64,
    pub is_finished: bool,
    pub is_clean_exit: bool,
    pub import_offset: i64,
    pub counters: Vec<(String, i64)>,
}

/// One counter of one session (`cube_deleted`, `undo`, `render_seconds`, ...).
#[derive(Default, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct BlenderSessionCounter {
    pub blender_session_id: String,
    pub kind: String,
    pub value: i64,
}

/// The session table in a handful of numbers, over the sessions started since a date.
#[derive(Default, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct BlenderSessionTotals {
    pub open_seconds: i64,
    pub active_seconds: i64,
    pub longest_session_seconds: i64,
    pub sessions: i64,
    pub finished_sessions: i64,
    pub unclean_exits: i64,
    /// Sessions of a build whose version string says Alpha.
    pub alpha_sessions: i64,
    /// `ended_at` of the newest finished session; empty when none.
    pub latest_end: String,
}

/// Time in Blender per installed version, summed over its sessions. `blender_version_id` is
/// None for sessions whose version is no longer installed.
#[derive(Default, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct BlenderVersionTime {
    pub blender_version_id: Option<String>,
    pub open_seconds: i64,
    pub active_seconds: i64,
    pub sessions: i64,
    pub last_used: String,
}
