use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Default, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct BlendFile {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub created_datetime: String,
    pub modified_datetime: String,
    pub accessed_datetime: String,
    pub last_used_blender_version_id: Option<String>,
    pub created: String,
    pub modified: String,
}
