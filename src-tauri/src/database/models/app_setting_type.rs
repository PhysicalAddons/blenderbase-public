use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Default, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct AppSettingType {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: String,
    pub parent_app_setting_type_id: Option<i64>,
    pub created: String,
    pub modified: String,
}
