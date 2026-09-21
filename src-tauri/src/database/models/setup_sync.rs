use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

/// The folder a setup is kept in sync through, and the last setup this computer saved
/// there or applied from there. One row, id 1.
#[derive(Default, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct SetupSync {
    pub id: i64,
    pub folder_path: String,
    pub last_synced_hash: String,
    pub last_synced_at: String,
    pub created: String,
    pub modified: String,
}
