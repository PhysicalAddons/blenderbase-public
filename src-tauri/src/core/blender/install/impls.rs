use std::str::FromStr;

use regex::Regex;
use tauri::AppHandle;

use crate::{
    core::{
        delete_directory, delete_file, instance_native_ask_dialog_window, launch_executable,
        find_sha256_in_listing, http_get_as_string, is_sha256_hex, open_archive,
        probe_blender_build_info, resolve_blender_console_executable, sha256_of_file, write_file,
        DownloadStatusKind, DownloadableBlenderVersion, OrderKind,
        BLENDERBASE_DOWNLOAD_DATA, BLENDER_LAUNCHER_EXE, BLENDER_ORG_RELEASE_CHECKSUM_BASE,
        BLENDER_VERSION_VARIANT_REGEX,
        FORWARD_SLASH_DELIMETER, LTS, LTS_VERSION_ARR, PR, STABLE, PROJECTS_BLENDER_ORG_BLENDER_BLENDER_COMMIT,
        PROJECTS_BLENDER_ORG_BLENDER_BLENDER_PULLS,
    },
    database::{BlenderInstallationLocation, BlenderVersion},
    AppState,
};

pub trait TBlenderInstallService {
    async fn refresh_blender_versions_init(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn refresh_blender_versions(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn init_blender_version(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        downloadable_blender_version: DownloadableBlenderVersion,
        blender_installation_location: BlenderInstallationLocation,
    ) -> Result<BlenderVersion, String>;
    async fn install_blender_version(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
        archive_file_path: std::path::PathBuf,
    ) -> Result<(), String>;
    async fn write_blender_version_download_data(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        downloadable_blender_version: DownloadableBlenderVersion,
        directory_path: std::path::PathBuf,
    ) -> Result<(), String>;
    async fn insert_blender_version(
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        executable_file_path: std::path::PathBuf,
        blender_installation_location_id: String,
    ) -> Result<(), String>;
    async fn refresh_blender_version(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_version: BlenderVersion,
        blender_installation_location_id: String,
    ) -> Result<(), String>;
    async fn update_blender_version(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_version: BlenderVersion,
    ) -> Result<(), String>;
    async fn set_blender_version_as_default(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
    ) -> Result<(), String>;
    async fn update_blender_version_download_status_type(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_version: BlenderVersion,
        download_status_type: &str,
    ) -> Result<(), String>;
    async fn set_default_blender_version(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn fetch_blender_versions(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
        limit: Option<i64>,
        is_defeault: Option<bool>,
        executable_file_path: Option<String>,
        series: Option<String>,
        order: &str,
        download_status_types: Option<Vec<String>>,
    ) -> Result<Vec<BlenderVersion>, String>;
    async fn update_install_blender_build_type(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        code: String,
    ) -> Result<(), String>;
    async fn delete_blender_version(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
    ) -> Result<(), String>;
    /// Fills in build date, commit hash, branch and release cycle for versions that were
    /// installed without download data, by asking each build with `--version`.
    async fn refresh_blender_version_details(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        ids: Vec<String>,
    ) -> Result<Vec<BlenderVersion>, String>;
    async fn launch_blender_version(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
    ) -> Result<(), String>;
}

pub struct BlenderInstallServiceImpl;

impl TBlenderInstallService for BlenderInstallServiceImpl {
    async fn refresh_blender_versions_init(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        let bilr = state.blender_installation_location_repository();
        let bvr = state.blender_version_repository();
        let blender_installation_locations = match bilr.fetch(None, None, None, None).await {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("Failed refresh_blender_versions_init: {:?}", e));
            }
        };
        for blender_installation_location in blender_installation_locations {
            let directory_entries =
                match std::fs::read_dir(blender_installation_location.directory_path) {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(format!("Failed refresh_blender_versions_init: {:?}", e));
                    }
                };
            for directory in directory_entries {
                let entry = match directory {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(format!("Failed refresh_blender_versions_init: {:?}", e));
                    }
                };
                if !entry.path().is_dir() {
                    continue;
                }
                let executable_file_path = entry.path().join(BLENDER_LAUNCHER_EXE);
                if !executable_file_path.exists() {
                    continue;
                }
                let mut existing_entries = match bvr
                    .fetch(
                        None,
                        None,
                        None,
                        Some(executable_file_path.to_string_lossy().to_string()),
                        None,
                    )
                    .await
                {
                    Ok(val) => val,
                    Err(e) => {
                        return Err(format!("Failed refresh_blender_versions_init: {:?}", e));
                    }
                };
                if !existing_entries.is_empty() {
                    match Self::refresh_blender_version(
                        &self,
                        app.clone(),
                        state.clone(),
                        existing_entries.remove(0),
                        blender_installation_location.id.clone(),
                    )
                    .await
                    {
                        Ok(_) => continue,
                        Err(e) => return Err(format!("Failed refresh_blender_versions_init: {:?}", e)),
                    }
                }
                match Self::insert_blender_version(
                    app.clone(),
                    state.clone(),
                    executable_file_path,
                    blender_installation_location.id.clone(),
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(format!("Failed refresh_blender_versions_init: {:?}", e));
                    }
                }
            }
        }
        let current_entries = match bvr.fetch(None, None, None, None, None).await {
            Ok(val) => val,
            Err(e) => {
                return Err(format!("Failed refresh_blender_versions_init: {:?}", e));
            }
        };
        for entry in current_entries {
            let should_delete = match &entry.executable_file_path {
                Some(executable_path) => {
                    let path = std::path::Path::new(executable_path);
                    !path.exists()
                }
                None => true,
            } || entry.download_status_type_id != 4;
            if should_delete {
                if let Err(e) = bvr.delete(entry.id).await {
                    return Err(format!("Failed refresh_blender_versions_init: {:?}", e));
                }
            }
        }
        Ok(())
    }
    async fn refresh_blender_versions(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        let bilr = state.blender_installation_location_repository();
        let bvr = state.blender_version_repository();
        let blender_installation_locations = match bilr.fetch(None, None, None, None).await {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("Failed refresh_blender_versions:{:?}", e));
            }
        };
        for blender_installation_location in blender_installation_locations {
            let directory_entries =
                match std::fs::read_dir(blender_installation_location.directory_path) {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(format!("Failed refresh_blender_versions:{:?}", e));
                    }
                };
            for directory in directory_entries {
                let entry = match directory {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(format!("Failed refresh_blender_versions:{:?}", e));
                    }
                };
                if !entry.path().is_dir() {
                    continue;
                }
                let executable_file_path = entry.path().join(BLENDER_LAUNCHER_EXE);
                if !executable_file_path.exists() {
                    continue;
                }
                let mut existing_entries = match bvr
                    .fetch(
                        None,
                        None,
                        None,
                        Some(executable_file_path.to_string_lossy().to_string()),
                        None,
                    )
                    .await
                {
                    Ok(val) => val,
                    Err(e) => {
                        return Err(format!("Failed refresh_blender_versions:{:?}", e));
                    }
                };
                if !existing_entries.is_empty() {
                    match Self::refresh_blender_version(
                        &self,
                        app.clone(),
                        state.clone(),
                        existing_entries.remove(0),
                        blender_installation_location.id.clone(),
                    )
                    .await
                    {
                        Ok(_) => continue,
                        Err(e) => return Err(format!("Failed refresh_blender_versions:{:?}", e)),
                    }
                }
                match Self::insert_blender_version(
                    app.clone(),
                    state.clone(),
                    executable_file_path,
                    blender_installation_location.id.clone(),
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(format!("Failed refresh_blender_versions:{:?}", e));
                    }
                }
            }
        }
        let current_entries = match bvr.fetch(None, None, None, None, None).await {
            Ok(val) => val,
            Err(e) => {
                return Err(format!("Failed refresh_blender_versions:{:?}", e));
            }
        };
        for entry in current_entries {
            if let Some(executable_path) = &entry.executable_file_path {
                let path = std::path::Path::new(executable_path);
                if !path.exists() {
                    if let Err(e) = bvr.delete(entry.id).await {
                        return Err(format!("Failed refresh_blender_versions:{:?}", e));
                    }
                }
            }
        }
        Ok(())
    }

    async fn init_blender_version(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        downloadable_blender_version: DownloadableBlenderVersion,
        blender_installation_location: BlenderInstallationLocation,
    ) -> Result<BlenderVersion, String> {
        let blender_version_repository = state.blender_version_repository();
        let download_status_type_repository = state.download_status_type_repository();
        let mut download_status_types = match download_status_type_repository
            .fetch(
                None,
                None,
                Some(vec![DownloadStatusKind::Pending.to_string()]),
            )
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed init_blender_version: {:?}", e)),
        };
        if download_status_types.is_empty() {
            return Err(format!(
                "Failed init_blender_version"
            ));
        }
        let pending: crate::database::DownloadStatusType = download_status_types.remove(0);
        let blender_version = BlenderVersion {
            id: uuid::Uuid::new_v4().to_string(),
            is_default: match Self::is_default(app.clone(), state.clone()).await {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed init_blender_version: {}", e)),
            },
            custom_name: None,
            url: Some(downloadable_blender_version.url),
            app: Some(downloadable_blender_version.app),
            version: Some(downloadable_blender_version.version.clone()),
            series: Some(
                downloadable_blender_version
                    .version
                    .split('.')
                    .take(2)
                    .collect::<Vec<_>>()
                    .join("."),
            ),
            risk_id: Some(downloadable_blender_version.risk_id),
            branch: Some(downloadable_blender_version.branch),

            patch_url: Some(match &downloadable_blender_version.patch {
                Some(v) => format!(
                    "{}{}{}",
                    PROJECTS_BLENDER_ORG_BLENDER_BLENDER_PULLS,
                    FORWARD_SLASH_DELIMETER,
                    v.replace(PR, "")
                ),
                None => String::new(),
            }),
            patch: downloadable_blender_version.patch,
            hash_url: Some(format!(
                "{}{}{}",
                PROJECTS_BLENDER_ORG_BLENDER_BLENDER_COMMIT,
                FORWARD_SLASH_DELIMETER,
                downloadable_blender_version.hash
            )),
            hash: Some(downloadable_blender_version.hash),
            platform: Some(downloadable_blender_version.platform),
            architecture: Some(downloadable_blender_version.architecture),
            bitness: downloadable_blender_version.bitness,
            file_mtime: downloadable_blender_version.file_mtime,
            file_name: Some(downloadable_blender_version.file_name),
            file_size: downloadable_blender_version.file_size,
            file_extension: Some(downloadable_blender_version.file_extension),
            release_cycle: Some(downloadable_blender_version.release_cycle),
            checksum: Some(downloadable_blender_version.checksum),
            installation_directory_path: String::new(),
            executable_file_path: None,
            blender_installation_location_id: blender_installation_location.id,
            download_status_type_id: pending.id,
            created: chrono::Utc::now().to_rfc3339(),
            modified: chrono::Utc::now().to_rfc3339(),
        };
        let existing_entry_option = match Self::find_existing(
            app.clone(),
            state.clone(),
            &blender_version,
            DownloadStatusKind::Pending,
        )
        .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed init_blender_version: {}", e)),
        };
        match existing_entry_option {
            Some(v) => Ok(v),
            None => {
                match blender_version_repository.insert(&blender_version).await {
                    // TODO if loose pendings on app start, delete the entries.
                    Ok(_) => Ok(blender_version),
                    Err(e) => return Err(format!("Failed init_blender_version: {}", e)),
                }
            }
        }
    }

    async fn install_blender_version(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
        archive_file_path: std::path::PathBuf,
    ) -> Result<(), String> {
        let repository = state.blender_version_repository();
        let mut blender_versions =
            match repository.fetch(id, None, None, None, None).await {
                Ok(v) => v,
                Err(e) => return Err(format!(
                    "Failed install_blender_version: Failed to extract downloaded Blender versions files from archive file: {:?}",
                    e
                )),
            };
        if blender_versions.is_empty() {
            return Err(format!("Failed install_blender_version"));
        }
        let mut blender_version = blender_versions.remove(0);
        // Integrity check before anything is extracted. A build whose
        // checksum cannot be established is refused, not installed unverified.
        let expected =
            Self::expected_archive_sha256(state.clone(), &blender_version, &archive_file_path)
                .await?;
        let actual = match sha256_of_file(archive_file_path.clone()).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed install_blender_version: {}", e)),
        };
        if actual != expected {
            let _ = delete_file(archive_file_path.clone()).await;
            return Err(format!(
                "Failed install_blender_version: the downloaded file failed its integrity check (expected SHA-256 {}, got {}). The download was discarded; please try again.",
                expected, actual
            ));
        }
        let installation_directory_path =
            match open_archive(archive_file_path.clone()).await {
                Ok(v) => v,
                Err(e) => return Err(format!(
                    "Failed install_blender_version: Failed to extract downloaded Blender versions files from archive file: {:?}",
                    e
                )),
            };
        blender_version.installation_directory_path =
            installation_directory_path.to_string_lossy().to_string();
        blender_version.executable_file_path = Some(
            installation_directory_path
                .join("blender-launcher.exe")
                .to_string_lossy()
                .to_string(),
        );
        match delete_file(archive_file_path).await {
            Ok(_) => {}
            Err(e) => return Err(format!("Failed install_blender_version: Failed to delete downloaded archive file: {:?}", e)),
        }
        match repository.update(&blender_version).await {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("Failed install_blender_version: Failed to delete downloaded archive file: {:?}", e)),
        }
    }

    async fn write_blender_version_download_data(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
        downloadable_blender_version: DownloadableBlenderVersion,
        directory_path: std::path::PathBuf,
    ) -> Result<(), String> {
        let file_path: std::path::PathBuf =
            directory_path.join(format!("{}{}", BLENDERBASE_DOWNLOAD_DATA, ".json"));
        let content = serde_json::to_string(&downloadable_blender_version).unwrap();
        match write_file(file_path, content).await {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("Failed write_blender_version_download_data: {:?}", e)),
        }
    }

    async fn insert_blender_version(
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        executable_file_path: std::path::PathBuf,
        blender_installation_location_id: String,
    ) -> Result<(), String> {
        let bvr = state.blender_version_repository();
        let dstr = state.download_status_type_repository();
        let mut download_status_types = match dstr
            .fetch(
                None,
                None,
                Some(vec![DownloadStatusKind::Completed.to_string()]),
            )
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed insert_blender_version: {:?}", e)),
        };
        if download_status_types.is_empty() {
            return Err(format!(
                "Failed insert_blender_version: Failed to fetch download status types when inserting Blender version"
            ));
        }
        let download_status_type_completed = download_status_types.remove(0);
        let parent_dir = match executable_file_path.parent() {
            Some(val) => val,
            None => return Err(format!("Failed insert_blender_version: Failed to get file path parent")),
        };
        let dir_name = match parent_dir.file_name() {
            Some(val) => val.to_string_lossy().to_string(),
            None => return Err(format!("Failed insert_blender_version: Failed to get file name")),
        };
        let re = match Regex::new(BLENDER_VERSION_VARIANT_REGEX) {
            Ok(val) => val,
            Err(e) => return Err(format!("Failed insert_blender_version: {:?}", e)),
        };
        let mut version = String::new();
        let mut variant = String::new();
        if let Some(caps) = re.captures(&dir_name) {
            version = caps
                .name("version")
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            variant = caps
                .name("variant")
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
        }

        let path = format!(
            "{}/{}{}",
            parent_dir.to_string_lossy(),
            BLENDERBASE_DOWNLOAD_DATA,
            ".json"
        );
        let download_data_file_path = std::path::Path::new(path.as_str());
        let mut downloadable_blender_version = DownloadableBlenderVersion::default();
        if download_data_file_path.exists() {
            let download_data_file = match std::fs::File::open(download_data_file_path) {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed insert_blender_version: {:?}", e)),
            };
            downloadable_blender_version = match serde_json::from_reader(download_data_file) {
                Ok(v) => v,
                Err(e) => {
                    return Err(format!(
                        "Failed insert_blender_version: invalid download data file {}: {:?}",
                        path, e
                    ))
                }
            };
        }
        // TODO add defaults for values, if download data does not exist.
        let b = BlenderVersion {
            id: uuid::Uuid::new_v4().to_string(),
            is_default: false,
            custom_name: None,
            url: Some(downloadable_blender_version.url),
            app: Some(downloadable_blender_version.app),
            version: Some(version.clone()),
            series: Some(version.split('.').take(2).collect::<Vec<_>>().join(".")),
            risk_id: Some(variant.clone()),
            branch: Some(downloadable_blender_version.branch),
            patch_url: Some(match &downloadable_blender_version.patch {
                Some(v) => format!(
                    "{}{}{}",
                    PROJECTS_BLENDER_ORG_BLENDER_BLENDER_PULLS,
                    FORWARD_SLASH_DELIMETER,
                    v.replace(PR, "")
                ),
                None => String::new(),
            }),
            patch: downloadable_blender_version.patch,
            hash_url: Some(format!(
                "{}{}{}",
                PROJECTS_BLENDER_ORG_BLENDER_BLENDER_COMMIT,
                FORWARD_SLASH_DELIMETER,
                downloadable_blender_version.hash
            )),
            hash: Some(downloadable_blender_version.hash),
            platform: Some(downloadable_blender_version.platform),
            architecture: Some(downloadable_blender_version.architecture),
            bitness: downloadable_blender_version.bitness,
            file_mtime: downloadable_blender_version.file_mtime,
            file_name: Some(downloadable_blender_version.file_name),
            file_size: downloadable_blender_version.file_size,
            file_extension: Some(downloadable_blender_version.file_extension),
            release_cycle: Some(infer_release_cycle(&variant, &version, &downloadable_blender_version.release_cycle)),
            checksum: Some(downloadable_blender_version.checksum),
            installation_directory_path: parent_dir.to_string_lossy().to_string(),
            executable_file_path: Some(executable_file_path.to_string_lossy().to_string()),
            blender_installation_location_id: blender_installation_location_id,
            download_status_type_id: download_status_type_completed.id,
            created: chrono::Utc::now().to_rfc3339(),
            modified: chrono::Utc::now().to_rfc3339(),
        };
        match bvr.insert(&b).await {
            Ok(_) => Ok(()),
            Err(e) => {
                return Err(format!("Failed insert_blender_version: {:?}", e));
            }
        }
    }

    async fn refresh_blender_version(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        mut blender_version: BlenderVersion,
        blender_installation_location_id: String,
    ) -> Result<(), String> {
        let bvr = state.blender_version_repository();
        let bilr = state.blender_installation_location_repository();
        let mut blender_installation_locations = match bilr
            .fetch(Some(blender_installation_location_id), None, None, None)
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed refresh_blender_version: {:?}", e)),
        };
        if blender_installation_locations.is_empty() {
            return Err(format!(
                "Failed refresh_blender_version"
            ));
        }
        let blender_installation_location = blender_installation_locations.remove(0);
        let path = format!(
            "{}/{}{}",
            blender_installation_location.directory_path, BLENDERBASE_DOWNLOAD_DATA, ".json"
        );
        let download_data_file_path = std::path::Path::new(path.as_str());
        let downloadable_blender_version: DownloadableBlenderVersion;
        if download_data_file_path.exists() {
            let download_data_file = match std::fs::File::open(download_data_file_path) {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed refresh_blender_version: {:?}", e)),
            };
            downloadable_blender_version = match serde_json::from_reader(download_data_file) {
                Ok(v) => v,
                Err(e) => {
                    return Err(format!(
                        "Failed refresh_blender_version: invalid download data file {}: {:?}",
                        path, e
                    ))
                }
            };
            blender_version = BlenderVersion {
                url: Some(downloadable_blender_version.url),
                app: Some(downloadable_blender_version.app),
                version: Some(downloadable_blender_version.version.clone()),
                series: Some(
                    downloadable_blender_version
                        .version
                        .split('.')
                        .take(2)
                        .collect::<Vec<_>>()
                        .join("."),
                ),
                risk_id: Some(downloadable_blender_version.risk_id),
                branch: Some(downloadable_blender_version.branch),
                patch_url: Some(match &downloadable_blender_version.patch {
                    Some(v) => format!(
                        "{}{}{}",
                        PROJECTS_BLENDER_ORG_BLENDER_BLENDER_PULLS,
                        FORWARD_SLASH_DELIMETER,
                        v.replace(PR, "")
                    ),
                    None => String::new(),
                }),
                patch: downloadable_blender_version.patch,
                hash_url: Some(format!(
                    "{}{}{}",
                    PROJECTS_BLENDER_ORG_BLENDER_BLENDER_COMMIT,
                    FORWARD_SLASH_DELIMETER,
                    downloadable_blender_version.hash
                )),
                hash: Some(downloadable_blender_version.hash),
                platform: Some(downloadable_blender_version.platform),
                architecture: Some(downloadable_blender_version.architecture),
                bitness: downloadable_blender_version.bitness,
                file_mtime: downloadable_blender_version.file_mtime,
                file_name: Some(downloadable_blender_version.file_name),
                file_size: downloadable_blender_version.file_size,
                file_extension: Some(downloadable_blender_version.file_extension),
                release_cycle: Some(downloadable_blender_version.release_cycle),
                checksum: Some(downloadable_blender_version.checksum),
                ..blender_version
            };
        }
        match bvr.update(&blender_version).await {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("Failed refresh_blender_version: {}", e)),
        }
    }

    async fn update_blender_version(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_version: BlenderVersion,
    ) -> Result<(), String> {
        let r = state.blender_version_repository();
        match r.update(&blender_version).await {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("Failed update_blender_version: {}", e)),
        }
    }

    async fn set_blender_version_as_default(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
    ) -> Result<(), String> {
        let r = state.blender_version_repository();
        let blender_versions = match r.fetch(None, None, None, None, None).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed set_blender_version_as_default: {:?}", e)),
        };
        if blender_versions.is_empty() {
            return Err(format!(
                "No Blender versions installed. Can't set a default Blender version"
            ));
        }
        for mut bv in blender_versions {
            let new_default = match &id {
                Some(v) => bv.id.eq(v),
                None => false,
            };
            if bv.is_default != new_default {
                bv.is_default = new_default;
                match r.update(&bv).await {
                    Ok(_) => {}
                    Err(e) => return Err(format!("Failed set_blender_version_as_default: {:?}", e)),
                }
            }
        }
        Ok(())
    }

    async fn set_default_blender_version(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        let bvr = state.blender_version_repository();
        let blender_versions = match bvr.fetch(None, None, None, None, None).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed set_default_blender_version: {}", e)),
        };
        if blender_versions.is_empty() {
            return Ok(());
        }
        if let Some(_) = blender_versions.iter().find(|e| e.is_default == true) {
            return Ok(());
        }
        let blender_version = match blender_versions.iter().max_by_key(|x| match &x.version {
            Some(v) => Self::parse_version(v),
            None => (0, 0, 0),
        }) {
            Some(v) => v.clone(),
            None => return Err("Failed set_default_blender_version: No versions found".to_string()),
        };
        let mut blender_version = blender_version;
        blender_version.is_default = true;
        match bvr.update(&blender_version).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed set_default_blender_version: {}", e)),
        }
    }

    async fn fetch_blender_versions(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
        limit: Option<i64>,
        is_defeault: Option<bool>,
        executable_file_path: Option<String>,
        series: Option<String>,
        order: &str,
        download_status_types: Option<Vec<String>>,
    ) -> Result<Vec<BlenderVersion>, String> {
        let blender_version_repository = state.blender_version_repository();
        let download_status_type_repository = state.download_status_type_repository();
        let order = match OrderKind::from_str(order) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("{:?}", e));
            }
        };
        let download_status_kinds = match download_status_type_repository
            .fetch(None, None, download_status_types)
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed fetch_blender_versions: {:?}", e)),
        };
        let mut results = match blender_version_repository
            .fetch(id, limit, is_defeault, executable_file_path, series)
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed fetch_blender_versions: {:?}", e)),
        };
        if !download_status_kinds.is_empty() {
            results.retain(|x| {
                download_status_kinds
                    .iter()
                    .any(|status| status.id == x.download_status_type_id)
            });
        }
        match order {
            // Sort ASC
            OrderKind::Asc => results.sort_by(|a, b| a.version.cmp(&b.version)),
            // Sort DESC
            OrderKind::Desc => results.sort_by(|a, b| b.version.cmp(&a.version)),
        }
        Ok(results)
    }

    async fn update_blender_version_download_status_type(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        mut blender_version: BlenderVersion,
        download_status_type: &str,
    ) -> Result<(), String> {
        let blender_version_repository = state.blender_version_repository();
        let download_status_kind = match DownloadStatusKind::from_str(download_status_type) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("Failed update_blender_version_download_status_type: {:?}", e));
            }
        };
        let download_status_type_repository = state.download_status_type_repository();
        let mut download_status_types = match download_status_type_repository
            .fetch(None, None, Some(vec![download_status_kind.to_string()]))
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed update_blender_version_download_status_type: {:?}", e)),
        };
        if download_status_types.is_empty() {
            return Err(format!("Failed update_blender_version_download_status_type: Failed to find download_status_types"));
        }
        let new_download_status_type: crate::database::DownloadStatusType =
            download_status_types.remove(0);
        blender_version.download_status_type_id = new_download_status_type.id;
        match blender_version_repository.update(&blender_version).await {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("Failed update_blender_version_download_status_type: {}", e)),
        }
    }

    async fn update_install_blender_build_type(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
        _code: String,
    ) -> Result<(), String> {
        // let repository = state.blender_version_build_type_repository();
        // let results = match repository.fetch(None, None, None).await {
        //     Ok(v) => v,
        //     Err(err) => {
        //         return Err(format!("Failed to fetch Blender repo paths: {:?}", err));
        //     }
        // };
        // for mut entry in results {
        //     if entry.code == code {
        //         entry.is_default = 1;
        //         match repository.update(&entry).await {
        //             Ok(_) => {}
        //             Err(err) => {
        //                 return Err(format!("Failed to update Blender repo paths: {:?}", err));
        //             }
        //         }
        //     } else {
        //         entry.is_default = 0;
        //         match repository.update(&entry).await {
        //             Ok(_) => {}
        //             Err(err) => {
        //                 return Err(format!("Failed to update Blender repo paths: {:?}", err));
        //             }
        //         }
        //     }
        // }
        Ok(())
    }

    async fn delete_blender_version(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
    ) -> Result<(), String> {
        let r = state.blender_version_repository();
        let confirmation = instance_native_ask_dialog_window(
            app.clone(),
            format!("Are you sure you want to delete this installed Blender version?"),
            tauri_plugin_dialog::MessageDialogKind::Warning,
        );
        if confirmation == false {
            return Ok(());
        }
        let mut a = match r.fetch(Some(id.clone()), None, None, None, None).await {
            Ok(v) => v,
            Err(err) => return Err(format!("Failed delete_blender_version: {:?}", err)),
        };
        if a.is_empty() {
            return Err(format!(
                "Failed delete_blender_version: Failed to find a Blender version with id {:?}. Can't delete Blender version",
                id
            ));
        }
        let b = a.remove(0);
        match delete_directory(std::path::PathBuf::from(b.installation_directory_path)).await {
            Ok(_) => {}
            Err(err) => return Err(format!("Failed delete_blender_version: {:?}", err)),
        }
        match r.delete(id).await {
            Ok(_) => Ok(()),
            Err(err) => return Err(format!("Failed delete_blender_version: {:?}", err)),
        }
    }

    async fn refresh_blender_version_details(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        ids: Vec<String>,
    ) -> Result<Vec<BlenderVersion>, String> {
        let repository = state.blender_version_repository();
        let mut result: Vec<BlenderVersion> = Vec::new();
        for id in ids {
            let mut entries = match repository.fetch(Some(id), None, None, None, None).await {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed refresh_blender_version_details: {:?}", e)),
            };
            if entries.is_empty() {
                continue;
            }
            result.push(Self::probe_and_store_details(&state, entries.remove(0)).await);
        }
        Ok(result)
    }

    async fn launch_blender_version(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
    ) -> Result<(), String> {
        let repository = state.blender_version_repository();
        let mut blender_version_list =
            match repository.fetch(Some(id), None, None, None, None).await {
                Ok(v) => v,
                Err(err) => return Err(format!("Failed launch_blender_version: {:?}", err)),
            };
        if blender_version_list.is_empty() {
            return Err(format!("Failed launch_blender_version: Failed to find a Blender version with id"));
        }
        let blender_version = blender_version_list.remove(0);
        let launch_args: Vec<String> = vec![];
        // match launch_arguments_id {
        //     Some(arg_id) => {
        //         let mut launch_argument_entry_list = match launch_argument_repository
        //             .fetch(Some(&arg_id), None, None)
        //             .await
        //         {
        //             Ok(v) => v,
        //             Err(err) => return Err(format!("Failed to fetch launch arguments: {:?}", err))
        //         };
        //         if launch_argument_entry_list.is_empty() {
        //             return Err(format!("Failed to fetch launch argument by ID"));
        //         }
        //         let entry = launch_argument_entry_list.remove(0);
        //         match launch_argument_repository.update(&entry).await {
        //             Ok(_) => {}
        //             Err(err) => return Err(format!("Failed to update launch argument: {:?}", err))
        //         }
        //         let parsed_args: Vec<String> = entry
        //             .argument_string
        //             .split_whitespace()
        //             .map(|s| s.to_string())
        //             .collect();
        //         final_launch_args.extend(parsed_args);
        //     }
        //     None => {}
        // }
        match launch_executable(
            std::path::PathBuf::from(match blender_version.executable_file_path {
                Some(v) => v,
                None => return Err(format!("Failed launch_blender_version")),
            }),
            Some(launch_args),
        ) {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("Failed launch_blender_version: {:?}", e)),
        }
    }
}

impl BlenderInstallServiceImpl {
    /// Resolves the SHA-256 a downloaded archive must match.
    ///
    /// Daily and patch builds carry their checksum in the builder.blender.org
    /// feed, which is stored on the version row. Stable and LTS releases are
    /// listed from a mirror that publishes no checksums, so the official
    /// `.sha256` file for that release is fetched from download.blender.org
    /// and the archive's file name looked up in it.
    async fn expected_archive_sha256(
        state: tauri::State<'_, AppState>,
        blender_version: &BlenderVersion,
        archive_file_path: &std::path::Path,
    ) -> Result<String, String> {
        if let Some(stored) = blender_version.checksum.as_deref().map(str::trim) {
            if !stored.is_empty() {
                if is_sha256_hex(stored) {
                    return Ok(stored.to_ascii_lowercase());
                }
                return Err(format!(
                    "Failed install_blender_version: stored checksum '{}' is not a SHA-256 digest",
                    stored
                ));
            }
        }
        let file_name = match blender_version
            .file_name
            .as_deref()
            .filter(|s| !s.trim().is_empty())
        {
            Some(v) => v.trim().to_string(),
            None => archive_file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
        };
        let version = blender_version.version.clone().unwrap_or_default();
        let series = blender_version.series.clone().unwrap_or_default();
        if version.is_empty() || series.is_empty() || file_name.is_empty() {
            return Err(String::from(
                "Failed install_blender_version: no checksum is available for this build, so it cannot be verified",
            ));
        }
        let url = format!(
            "{}Blender{}/blender-{}.sha256",
            BLENDER_ORG_RELEASE_CHECKSUM_BASE, series, version
        );
        let listing = match http_get_as_string(state, url.clone()).await {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "Failed install_blender_version: could not fetch the published checksums from {}: {}",
                    url, e
                ))
            }
        };
        match find_sha256_in_listing(&listing, &file_name) {
            Some(v) => Ok(v),
            None => Err(format!(
                "Failed install_blender_version: {} is not listed in the published checksums at {}",
                file_name, url
            )),
        }
    }

    async fn probe_and_store_details(
        state: &tauri::State<'_, AppState>,
        mut blender_version: BlenderVersion,
    ) -> BlenderVersion {
        let executable = match &blender_version.executable_file_path {
            Some(v) if !v.is_empty() => resolve_blender_console_executable(std::path::Path::new(v)),
            _ => return blender_version,
        };
        let info = match probe_blender_build_info(&executable, 30).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "refresh_blender_version_details: {}: {}",
                    blender_version.version.clone().unwrap_or_default(),
                    e
                );
                return blender_version;
            }
        };
        if !info.hash.is_empty() {
            blender_version.hash_url = Some(format!(
                "{}{}{}",
                PROJECTS_BLENDER_ORG_BLENDER_BLENDER_COMMIT, FORWARD_SLASH_DELIMETER, info.hash
            ));
            blender_version.hash = Some(info.hash);
        }
        let date = if info.commit_date.is_empty() { &info.build_date } else { &info.commit_date };
        if let Ok(d) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                blender_version.file_mtime = dt.and_utc().timestamp();
            }
        }
        if !info.branch.is_empty() {
            blender_version.branch = Some(info.branch);
        }
        if !info.cycle.is_empty() {
            blender_version.release_cycle = Some(info.cycle);
        }
        if let Err(e) = state.blender_version_repository().update(&blender_version).await {
            eprintln!("refresh_blender_version_details: could not store details: {:?}", e);
        }
        blender_version
    }

    async fn is_default(_app: AppHandle, state: tauri::State<'_, AppState>) -> Result<bool, String> {
        let blender_version_repository = state.blender_version_repository();
        let download_status_type_repository = state.download_status_type_repository();
        let mut download_status_type_entries = match download_status_type_repository
            .fetch(
                None,
                None,
                Some(vec![DownloadStatusKind::Completed.to_string()]),
            )
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed is_default: {:?}", e)),
        };
        if download_status_type_entries.is_empty() {
            return Err(format!("Failed is_default: Failed to get download status type entries"));
        }
        let completeted = download_status_type_entries.remove(0);
        let blender_version_entries: Vec<BlenderVersion> = match blender_version_repository
            .fetch(None, None, None, None, None)
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed is_default: {:?}", e)),
        };
        if blender_version_entries.is_empty() {
            return Ok(true);
        }
        if let Some(_) = blender_version_entries.iter().find(|e| e.is_default) {
            return Ok(false);
        }
        if let Some(_) = blender_version_entries
            .iter()
            .find(|e| e.download_status_type_id == completeted.id)
        {
            return Ok(false);
        }
        return Ok(true);
    }
    async fn find_existing(
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_version: &BlenderVersion,
        download_status_kind: DownloadStatusKind,
    ) -> Result<Option<BlenderVersion>, String> {
        let blender_version_repository = state.blender_version_repository();
        let download_status_type_repository = state.download_status_type_repository();
        let mut download_status_type_entries = match download_status_type_repository
            .fetch(None, None, Some(vec![download_status_kind.to_string()]))
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed find_existing: {:?}", e)),
        };
        if download_status_type_entries.is_empty() {
            return Err(format!(
                "Failed find_existing: find_existing failed to get download status type entries"
            ));
        }
        let download_status_type = download_status_type_entries.remove(0);
        let blender_version_entries = match blender_version_repository
            .fetch(None, None, None, None, None)
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed find_existing: {:?}", e)),
        };
        if let Some(v) = blender_version_entries.iter().find(|x| 
            // id
            // is_default
            // custom_name
            x.url == blender_version.url &&
            x.app == blender_version.app &&
            x.version == blender_version.version &&
            x.series == blender_version.series &&
            x.risk_id == blender_version.risk_id &&
            x.branch == blender_version.branch &&
            // x.patch_url == blender_version.patch_url &&
            x.patch == blender_version.patch &&
            // x.hash_url == blender_version.hash_url &&
            x.hash == blender_version.hash &&
            x.platform == blender_version.platform &&
            x.architecture == blender_version.architecture &&
            x.bitness == blender_version.bitness &&
            x.file_mtime == blender_version.file_mtime &&
            x.file_name == blender_version.file_name &&
            x.file_size == blender_version.file_size &&
            x.file_extension == blender_version.file_extension &&
            x.release_cycle == blender_version.release_cycle &&
            x.checksum == blender_version.checksum &&
            x.installation_directory_path == blender_version.installation_directory_path &&
            // x.executable_file_path == blender_version.executable_file_path &&
            x.blender_installation_location_id == blender_version.blender_installation_location_id &&
            x.download_status_type_id == download_status_type.id //blender_version.download_status_type_id && // TODO use arg
            // x.created == blender_version.created &&
            // x.modified == blender_version.modified
        ) {
            return Ok(Some(v.to_owned()));
        }
        return Ok(None);
    }
    fn parse_version(v: &str) -> (u64, u64, u64) {
        let mut parts = v.split('.');
        let major = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let minor = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let patch = parts.next().unwrap_or("0").parse().unwrap_or(0);
        (major, minor, patch)
    }
}

/// Release cycle of an installed build when no download data recorded one. Release archives
/// carry only the platform in their name (`blender-5.2.1-windows-x64`), so a platform word in
/// the variant slot means a release build: LTS for LTS series, stable otherwise. Daily and
/// candidate builds name their cycle in the folder (`alpha`, `beta`, `candidate`).
fn infer_release_cycle(variant: &str, version: &str, recorded: &str) -> String {
    if !recorded.trim().is_empty() {
        return recorded.to_string();
    }
    let v = variant.trim().to_lowercase();
    let platform_words = ["windows", "win", "darwin", "macos", "mac", "linux", "x64", "x86", "arm64"];
    if v.is_empty() || platform_words.contains(&v.as_str()) {
        let series = version.split('.').take(2).collect::<Vec<_>>().join(".");
        if LTS_VERSION_ARR.contains(&series.as_str()) {
            return LTS.to_string();
        }
        return STABLE.to_string();
    }
    v
}
