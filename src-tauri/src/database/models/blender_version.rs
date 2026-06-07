use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Default, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct BlenderVersion {
    pub id: String,
    pub is_default: bool,
    pub custom_name: Option<String>,
    pub url: Option<String>, // pub download_url: Option<String>,
    pub app: Option<String>,
    pub version: Option<String>, // pub version: Option<String>,
    pub series: Option<String>,  // pub series: Option<String>,
    pub risk_id: Option<String>, // pub variant_type: Option<String>,
    pub branch: Option<String>,
    pub patch_url: Option<String>, // pub pull_request_link_url: Option<String>,
    pub patch: Option<String>,     // pub pull_request_number: Option<String>,
    pub hash_url: Option<String>,  // pub commit_link_url: Option<String>,
    pub hash: Option<String>,      // pub commit_hash: Option<String>,
    pub platform: Option<String>,
    pub architecture: Option<String>, // pub architecture: Option<String>,
    pub bitness: i32,
    pub file_mtime: i64, // pub release_date: Option<String>,
    pub file_name: Option<String>,
    pub file_size: i64,                 // pub app_size: i64,
    pub file_extension: Option<String>, // pub file_extension: Option<String>,
    pub release_cycle: Option<String>,
    pub checksum: Option<String>,
    pub installation_directory_path: String,
    pub executable_file_path: Option<String>,
    pub blender_installation_location_id: String,
    pub download_status_type_id: i64,
    pub created: String,
    pub modified: String,
}
