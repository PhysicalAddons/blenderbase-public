use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

/// One unlocked achievement. `is_seen` goes to true when the Stats view is opened; the
/// title-bar badge shows while any unlock is unseen. Filled from phase 3 on.
#[derive(Default, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct AchievementUnlock {
    pub achievement_id: String,
    pub unlocked_at: String,
    pub blender_session_id: Option<String>,
    pub is_seen: bool,
    pub created: String,
}
