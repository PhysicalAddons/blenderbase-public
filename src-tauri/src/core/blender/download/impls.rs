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

/// One scraped series folder: the stamp the index showed for it, and what it held.
#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
struct CachedSeries {
    stamp: String,
    versions: Vec<DownloadableBlenderVersion>,
}

/// The release scrape kept between runs (`release-cache.json` in the app data
/// folder). Unreadable or missing files just mean a full scrape.
#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
struct ReleaseCache {
    series: std::collections::HashMap<String, CachedSeries>,
}

impl ReleaseCache {
    fn load(path: &std::path::Path) -> Self {
        std::fs::File::open(path)
            .ok()
            .and_then(|f| serde_json::from_reader(f).ok())
            .unwrap_or_default()
    }
    fn save(&self, path: &std::path::Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
        serde_json::to_writer(file, self).map_err(|e| e.to_string())
    }
}

fn release_cache_path() -> std::path::PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(std::env::temp_dir);
    path.push(crate::core::COM_PHYSICALADDONS_BLENDERBASE);
    path.push("release-cache.json");
    path
}

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
        // Be a light touch on the mirror. The top-level index (one small page)
        // lists a modification stamp for every series folder; a series is only
        // fetched again when that stamp differs from the one recorded with the
        // last scrape, and the scrape is kept on disk so restarts do not start
        // over. Repeated refreshes within a minute do not even fetch the index.
        const INDEX_MIN_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60);
        if let Ok(recent) = state.release_scrape_cache.lock() {
            if let Some((at, versions)) = recent.as_ref() {
                if at.elapsed() < INDEX_MIN_INTERVAL {
                    return Ok(versions.clone());
                }
            }
        }
        let body = match http_get_as_string(state.clone(), url.clone()).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed scrape_release_blender_versions: {:?}", e)),
        };
        // (series folder URL, stamp shown for it in the index), oldest series first.
        let b3d_link_regex = &*B3D_LINK_RE;
        let timestamp_regex = &*TIMESTAMP_RE;
        let mut series: Vec<(String, String)> = Vec::new();
        for line in body.lines() {
            let Some(row) = timestamp_regex.captures(line) else { continue };
            let (Some(href), Some(stamp)) = (row.get(1), row.get(3)) else { continue };
            let Some(link) = b3d_link_regex.captures(href.as_str().as_bytes()) else { continue };
            let Some(version) = link.get(1) else { continue };
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
            series.push((format!("{}{}{}", url, BLENDER, version_str), stamp.as_str().to_string()));
        }
        let cache_path = release_cache_path();
        let mut cache = ReleaseCache::load(&cache_path);
        let mut chunks: Vec<(usize, Vec<DownloadableBlenderVersion>)> = Vec::new();
        let mut pending: Vec<(usize, String, String)> = Vec::new();
        for (index, (series_url, stamp)) in series.iter().enumerate() {
            match cache.series.get(series_url) {
                Some(entry) if entry.stamp == *stamp => chunks.push((index, entry.versions.clone())),
                _ => pending.push((index, series_url.clone(), stamp.clone())),
            }
        }
        // Changed or unknown series, one at a time with a short pause between
        // them: the mirror throttles even modest parallel bursts, and a first
        // run is the only time more than a couple of series need reading.
        // The fetch itself backs off if a 429 still comes.
        let client = state.http_client.clone();
        let mut changed = false;
        let last = pending.len().saturating_sub(1);
        for (n, (index, series_url, stamp)) in pending.into_iter().enumerate() {
            match Self::scrape_release_blender_series(client.clone(), series_url.clone()).await {
                Ok(v) => {
                    cache.series.insert(series_url, CachedSeries { stamp, versions: v.clone() });
                    changed = true;
                    chunks.push((index, v));
                }
                Err(e) => return Err(format!("Failed scrape_release_blender_versions: {:?}", e)),
            }
            if n < last {
                tokio::time::sleep(std::time::Duration::from_millis(400)).await;
            }
        }
        if changed {
            // Series that vanished from the index are dropped with the rewrite.
            cache.series.retain(|k, _| series.iter().any(|(u, _)| u == k));
            if let Err(e) = cache.save(&cache_path) {
                eprintln!("Could not write the release cache {}: {}", cache_path.display(), e);
            }
        }
        chunks.sort_by_key(|(index, _)| *index);
        let mut data: Vec<DownloadableBlenderVersion> = Vec::new();
        for (_, mut v) in chunks {
            data.append(&mut v);
        }
        if let Ok(mut recent) = state.release_scrape_cache.lock() {
            *recent = Some((std::time::Instant::now(), data.clone()));
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
