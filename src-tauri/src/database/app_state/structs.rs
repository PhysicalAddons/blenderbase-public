use std::sync::Mutex;

use crate::core::ActionTimestamp;

#[derive(Debug)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub http_client: reqwest::Client,
    pub action_timeouts: Mutex<ActionTimestamp>,
}
