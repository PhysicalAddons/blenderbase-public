use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone)]
pub struct ActionTimestamp {
    pub fs_utility_cmd_check_internet_connection: Option<DateTime<Utc>>, // chrono::Utc::now().to_rfc3339()
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BlenderVersionBuildTypeDTO {
    pub id: String,
    pub text: String,
    pub is_default: bool,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DownloadableBlenderVersion {
    pub url: String,            // * ok url
    pub app: String,            // * ok Blender
    pub version: String,        // * ok x.x.x
    pub risk_id: String,        // * ok candidate
    pub branch: String,         // * ok v42, main-PR...
    pub patch: Option<String>,  // * ok PR...
    pub hash: String,           // * ok 824234324
    pub platform: String,       // * ok windows
    pub architecture: String,   // * ok amd64
    pub bitness: i32,           // * ok 64, 32
    pub file_mtime: i64,        // * ok 21312
    pub file_name: String,      // * ok ads.zip
    pub file_size: i64,         // * ok 123
    pub file_extension: String, // * ok zip
    pub release_cycle: String,  // * ok candiadte
    pub checksum: String,       // * ok
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PermissionDetails {
    pub full_control: bool,
    pub modify: bool,
    pub read_and_execute: bool,
    pub list_folder_contents: bool,
    pub read: bool,
    pub write: bool,
    pub special_permissions: bool,
}

// url: Some(downloadable_blender_version.url),
// app: Some(downloadable_blender_version.app),
// version: Some(downloadable_blender_version.version.clone()),
// series: Some(downloadable_blender_version.version.split(".").collect::<Vec<&str>>()[..=2].join(".")),
// risk_id: Some(downloadable_blender_version.risk_id),
// branch: Some(downloadable_blender_version.branch),

// patch_url: Some(match &downloadable_blender_version.patch {
//     Some(v) => format!(
//         "{}{}{}",
//         PROJECTS_BLENDER_ORG_BLENDER_BLENDER_PULLS,
//         FORWARD_SLASH_DELIMETER,
//         v.replace(PR, "")
//     ),
//     None => String::new(),
// }),
// patch: downloadable_blender_version.patch,
// hash_url: Some(format!(
//     "{}{}{}",
//     PROJECTS_BLENDER_ORG_BLENDER_BLENDER_COMMIT,
//     FORWARD_SLASH_DELIMETER,
//     downloadable_blender_version.hash
// )),
// hash: Some(downloadable_blender_version.hash),
// platform: Some(downloadable_blender_version.platform),
// architecture: Some(downloadable_blender_version.architecture),
// bitness: downloadable_blender_version.bitness,
// file_mtime: downloadable_blender_version.file_mtime,
// file_name: Some(downloadable_blender_version.file_name),
// file_size: downloadable_blender_version.file_size,
// file_extension: Some(downloadable_blender_version.file_extension),
// release_cycle: Some(downloadable_blender_version.release_cycle),
// checksum: Some(downloadable_blender_version.checksum),
// installation_directory_path: String::new(),
// executable_file_path: None,
// blender_installation_location_id: blender_installation_location.id,
// download_status_type_id: pending.id,
// created: chrono::Utc::now().to_rfc3339(),
// modified: chrono::Utc::now().to_rfc3339(),
