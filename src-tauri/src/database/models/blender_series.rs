use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Default, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct BlenderSeries {
    pub id: String,
    pub is_collapsed: bool,
    pub series: String,
    pub config_directory_path: String,
    pub created: String,
    pub modified: String,
}
