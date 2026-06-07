use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Default, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct BlenderInstallationLocation {
    pub id: String,
    pub is_default: bool,
    pub full_control: bool, // Protected
    pub modify: bool,
    pub read_and_execute: bool,
    pub list_folder_contents: bool,
    pub read: bool,
    pub write: bool, // Protected
    pub special_permissions: bool,
    pub directory_path: String,
    pub created_by: Option<String>,
    pub created: String,
    pub modified: String,
}
