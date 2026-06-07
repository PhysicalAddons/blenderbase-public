use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Default, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct MeasurementUnitType {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: String,
    pub created: String,
    pub modified: String,
}
