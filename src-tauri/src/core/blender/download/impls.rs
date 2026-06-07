use std::str::FromStr;

use tauri::AppHandle;

#[cfg(target_os = "windows")]
use crate::core::{WINDOWS, X32, X64};
use crate::{
    AppState, core::{
        B3D_LINK_REGEX, BLENDER, BLENDER_DOWNLOAD_LINK_REGEX, BLENDER_VERSION_REGEX, BUILDER_BLENDER_ORG_DOWNLOAD_DAILY_FORMAT_JSON_V2, BUILDER_BLENDER_ORG_DOWNLOAD_PATCH_FORMAT_JSON_V2, BlenderBuildKind, BlenderVersionBuildTypeDTO, DOWNLOAD_BLENDER_ORG_RELEASE, DownloadableBlenderVersion, FILE_REGEX_RELEASE, INTEL, ISO_FORMAT, LTS, LTS_VERSION_ARR, OrderKind, PUB_GRAPHICS_BLENDER_RELEASE, PUBLISH_TIMESTAMP_REGEX, STABLE, http_get_as_json, http_get_as_string
    }, database::DownloadStatusType
};

pub trait TBlenderDownloadService {
    async fn get_downloadable_blender_version_data(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        build: &str,
        order: &str,
    ) -> Result<Vec<DownloadableBlenderVersion>, String>;
    async fn fetch_blender_version_build_types(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<i64>,
        limit: Option<i64>,
        code: Option<String>,
    ) -> Result<Vec<BlenderVersionBuildTypeDTO>, String>;
    async fn update_download_blender_build_type(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        code: String,
    ) -> Result<(), String>;
    async fn fetch_download_status_type(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<i64>,
        limit: Option<i64>,
        code: Option<Vec<String>>,
    ) -> Result<Vec<DownloadStatusType>, String>;
}

pub struct BlenderDownloadServiceImpl;

impl TBlenderDownloadService for BlenderDownloadServiceImpl {
    async fn get_downloadable_blender_version_data(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        build: &str,
        order: &str,
    ) -> Result<Vec<DownloadableBlenderVersion>, String> {
        let build = match BlenderBuildKind::from_str(build) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("Failed get_downloadable_blender_version_data: {:?}", e));
            }
        };
        let order = match OrderKind::from_str(order) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("Failed get_downloadable_blender_version_data: {:?}", e));
            }
        };
        let url: String = match build {
            BlenderBuildKind::Release => String::from(DOWNLOAD_BLENDER_ORG_RELEASE),
            BlenderBuildKind::Daily => String::from(BUILDER_BLENDER_ORG_DOWNLOAD_DAILY_FORMAT_JSON_V2),
            BlenderBuildKind::Patch => String::from(BUILDER_BLENDER_ORG_DOWNLOAD_PATCH_FORMAT_JSON_V2),
        };
        // TODO TEST CODE
        // let url = String::from(BUILDER_BLENDER_ORG_DOWNLOAD_DAILY_FORMAT_JSON_V2);
        // build = BlenderBuildKind::Daily;
        // TODO TEST CODE
        let response_json: Vec<DownloadableBlenderVersion>;
        if build == BlenderBuildKind::Release {
            response_json = match Self::scrape_release_blender_versions(state, url).await {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed get_downloadable_blender_version_data: {:?}", e)),
            };
        } else {
            response_json = match http_get_as_json(state, url).await {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed get_downloadable_blender_version_data: {:?}", e)),
            };
        }
        #[cfg(target_os = "windows")]
        let mut filtered_data: Vec<DownloadableBlenderVersion> = response_json
            .into_iter()
            .filter(|p| {
                p.bitness == 64
                    && p.platform == "windows"
                    && (p.architecture == "amd64" || p.architecture == "x64")
                    && p.file_extension == "zip"
            })
            .collect();
        #[cfg(target_os = "macos")]
        let filtered_data = response_json
            .into_iter()
            .filter(|p| {
                p.bitness == 64
                    && p.platform == "darwin"
                    && p.architecture == "arm64"
                    && p.file_extension == "dmg"
            })
            .collect();
        #[cfg(target_os = "linux")]
        let filtered_data = response_json
            .into_iter()
            .filter(|p| {
                p.bitness == 64
                    && p.platform == "linux"
                    && p.architecture == "x86_64"
                    && p.file_extension == "xz"
            })
            .collect();
        match order {
            // Sort ASC
            OrderKind::Asc => filtered_data.sort_by(|a, b| a.version.cmp(&b.version)),
            // Sort DESC
            OrderKind::Desc => filtered_data.sort_by(|a, b| b.version.cmp(&a.version)),
        }
        return Ok(filtered_data);
    }
    async fn fetch_blender_version_build_types(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<i64>,
        limit: Option<i64>,
        code: Option<String>,
    ) -> Result<Vec<BlenderVersionBuildTypeDTO>, String> {
        let repository = state.blender_version_build_type_repository();
        let mut results = match repository.fetch(id, limit, code).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed fetch_blender_version_build_types: {:?}", e)),
        };
        results.sort_by(|a, b| a.id.cmp(&b.id));
        let results_dto = results
            .iter()
            .map(|x| BlenderVersionBuildTypeDTO {
                id: x.code.clone().to_lowercase(),
                text: x.code.clone(),
                is_default: x.is_default != 0,
            })
            .collect();
        Ok(results_dto)
    }
    async fn update_download_blender_build_type(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        code: String,
    ) -> Result<(), String> {
        let repository = state.blender_version_build_type_repository();
        let results = match repository.fetch(None, None, None).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed update_download_blender_build_type: {:?}", e)),
        };
        for mut entry in results {
            if entry.code == code {
                entry.is_default = 1;
                match repository.update(&entry).await {
                    Ok(_) => {}
                    Err(e) => return Err(format!("Failed update_download_blender_build_type: {:?}", e)),
                }
            } else {
                entry.is_default = 0;
                match repository.update(&entry).await {
                    Ok(_) => {}
                    Err(e) => return Err(format!("Failed update_download_blender_build_type: {:?}", e)),
                }
            }
        }
        Ok(())
    }
    async fn fetch_download_status_type(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<i64>,
        limit: Option<i64>,
        code: Option<Vec<String>>,
    ) -> Result<Vec<DownloadStatusType>, String> {
        let repository = state.download_status_type_repository();
        let mut results = match repository.fetch(id, limit, code).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed fetch_download_status_type: {:?}", e)),
        };
        results.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(results)
    }
    // async fn fetch_download_status_type(
    //     &self,
    //     app: AppHandle,
    //     state: tauri::State<'_, AppState>,
    //     id: Option<i64>,
    //     limit: Option<i64>,
    //     code: Option<Vec<String>>,
    // ) -> Result<Vec<DownloadStatusType>, String> {
    //     let repository = state.download_status_type_repository();
    //     let mut results = match repository.fetch(id, limit, code).await {
    //         Ok(v) => v,
    //         Err(e) => return Err(format!("{:?}", e))
    //     };
    //     results.sort_by(|a, b| a.id.cmp(&b.id));
    //     Ok(results)
    // }
}

impl BlenderDownloadServiceImpl {
    async fn scrape_release_blender_versions(
        state: tauri::State<'_, AppState>,
        url: String,
    ) -> Result<Vec<DownloadableBlenderVersion>, String> {
        let body = match http_get_as_string(state.clone(), url.clone()).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed scrape_release_blender_versions: {:?}", e)),
        };
        let b3d_link_regex = regex::bytes::Regex::new(B3D_LINK_REGEX).unwrap();
        let lines: Vec<&str> = body.lines().collect();
        let mut data: Vec<DownloadableBlenderVersion> = Vec::new();
        for line in lines {
            if let Some(captures) = b3d_link_regex.captures(line.as_bytes()) {
                // Accessing the first capture group (If "Blender3.6" counts as .get(0), then .get(1) is "3.6").
                if let Some(version) = captures.get(1) {
                    let version_str = match std::str::from_utf8(version.as_bytes()) {
                        Ok(v) => v,
                        Err(e) => return Err(format!("Failed scrape_release_blender_versions: {:?}", e)),
                    };
                    let version_float = match version_str.parse::<f32>() {
                        Ok(v) => v,
                        Err(e) => return Err(format!("Failed scrape_release_blender_versions: {:?}", e)),
                    };
                    if version_float < 3.1 {
                        continue;
                    }
                    let url = format!("{}{}{}", url, BLENDER, version_str);
                    match Self::scrape_release_blender_series(state.clone(), url).await {
                        Ok(mut v) => {
                            data.append(&mut v);
                        }
                        Err(e) => return Err(format!("Failed scrape_release_blender_versions: {:?}", e)),
                    }
                }
            }
        }
        return Ok(data);
    }
    async fn scrape_release_blender_series(
        state: tauri::State<'_, AppState>,
        url: String,
    ) -> Result<Vec<DownloadableBlenderVersion>, String> {
        let body = match http_get_as_string(state, url.clone()).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed scrape_release_blender_series: {:?}", e)),
        };
        // Define a regular expression
        let blender_link_regex = regex::Regex::new(BLENDER_DOWNLOAD_LINK_REGEX).unwrap();
        let timestamp_regex = regex::Regex::new(PUBLISH_TIMESTAMP_REGEX).unwrap();
        let version_regex = regex::Regex::new(BLENDER_VERSION_REGEX).unwrap();
        let mut blender_version: String = String::new();
        let mut blender_variant: String = String::new();
        let mut blender_release_timestamp: i64 = 0;
        let mut blender_file_size: f64 = 0.0;
        let mut blender_file_type: String = String::new();
        let mut file_name: &str;
        let lines: Vec<&str> = body.lines().collect();
        let mut data: Vec<DownloadableBlenderVersion> = Vec::new();
        for line in lines {
            if let Some(captures) = timestamp_regex.captures(line) {
                if let Some(filename) = captures.get(1) {
                    let regex = regex::Regex::new(FILE_REGEX_RELEASE).unwrap();
                    file_name = filename.as_str();
                    match file_name {
                        #[cfg(target_os = "windows")]
                        link if link.ends_with(".zip") && (regex.is_match(&file_name)) => {
                            blender_file_type = "zip".to_string();
                        }
                        #[cfg(target_os = "macos")]
                        link if link.ends_with(".dmg") && (regex.is_match(&file_name)) => {
                            blender_file_type = "dmg".to_string();
                        }
                        _ => {
                            continue;
                        }
                    }
                    if let Some(capture) = version_regex.captures(filename.as_str()) {
                        if let Some(version_match) = capture.get(0) {
                            let version_str = match version_match.as_str().parse::<String>() {
                                Ok(v) => v,
                                Err(e) => return Err(format!("Failed scrape_release_blender_series: {:?}", e)),
                            };
                            blender_version = version_str.clone();
                            let truncated_version: String =
                                version_str.split('.').take(2).collect::<Vec<_>>().join(".");
                            if LTS_VERSION_ARR.contains(&truncated_version.as_str()) {
                                blender_variant = String::from(LTS);
                            } else {
                                blender_variant = String::from(STABLE);
                            }
                        }
                    }
                }
                if let Some(date) = captures.get(3) {
                    let raw_date = date.as_str();
                    let date_time =
                        match chrono::NaiveDateTime::parse_from_str(raw_date, ISO_FORMAT) {
                            Ok(v) => v,
                            Err(e) => return Err(format!("Failed scrape_release_blender_series: {:?}", e)),
                        };
                    blender_release_timestamp = date_time.and_utc().timestamp();
                }
                if let Some(size) = captures.get(2) {
                    blender_file_size = match size.as_str().replace(" MiB", "").parse::<f64>() {
                        Ok(v) => v * 1048576.0,
                        Err(e) => return Err(format!("Failed scrape_release_blender_series: {:?}", e)),
                    };
                }
            }
            // Check for lines containing blender download links
            if let Some(captures) = blender_link_regex.captures(line) {
                if let Some(link) = captures.get(1) {
                    let link_str = link.as_str();
                    // This is for the EU mirror to work correctly.
                    if link_str != PUB_GRAPHICS_BLENDER_RELEASE {
                        #[cfg(target_os = "windows")]
                        let architecture = match (
                            link_str.contains(WINDOWS),
                            link_str.contains("64"),
                            link_str.contains("32"),
                        ) {
                            (true, true, _) => X64,
                            (true, _, true) => X32,
                            _ => "unknown",
                        };
                        #[cfg(target_os = "macos")]
                        let architecture = match (
                            link_str.contains(MACOS),
                            link_str.contains(ARM64),
                            link_str.contains(X64),
                        ) {
                            (true, true, _) => APPLE_SILICON,
                            (true, _, true) => INTEL,
                            _ => "unknown",
                        };
                        if architecture == INTEL {
                            continue;
                        }
                        let new_app = DownloadableBlenderVersion {
                            url: format!("{}/{}", url, link_str),
                            app: BLENDER.to_string(),
                            version: blender_version.to_string(),
                            risk_id: blender_variant.to_string(),
                            branch: String::new(),
                            patch: None,
                            hash: String::new(),
                            #[cfg(target_os = "windows")]
                            platform: String::from("windows"),
                            #[cfg(target_os = "macos")]
                            platform: String::from("darwin"),
                            architecture: architecture.to_string(),
                            bitness: match architecture
                                .to_string()
                                .replace("x", "")
                                .replace("arm", "")
                                .parse::<i32>()
                            {
                                Ok(v) => v,
                                Err(e) => return Err(format!("Failed scrape_release_blender_series: {:?}", e)),
                            },
                            file_mtime: blender_release_timestamp,
                            file_name: link_str.to_string(),
                            file_size: blender_file_size.round() as i64,
                            file_extension: blender_file_type.to_string(),
                            release_cycle: blender_variant.to_string(),
                            checksum: String::new(),
                        };
                        data.push(new_app);
                    }
                }
            }
        }
        Ok(data)
    }
}
