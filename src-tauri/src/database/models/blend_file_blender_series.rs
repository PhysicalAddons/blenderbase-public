use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Default, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct BlendFileBlenderSeries {
    pub blend_file_id: String,
    pub blender_series_id: String,
    pub created: String,
    pub modified: String,
}
