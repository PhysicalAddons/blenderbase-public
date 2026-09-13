use std::{str::FromStr, sync::LazyLock};

use tauri::AppHandle;

#[cfg(target_os = "windows")]
use crate::core::{WINDOWS, X32, X64};
#[cfg(target_os = "macos")]
use crate::core::{ARM64, MACOS, X64};
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use crate::core::{LINUX, X64, X86_64};
use crate::{
    AppState, core::{
        B3D_LINK_REGEX, BLENDER, BLENDER_DOWNLOAD_LINK_REGEX, BLENDER_VERSION_REGEX, BUILDER_BLENDER_ORG_DOWNLOAD_DAILY_FORMAT_JSON_V2, BUILDER_BLENDER_ORG_DOWNLOAD_PATCH_FORMAT_JSON_V2, BlenderBuildKind, BlenderVersionBuildTypeDTO, DOWNLOAD_BLENDER_ORG_RELEASE, DownloadableBlenderVersion, FILE_REGEX_RELEASE, ISO_FORMAT, LTS, LTS_VERSION_ARR, OrderKind, PUB_GRAPHICS_BLENDER_RELEASE, PUBLISH_TIMESTAMP_REGEX, STABLE, http_get_as_json, http_get_as_string, http_get_as_string_with_client
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

// The release listing is scraped line by line for every series; compiling
// these once keeps the per-line work to the match itself.
static B3D_LINK_RE: LazyLock<regex::bytes::Regex> =
    LazyLock::new(|| regex::bytes::Regex::new(B3D_LINK_REGEX).expect("valid regex"));
static BLENDER_LINK_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(BLENDER_DOWNLOAD_LINK_REGEX).expect("valid regex"));
static TIMESTAMP_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(PUBLISH_TIMESTAMP_REGEX).expect("valid regex"));
static VERSION_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(BLENDER_VERSION_REGEX).expect("valid regex"));
static FILE_RELEASE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(FILE_REGEX_RELEASE).expect("valid regex"));

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
        let mut filtered_data: Vec<DownloadableBlenderVersion> = response_json
            .into_iter()
            .filter(|p| {
                p.bitness == 64
                    && p.platform == "darwin"
                    && p.architecture == "arm64"
                    && p.file_extension == "dmg"
            })
            .collect();
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let mut filtered_data: Vec<DownloadableBlenderVersion> = response_json
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
        let b3d_link_regex = &*B3D_LINK_RE;
        let lines: Vec<&str> = body.lines().collect();
        let mut series_urls: Vec<String> = Vec::new();
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
                    series_urls.push(format!("{}{}{}", url, BLENDER, version_str));
                }
            }
        }
        // One directory page per series; fetching them one after another took most of the
        // time, so they run concurrently with a small cap to stay polite to the mirror.
        let client = state.http_client.clone();
        let limiter = std::sync::Arc::new(tokio::sync::Semaphore::new(6));
        let mut tasks = tokio::task::JoinSet::new();
        for (index, series_url) in series_urls.into_iter().enumerate() {
            let client = client.clone();
            let limiter = limiter.clone();
            tasks.spawn(async move {
                let _permit = limiter.acquire_owned().await.ok();
                (index, Self::scrape_release_blender_series(client, series_url).await)
            });
        }
        let mut chunks: Vec<(usize, Vec<DownloadableBlenderVersion>)> = Vec::new();
        while let Some(joined) = tasks.join_next().await {
            match joined {
                Ok((index, Ok(v))) => chunks.push((index, v)),
                Ok((_, Err(e))) => return Err(format!("Failed scrape_release_blender_versions: {:?}", e)),
                Err(e) => return Err(format!("Failed scrape_release_blender_versions: {:?}", e)),
            }
        }
        chunks.sort_by_key(|(index, _)| *index);
        let mut data: Vec<DownloadableBlenderVersion> = Vec::new();
        for (_, mut v) in chunks {
            data.append(&mut v);
        }
        return Ok(data);
    }
    async fn scrape_release_blender_series(
        client: reqwest::Client,
        url: String,
    ) -> Result<Vec<DownloadableBlenderVersion>, String> {
        let body = match http_get_as_string_with_client(client, url.clone()).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed scrape_release_blender_series: {:?}", e)),
        };
        let blender_link_regex = &*BLENDER_LINK_RE;
        let timestamp_regex = &*TIMESTAMP_RE;
        let version_regex = &*VERSION_RE;
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
                    let regex = &*FILE_RELEASE_RE;
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
                        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
                        link if link.ends_with(".tar.xz") && (regex.is_match(&file_name)) => {
                            // builder.blender.org reports `.tar.xz` builds as extension "xz".
                            blender_file_type = "xz".to_string();
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
                        let link_lower: String = link_str.to_ascii_lowercase();
                        #[cfg(target_os = "windows")]
                        let architecture: &str = match (
                            link_lower.contains(WINDOWS),
                            link_lower.contains("64"),
                            link_lower.contains("32"),
                        ) {
                            (true, true, _) => X64,
                            (true, _, true) => X32,
                            _ => "unknown",
                        };
                        // Only Apple Silicon builds are offered; Intel (`x64`) images are skipped.
                        #[cfg(target_os = "macos")]
                        let architecture: &str = match (
                            link_lower.contains(MACOS),
                            link_lower.contains(ARM64),
                            link_lower.contains(X64),
                        ) {
                            (true, true, _) => ARM64,
                            (true, _, true) => continue,
                            _ => "unknown",
                        };
                        // Release tarballs are named `linux-x64`; the daily feed calls the
                        // same architecture `x86_64`, so that is the value stored.
                        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
                        let architecture: &str = match (
                            link_lower.contains(LINUX),
                            link_lower.contains(X64) || link_lower.contains(X86_64),
                        ) {
                            (true, true) => X86_64,
                            _ => "unknown",
                        };
                        let bitness: i32 = if architecture.ends_with("32") { 32 } else { 64 };
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
                            #[cfg(not(any(target_os = "windows", target_os = "macos")))]
                            platform: String::from("linux"),
                            architecture: architecture.to_string(),
                            bitness,
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
