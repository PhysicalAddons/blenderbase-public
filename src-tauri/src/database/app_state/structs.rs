use std::sync::Mutex;
use std::time::Instant;

use crate::core::{ActionTimestamp, DownloadableBlenderVersion};

#[derive(Debug)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub http_client: reqwest::Client,
    pub action_timeouts: Mutex<ActionTimestamp>,
    /// The last release scrape and when it ran: refreshes within a minute of it
    /// are answered from here without touching the mirror at all.
    pub release_scrape_cache: Mutex<Option<(Instant, Vec<DownloadableBlenderVersion>)>>,
}
