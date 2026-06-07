use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Default, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Addon {
    pub id: String,
    pub is_enabled: bool,
    pub is_symbolic_link: bool,
    pub main_python_file_path: String,
    pub installation_directory: String,
    pub variant_type: Option<String>,
    pub functional_name: Option<String>,
    pub name: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub blender_version: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub warning: Option<String>,
    pub documentation_url: Option<String>,
    pub tracker_url: Option<String>,
    pub support: Option<String>,
    pub category: Option<String>,
    pub parent_blender_version_id: Option<String>,
    pub created: String,
    pub modified: String,
}
