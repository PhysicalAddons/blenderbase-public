use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use super::{
    apply::{
        apply_series, backups_directory, is_blender_running, undo_last_apply, SeriesApplyChoice, SeriesApplyReport,
        SetupApplyOptions, SetupTarget,
    },
    bundle::{
        addon_content, pack_addon, read_bundle_manifest, AddonContent, SetupBundleWriter, SETUP_BUNDLE_EXTENSION,
    },
    manifest::{
        is_identifier, is_series, SetupAddon, SetupAddonSource, SetupBlenderVersion, SetupBlob,
        SetupManifest, SetupMeta, SetupRepository, SetupSeries, SETUP_ADDON_REASON_FILES_NOT_INCLUDED,
        SETUP_ADDON_REASON_SYMLINK, SETUP_ADDON_REASON_SYSTEM_REPOSITORY, SETUP_PREFERENCES_FORMAT,
    },
    scripts::CAPTURE_SETUP_PY,
    sync::{read_sync_status, sync_file_path, SetupSyncStatus},
    transfer::{
        decrypt_file, delete_transfer, derive_keys, download_transfer, encrypt_file, generate_code, relay_url,
        upload_transfer, TransferProgress,
    },
};
use crate::{
    core::{
        extract_json_payload, py_string_literal, resolve_blender_console_executable,
        run_blender_python_with_env, ADDON_KIND_ADDON, ADDON_KIND_CORE, ADDON_KIND_EXTENSION,
        BLENDERBASE_JSON_MARKER, COM_PHYSICALADDONS_BLENDERBASE,
    },
    database::BlenderVersion,
    AppState,
};

/// Seconds one series may take to report its setup before the run is treated as hung.
const CAPTURE_TIMEOUT_SECS: u64 = 120;
/// Series read at the same time. Each is a full Blender start, so this stays small.
const CONCURRENT_CAPTURES: usize = 3;
/// Preference structs that mean the same on every computer. `filepaths` and `system` stay
/// behind: folders, GPU and memory choices belong to the machine, not to the setup.
pub(super) const PORTABLE_PREFERENCE_GROUPS: [&str; 3] = ["view", "edit", "inputs"];
/// The repository Blender reserves for extensions that ship with an installation.
const SYSTEM_REPOSITORY_SOURCE: &str = "SYSTEM";
/// Event that carries one line of progress for the status bar while a setup is saved.
pub const SETUP_PROGRESS_EVENT: &str = "setup-progress";

/// Receives one line per step of a long run. Shared across the capture tasks and the
/// blocking writer, hence the `Arc`.
pub type SetupProgress = Arc<dyn Fn(String) + Send + Sync>;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SetupExportOptions {
    /// Pack addons that were installed from a file into the bundle, so they can be restored
    /// without the original download. They are only listed otherwise.
    #[serde(default)]
    pub include_addon_files: bool,
}

/// A setup handed to the relay: the code to type on the other computer.
#[derive(Debug, Clone, Serialize)]
pub struct TransferSent {
    pub code: String,
    pub size: u64,
    pub expires: String,
}

/// What the frontend shows after an export, or before an import.
#[derive(Debug, Clone, Serialize)]
pub struct SetupBundleInfo {
    pub file_path: String,
    pub file_size: u64,
    /// The manifest's content hash: what a sync folder compares.
    pub content_hash: String,
    pub manifest: SetupManifest,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CapturedFile {
    #[serde(default)]
    name: String,
    file: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CapturedRepository {
    module: String,
    name: String,
    #[serde(default)]
    remote_url: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    needs_token: bool,
}

#[derive(Debug, Deserialize)]
struct CapturedAddon {
    module: String,
    kind: String,
    #[serde(default)]
    repo: String,
    #[serde(default)]
    package: String,
    name: String,
    #[serde(default)]
    version: String,
    enabled: bool,
    is_symlink: bool,
    #[serde(default)]
    file: String,
}

#[derive(Debug, Deserialize)]
struct CapturedSeries {
    blender_version: String,
    preferences: serde_json::Value,
    theme: Option<CapturedFile>,
    keymap: Option<CapturedFile>,
    #[serde(default)]
    repos: Vec<CapturedRepository>,
    #[serde(default)]
    addons: Vec<CapturedAddon>,
    #[serde(default)]
    warnings: Vec<String>,
}

pub trait TSetupService {
    async fn export_setup_bundle(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        file_path: String,
        options: SetupExportOptions,
    ) -> Result<SetupBundleInfo, String>;
    async fn inspect_setup_bundle(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        file_path: String,
    ) -> Result<SetupBundleInfo, String>;
    async fn apply_setup_bundle(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        file_path: String,
        options: SetupApplyOptions,
    ) -> Result<Vec<SeriesApplyReport>, String>;
    async fn undo_setup_apply(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        series: String,
    ) -> Result<usize, String>;
    async fn get_setup_sync(&self, app: AppHandle, state: tauri::State<'_, AppState>) -> Result<SetupSyncStatus, String>;
    async fn set_setup_sync_folder(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        folder_path: Option<String>,
    ) -> Result<SetupSyncStatus, String>;
    async fn save_setup_to_sync_folder(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        options: SetupExportOptions,
    ) -> Result<SetupBundleInfo, String>;
    async fn mark_setup_synced(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        content_hash: String,
    ) -> Result<SetupSyncStatus, String>;
    async fn send_setup_transfer(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        options: SetupExportOptions,
    ) -> Result<TransferSent, String>;
    async fn receive_setup_transfer(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        code: String,
    ) -> Result<SetupBundleInfo, String>;
}

pub struct SetupServiceImpl;

/// What turning one captured series into its manifest section needs besides the capture.
struct SeriesStep<'a> {
    series: &'a str,
    pack_directory: &'a Path,
    include_addon_files: bool,
    progress: &'a SetupProgress,
}

impl TSetupService for SetupServiceImpl {
    async fn export_setup_bundle(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        file_path: String,
        options: SetupExportOptions,
    ) -> Result<SetupBundleInfo, String> {
        let bundle_path = Self::bundle_path(&file_path);
        let installed = Self::installed_blender_versions(&state).await?;
        if installed.is_empty() {
            return Err(String::from("There is no installed Blender version to save a setup from"));
        }

        let work_directory =
            std::env::temp_dir().join(format!("blenderbase-setup-{}", uuid::Uuid::new_v4()));
        let app_version = app.package_info().version.to_string();
        let progress: SetupProgress = Arc::new(move |message: String| {
            let _ = app.emit(SETUP_PROGRESS_EVENT, message);
        });
        let mut blender: Vec<SetupBlenderVersion> = installed.iter().filter_map(Self::manifest_version).collect();
        blender.sort_by(|a, b| compare_versions(&a.version, &b.version));
        let targets = Self::series_targets(&installed);
        let outcome =
            Self::export_into(app_version, blender, targets, &bundle_path, &work_directory, &options, progress).await;
        let _ = std::fs::remove_dir_all(&work_directory);
        outcome
    }

    async fn inspect_setup_bundle(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
        file_path: String,
    ) -> Result<SetupBundleInfo, String> {
        let path = PathBuf::from(&file_path);
        let join = tokio::task::spawn_blocking(move || -> Result<SetupBundleInfo, String> {
            let manifest = read_bundle_manifest(&path)?;
            let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            Ok(SetupBundleInfo {
                file_path: path.to_string_lossy().to_string(),
                file_size,
                content_hash: manifest.content_hash()?,
                manifest,
                warnings: Vec::new(),
            })
        });
        match join.await {
            Ok(v) => v,
            Err(e) => Err(format!("Failed inspect_setup_bundle: {:?}", e)),
        }
    }

    async fn apply_setup_bundle(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        file_path: String,
        options: SetupApplyOptions,
    ) -> Result<Vec<SeriesApplyReport>, String> {
        let bundle_path = PathBuf::from(&file_path);
        let manifest = {
            let bundle_path = bundle_path.clone();
            tokio::task::spawn_blocking(move || read_bundle_manifest(&bundle_path))
                .await
                .map_err(|e| format!("Failed apply_setup_bundle: {:?}", e))??
        };
        if is_blender_running().await {
            return Err(String::from(
                "Close Blender first: an open Blender overwrites its preferences when it quits",
            ));
        }
        let targets = Self::series_targets(&Self::installed_blender_versions(&state).await?);
        let progress: SetupProgress = Arc::new(move |message: String| {
            let _ = app.emit(SETUP_PROGRESS_EVENT, message);
        });
        let work_directory =
            std::env::temp_dir().join(format!("blenderbase-setup-{}", uuid::Uuid::new_v4()));
        let mut reports = Vec::new();
        for (series, section) in &manifest.series {
            let choice = if options.choices.is_empty() {
                SeriesApplyChoice::everything(series)
            } else {
                match options.choices.iter().find(|c| &c.series == series) {
                    Some(c) => c.clone(),
                    None => continue,
                }
            };
            let outcome = match (targets.iter().find(|t| &t.series == series), backups_directory(series)) {
                (Some(target), Some(backups)) => {
                    apply_series(&bundle_path, section, target, &backups, &work_directory, &choice, &progress).await
                }
                (None, _) => Err(format!("Blender {} is not installed", series)),
                (_, None) => Err(String::from("Could not determine the app data directory")),
            };
            reports.push(match outcome {
                Ok(report) => report,
                Err(e) => SeriesApplyReport {
                    series: series.clone(),
                    skipped_reason: Some(e),
                    ..SeriesApplyReport::default()
                },
            });
        }
        let _ = std::fs::remove_dir_all(&work_directory);
        Ok(reports)
    }

    async fn undo_setup_apply(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        series: String,
    ) -> Result<usize, String> {
        if !is_series(&series) {
            return Err(format!("'{}' is not a Blender series", series));
        }
        if is_blender_running().await {
            return Err(String::from(
                "Close Blender first: an open Blender overwrites its preferences when it quits",
            ));
        }
        let targets = Self::series_targets(&Self::installed_blender_versions(&state).await?);
        let target = targets
            .iter()
            .find(|t| t.series == series)
            .ok_or_else(|| format!("Blender {} is not installed", series))?;
        let backups = backups_directory(&series)
            .ok_or_else(|| String::from("Could not determine the app data directory"))?;
        undo_last_apply(target, &backups).await
    }

    async fn get_setup_sync(&self, _app: AppHandle, state: tauri::State<'_, AppState>) -> Result<SetupSyncStatus, String> {
        let sync = state
            .setup_sync_repository()
            .fetch()
            .await
            .map_err(|e| format!("Failed to read the sync folder setting: {:?}", e))?;
        tokio::task::spawn_blocking(move || read_sync_status(&sync))
            .await
            .map_err(|e| format!("Failed get_setup_sync: {:?}", e))
    }

    async fn set_setup_sync_folder(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        folder_path: Option<String>,
    ) -> Result<SetupSyncStatus, String> {
        let folder_path = folder_path.unwrap_or_default().trim().to_string();
        if !folder_path.is_empty() && !Path::new(&folder_path).is_dir() {
            return Err(format!("{} is not a folder", folder_path));
        }
        state
            .setup_sync_repository()
            .set_folder(&folder_path)
            .await
            .map_err(|e| format!("Failed to save the sync folder: {:?}", e))?;
        self.get_setup_sync(app, state).await
    }

    /// Saves the setup under the folder's fixed file name and records its hash as synced,
    /// so this computer is not told about its own save.
    async fn save_setup_to_sync_folder(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        options: SetupExportOptions,
    ) -> Result<SetupBundleInfo, String> {
        let sync = state
            .setup_sync_repository()
            .fetch()
            .await
            .map_err(|e| format!("Failed to read the sync folder setting: {:?}", e))?;
        if sync.folder_path.trim().is_empty() {
            return Err(String::from("Choose a sync folder first"));
        }
        if !Path::new(&sync.folder_path).is_dir() {
            return Err(format!("The sync folder {} is not there", sync.folder_path));
        }
        let file_path = sync_file_path(&sync.folder_path).to_string_lossy().to_string();
        let info = self.export_setup_bundle(app, state.clone(), file_path, options).await?;
        state
            .setup_sync_repository()
            .mark_synced(&info.content_hash, &chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
            .await
            .map_err(|e| format!("Failed to record the sync: {:?}", e))?;
        Ok(info)
    }

    async fn mark_setup_synced(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        content_hash: String,
    ) -> Result<SetupSyncStatus, String> {
        state
            .setup_sync_repository()
            .mark_synced(&content_hash, &chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
            .await
            .map_err(|e| format!("Failed to record the sync: {:?}", e))?;
        self.get_setup_sync(app, state).await
    }

    /// Saves the setup to a temporary file, encrypts it under a fresh code and hands it to the
    /// relay. Only the code comes back; the files are removed again here.
    async fn send_setup_transfer(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        options: SetupExportOptions,
    ) -> Result<TransferSent, String> {
        let work_directory = std::env::temp_dir().join(format!("blenderbase-transfer-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&work_directory)
            .map_err(|e| format!("Could not create {}: {}", work_directory.display(), e))?;
        let outcome = Self::send_from(self, app, state, options, &work_directory).await;
        let _ = std::fs::remove_dir_all(&work_directory);
        outcome
    }

    /// Fetches the transfer for a code, decrypts it into the app's transfers folder and reads it
    /// like any setup file; the restore view opens it from there. The relay's copy is removed.
    async fn receive_setup_transfer(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        code: String,
    ) -> Result<SetupBundleInfo, String> {
        let progress: TransferProgress = Arc::new(move |message: String| {
            let _ = app.emit(SETUP_PROGRESS_EVENT, message);
        });
        progress(String::from("Checking the code…"));
        let keys = tokio::task::spawn_blocking(move || derive_keys(&code))
            .await
            .map_err(|e| format!("Failed receive_setup_transfer: {:?}", e))??;
        let directory = dirs::data_dir()
            .ok_or_else(|| String::from("Could not determine the app data directory"))?
            .join(COM_PHYSICALADDONS_BLENDERBASE)
            .join("transfers");
        std::fs::create_dir_all(&directory).map_err(|e| format!("Could not create {}: {}", directory.display(), e))?;
        let encrypted = directory.join(format!("{}.part", &keys.id[..16]));
        let relay = relay_url();
        progress(String::from("Downloading…"));
        download_transfer(&state.http_client, &relay, &keys, &encrypted, &progress).await?;
        let stamp = chrono::Local::now().format("%Y-%m-%d %H%M%S").to_string();
        let file_path = directory.join(format!("Transfer {}.bbsetup", stamp));
        progress(String::from("Decrypting…"));
        let decrypted = {
            let (key, encrypted, file_path) = (keys.key, encrypted.clone(), file_path.clone());
            tokio::task::spawn_blocking(move || decrypt_file(&key, &encrypted, &file_path))
                .await
                .map_err(|e| format!("Failed receive_setup_transfer: {:?}", e))?
        };
        let _ = std::fs::remove_file(&encrypted);
        decrypted?;
        let manifest = {
            let file_path = file_path.clone();
            tokio::task::spawn_blocking(move || read_bundle_manifest(&file_path))
                .await
                .map_err(|e| format!("Failed receive_setup_transfer: {:?}", e))??
        };
        // The file is here now; the relay does not need its copy any more.
        if let Err(e) = delete_transfer(&state.http_client, &relay, &keys).await {
            eprintln!("The relay kept its copy of the transfer: {}", e);
        }
        Ok(SetupBundleInfo {
            file_path: file_path.to_string_lossy().to_string(),
            file_size: std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0),
            content_hash: manifest.content_hash()?,
            manifest,
            warnings: Vec::new(),
        })
    }
}

impl SetupServiceImpl {
    async fn send_from(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        options: SetupExportOptions,
        work_directory: &Path,
    ) -> Result<TransferSent, String> {
        let bundle = work_directory.join("setup.bbsetup");
        self.export_setup_bundle(app.clone(), state.clone(), bundle.to_string_lossy().to_string(), options)
            .await?;
        let progress: TransferProgress = Arc::new(move |message: String| {
            let _ = app.emit(SETUP_PROGRESS_EVENT, message);
        });
        progress(String::from("Preparing the transfer…"));
        let code = generate_code();
        let keys = {
            let code = code.clone();
            tokio::task::spawn_blocking(move || derive_keys(&code))
                .await
                .map_err(|e| format!("Failed send_setup_transfer: {:?}", e))??
        };
        let encrypted = work_directory.join("setup.enc");
        {
            let (key, bundle, encrypted) = (keys.key, bundle.clone(), encrypted.clone());
            tokio::task::spawn_blocking(move || encrypt_file(&key, &bundle, &encrypted))
                .await
                .map_err(|e| format!("Failed send_setup_transfer: {:?}", e))??;
        }
        let receipt = upload_transfer(&state.http_client, &relay_url(), &keys, &encrypted, &progress).await?;
        Ok(TransferSent {
            code,
            size: receipt.size,
            expires: receipt.expires,
        })
    }
}

impl SetupServiceImpl {
    async fn export_into(
        app_version: String,
        blender: Vec<SetupBlenderVersion>,
        targets: Vec<SetupTarget>,
        bundle_path: &Path,
        work_directory: &Path,
        options: &SetupExportOptions,
        progress: SetupProgress,
    ) -> Result<SetupBundleInfo, String> {
        let mut manifest = SetupManifest::new(SetupMeta {
            created: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            app_version,
            platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            device: whoami::devicename().unwrap_or_default(),
        });
        manifest.blender = blender;

        let captured = Self::capture_every_series(targets, work_directory, &progress).await;

        let bundle_path = bundle_path.to_path_buf();
        let work_directory = work_directory.to_path_buf();
        let include_addon_files = options.include_addon_files;
        // Hashing, packing and zipping are blocking file work over what may be hundreds of megabytes.
        let join = tokio::task::spawn_blocking(move || -> Result<SetupBundleInfo, String> {
            let mut warnings: Vec<String> = Vec::new();
            let mut writer = SetupBundleWriter::create(&bundle_path)?;
            for (series, outcome) in captured {
                let captured = match outcome {
                    Ok(v) => v,
                    Err(e) => {
                        warnings.push(format!("Blender {}: {}", series, e));
                        continue;
                    }
                };
                warnings.extend(captured.warnings.iter().map(|w| format!("Blender {}: {}", series, w)));
                let pack_directory = work_directory.join(&series).join("packed");
                let step = SeriesStep {
                    series: &series,
                    pack_directory: &pack_directory,
                    include_addon_files,
                    progress: &progress,
                };
                match Self::series_section(captured, &mut writer, &step, &mut warnings) {
                    Ok(section) => {
                        manifest.series.insert(series, section);
                    }
                    Err(e) => {
                        writer.discard();
                        return Err(e);
                    }
                }
            }
            if manifest.series.is_empty() {
                writer.discard();
                return Err(match warnings.first() {
                    Some(first) => format!("No Blender configuration could be read. {}", first),
                    None => String::from("No installed Blender version has a configuration to save yet"),
                });
            }
            progress(String::from("Writing the setup file…"));
            let file_size = writer.finish(&manifest)?;
            Ok(SetupBundleInfo {
                file_path: bundle_path.to_string_lossy().to_string(),
                file_size,
                content_hash: manifest.content_hash()?,
                manifest,
                warnings,
            })
        });
        match join.await {
            Ok(v) => v,
            Err(e) => Err(format!("Failed export_setup_bundle: {:?}", e)),
        }
    }

    /// The versions of one series share its configuration folder, so the newest of them
    /// speaks for the series, both when a setup is read and when one is applied.
    fn series_targets(installed: &[BlenderVersion]) -> Vec<SetupTarget> {
        let mut newest: BTreeMap<String, SetupTarget> = BTreeMap::new();
        for version in installed {
            let Some(series) = version.series.as_deref().filter(|s| is_series(s)) else {
                continue;
            };
            let Some(executable) = version.executable_file_path.as_deref().filter(|p| !p.is_empty()) else {
                continue;
            };
            let number = version.version.clone().unwrap_or_default();
            let is_newer = newest
                .get(series)
                .map(|current| compare_versions(&number, &current.version) == std::cmp::Ordering::Greater)
                .unwrap_or(true);
            if is_newer {
                newest.insert(
                    series.to_string(),
                    SetupTarget {
                        series: series.to_string(),
                        version: number,
                        executable: resolve_blender_console_executable(Path::new(executable)),
                        user_resources: None,
                    },
                );
            }
        }
        newest.into_values().collect()
    }

    /// Runs the capture script once per series that has a configuration folder.
    async fn capture_every_series(
        targets: Vec<SetupTarget>,
        work_directory: &Path,
        progress: &SetupProgress,
    ) -> Vec<(String, Result<CapturedSeries, String>)> {
        let limit = Arc::new(tokio::sync::Semaphore::new(CONCURRENT_CAPTURES));
        let mut runs = tokio::task::JoinSet::new();
        for target in targets {
            // A series that was never started has nothing of the user's in it yet.
            let has_configuration = target
                .series_directory()
                .map(|directory| directory.join("config").is_dir())
                .unwrap_or(false);
            if !has_configuration {
                continue;
            }
            let out_directory = work_directory.join(&target.series);
            let limit = limit.clone();
            let progress = progress.clone();
            runs.spawn(async move {
                let _permit = limit.acquire_owned().await;
                progress(format!("Reading the setup of Blender {}…", target.series));
                let outcome = Self::capture_series(&target, &out_directory).await;
                (target.series, outcome)
            });
        }
        let mut captured = Vec::new();
        while let Some(joined) = runs.join_next().await {
            if let Ok(entry) = joined {
                captured.push(entry);
            }
        }
        captured.sort_by(|a, b| a.0.cmp(&b.0));
        captured
    }

    async fn capture_series(target: &SetupTarget, out_directory: &Path) -> Result<CapturedSeries, String> {
        std::fs::create_dir_all(out_directory)
            .map_err(|e| format!("Could not create {}: {}", out_directory.display(), e))?;
        let groups = format!(
            "({},)",
            PORTABLE_PREFERENCE_GROUPS
                .iter()
                .map(|g| py_string_literal(g))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let script = CAPTURE_SETUP_PY
            .replace("__MARKER__", BLENDERBASE_JSON_MARKER)
            .replace("__OUT_DIR__", &py_string_literal(&out_directory.to_string_lossy()))
            .replace("__PREFERENCE_GROUPS__", &groups);
        let stdout =
            run_blender_python_with_env(&target.executable, &script, CAPTURE_TIMEOUT_SECS, &target.envs()).await?;
        let payload = extract_json_payload(&stdout)?;
        match serde_json::Deserializer::from_str(payload.trim())
            .into_iter::<CapturedSeries>()
            .next()
        {
            Some(Ok(v)) => Ok(v),
            Some(Err(e)) => Err(format!("Blender reported an unreadable setup: {}", e)),
            None => Err(String::from("Blender reported no setup")),
        }
    }

    /// Turns what Blender reported into the manifest section, storing its files in the bundle.
    fn series_section(
        captured: CapturedSeries,
        writer: &mut SetupBundleWriter,
        step: &SeriesStep,
        warnings: &mut Vec<String>,
    ) -> Result<SetupSeries, String> {
        let mut section = SetupSeries {
            captured_with: captured.blender_version.clone(),
            ..SetupSeries::default()
        };

        let has_preferences = captured
            .preferences
            .as_object()
            .map(|groups| !groups.is_empty())
            .unwrap_or(false);
        if has_preferences {
            let json = serde_json::to_vec_pretty(&captured.preferences)
                .map_err(|e| format!("Could not store the preferences: {}", e))?;
            let (blob, size) = writer.add_blob_from_bytes(&json)?;
            section.preferences = Some(SetupBlob {
                blob,
                size,
                name: None,
                format: Some(String::from(SETUP_PREFERENCES_FORMAT)),
            });
        }
        section.theme = Self::stored_file(captured.theme, writer)?;
        section.keymap = Self::stored_file(captured.keymap, writer)?;

        // Only remote repositories travel: a local one is a folder on this computer.
        let remote: Vec<&CapturedRepository> = captured
            .repos
            .iter()
            .filter(|r| !r.remote_url.trim().is_empty() && is_identifier(&r.module))
            .collect();
        section.repositories = remote
            .iter()
            .map(|r| SetupRepository {
                module: r.module.clone(),
                name: r.name.clone(),
                url: r.remote_url.clone(),
                needs_token: r.needs_token,
            })
            .collect();

        for addon in captured.addons {
            let mut entry = SetupAddon {
                source: SetupAddonSource::Manual,
                module: addon.module.clone(),
                name: addon.name.clone(),
                version: Some(addon.version.clone()).filter(|v| !v.trim().is_empty()),
                enabled: addon.enabled,
                kind: Some(addon.kind.clone()).filter(|k| k != ADDON_KIND_CORE),
                repository: None,
                package: None,
                content_hash: None,
                content_size: None,
                file: None,
                reason: None,
            };
            let is_extension = addon.kind == ADDON_KIND_EXTENSION;
            let repository = captured.repos.iter().find(|r| r.module == addon.repo);
            if addon.kind == ADDON_KIND_CORE {
                entry.source = SetupAddonSource::Core;
            } else if addon.is_symlink {
                entry.reason = Some(String::from(SETUP_ADDON_REASON_SYMLINK));
            } else if is_extension
                && is_identifier(&addon.package)
                && remote.iter().any(|r| r.module == addon.repo)
            {
                entry.source = SetupAddonSource::Repo;
                entry.repository = Some(addon.repo.clone());
                entry.package = Some(addon.package.clone());
            } else if is_extension && repository.map(|r| r.source == SYSTEM_REPOSITORY_SOURCE).unwrap_or(false) {
                entry.reason = Some(String::from(SETUP_ADDON_REASON_SYSTEM_REPOSITORY));
            } else if addon.kind == ADDON_KIND_EXTENSION || addon.kind == ADDON_KIND_ADDON {
                entry.source = SetupAddonSource::File;
                if is_extension && is_identifier(&addon.package) {
                    entry.package = Some(addon.package.clone());
                }
                match Self::addon_files(&addon, is_extension, writer, step) {
                    Ok((content, file)) => {
                        entry.content_hash = Some(content.content_hash);
                        entry.content_size = Some(content.size);
                        if file.is_none() {
                            entry.reason = Some(String::from(SETUP_ADDON_REASON_FILES_NOT_INCLUDED));
                        }
                        entry.file = file;
                    }
                    Err(e) => {
                        warnings.push(format!("{}: {}", addon.name, e));
                        entry.source = SetupAddonSource::Manual;
                        entry.reason = Some(String::from(SETUP_ADDON_REASON_FILES_NOT_INCLUDED));
                    }
                }
            }
            section.addons.push(entry);
        }
        section.addons.sort_by(|a, b| a.module.cmp(&b.module));
        Ok(section)
    }

    fn stored_file(captured: Option<CapturedFile>, writer: &mut SetupBundleWriter) -> Result<Option<SetupBlob>, String> {
        let Some(captured) = captured else {
            return Ok(None);
        };
        let Some(file) = captured.file.filter(|f| Path::new(f).is_file()) else {
            return Ok(None);
        };
        let (blob, size) = writer.add_blob_from_file(Path::new(&file))?;
        Ok(Some(SetupBlob {
            blob,
            size,
            name: Some(captured.name).filter(|n| !n.trim().is_empty()),
            format: None,
        }))
    }

    /// The content hash is always taken, so a later save can tell whether the addon changed.
    /// The files are only packed, which is the slow part, when they go into the bundle.
    fn addon_files(
        addon: &CapturedAddon,
        is_extension: bool,
        writer: &mut SetupBundleWriter,
        step: &SeriesStep,
    ) -> Result<(AddonContent, Option<SetupBlob>), String> {
        let main_python_file = PathBuf::from(&addon.file);
        if !main_python_file.is_file() {
            return Err(String::from("its files could not be found"));
        }
        if !step.include_addon_files {
            (step.progress)(format!("Blender {}: checking {}…", step.series, addon.name));
            return Ok((addon_content(&main_python_file)?, None));
        }
        (step.progress)(format!("Blender {}: packing {}…", step.series, addon.name));
        let packed = pack_addon(&main_python_file, is_extension, step.pack_directory)?;
        let (blob, size) = writer.add_blob_from_file(&packed.file_path)?;
        let name = packed
            .file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string());
        if packed.file_path.starts_with(step.pack_directory) {
            let _ = std::fs::remove_file(&packed.file_path);
        }
        Ok((packed.content, Some(SetupBlob { blob, size, name, format: None })))
    }

    fn manifest_version(version: &BlenderVersion) -> Option<SetupBlenderVersion> {
        let number = version.version.clone().filter(|v| !v.trim().is_empty())?;
        let series = version.series.clone().filter(|s| is_series(s))?;
        Some(SetupBlenderVersion {
            version: number,
            series,
            channel: version
                .release_cycle
                .clone()
                .filter(|c| !c.trim().is_empty())
                .unwrap_or_else(|| String::from("stable")),
            branch: version.branch.clone().filter(|b| !b.trim().is_empty()),
            is_default: version.is_default,
            custom_name: version.custom_name.clone().filter(|n| !n.trim().is_empty()),
        })
    }

    /// Registered versions whose executable is on disk: a download that never finished, or a
    /// folder that was removed by hand, is not part of the setup.
    async fn installed_blender_versions(state: &tauri::State<'_, AppState>) -> Result<Vec<BlenderVersion>, String> {
        let versions = state
            .blender_version_repository()
            .fetch(None, None, None, None, None)
            .await
            .map_err(|e| format!("Failed to fetch Blender versions: {:?}", e))?;
        Ok(versions
            .into_iter()
            .filter(|v| {
                v.executable_file_path
                    .as_deref()
                    .map(|p| !p.is_empty() && Path::new(p).is_file())
                    .unwrap_or(false)
            })
            .collect())
    }

    /// The save dialog may hand back a name without the extension.
    fn bundle_path(file_path: &str) -> PathBuf {
        let path = PathBuf::from(file_path);
        let named = path
            .extension()
            .map(|e| e.to_string_lossy().eq_ignore_ascii_case(SETUP_BUNDLE_EXTENSION))
            .unwrap_or(false);
        if named {
            path
        } else {
            PathBuf::from(format!("{}.{}", file_path, SETUP_BUNDLE_EXTENSION))
        }
    }
}

/// The first argument that names an existing setup file. Anything else on the command
/// line (flags, other files) is ignored.
pub fn setup_file_argument(args: impl Iterator<Item = String>) -> Option<String> {
    args.map(PathBuf::from)
        .find(|p| {
            p.extension()
                .map(|e| e.to_string_lossy().eq_ignore_ascii_case(SETUP_BUNDLE_EXTENSION))
                .unwrap_or(false)
                && p.is_file()
        })
        .map(|p| p.to_string_lossy().to_string())
}

/// Orders `4.5.10` after `4.5.2`; anything that is not a number counts as zero.
pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let numbers = |v: &str| -> Vec<u64> {
        v.split('.')
            .map(|p| {
                p.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .unwrap_or(0)
            })
            .collect()
    };
    numbers(a).cmp(&numbers(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::run_blender_with_env;

    #[test]
    fn versions_are_ordered_by_number_not_by_text() {
        use std::cmp::Ordering::*;
        assert_eq!(compare_versions("4.5.10", "4.5.2"), Greater);
        assert_eq!(compare_versions("5.0.0", "4.5.2"), Greater);
        assert_eq!(compare_versions("5.2.1", "5.2.1"), Equal);
        assert_eq!(compare_versions("5.3.0-alpha", "5.3.0"), Equal);
    }

    /// Changes a few preferences, a theme colour and two shortcuts in an isolated user folder.
    const TWEAK_PY: &str = r#"
import bpy
p = bpy.context.preferences
p.view.show_splash = False
p.view.ui_scale = 1.2
p.inputs.use_mouse_emulate_3_button = True
p.edit.undo_steps = 64
p.inputs.walk_navigation.walk_speed = 4.5
p.themes[0].view_3d.space.gradients.high_gradient = (0.1, 0.2, 0.3)
wm = bpy.context.window_manager
bpy.utils.keyconfig_init()
wm.keyconfigs.update()
km = wm.keyconfigs.user.keymaps["3D View"]
for kmi in km.keymap_items:
    if kmi.idname == "view3d.view_selected":
        kmi.type = "F9"
        kmi.ctrl = True
        break
km.keymap_items.new("view3d.view_all", "F10", "PRESS", shift=True)
wm.keyconfigs.update()
import addon_utils
if bpy.app.version >= (4, 2, 0):
    bpy.ops.extensions.repo_refresh_all()
addon_utils.modules(refresh=True)
for module in ("bb_trip_legacy", "bl_ext.user_default.bb_trip_ext"):
    try:
        bpy.ops.preferences.addon_enable(module=module)
    except Exception as e:
        print("Blenderbase test: could not enable", module, e)
bpy.ops.wm.save_userpref()
print("__MARKER__" + '{"ok": true}')
"#;

    /// The whole round trip against a real Blender, named by `BLENDERBASE_TEST_BLENDER`, in
    /// isolated user folders: change a configuration, save it as a setup, apply the setup to
    /// an empty configuration, save that again, and compare. The user's own profile is never
    /// read or written.
    #[tokio::test]
    #[ignore = "needs an installed Blender; set BLENDERBASE_TEST_BLENDER"]
    async fn a_setup_survives_the_trip_to_an_empty_configuration() {
        let executable = std::env::var("BLENDERBASE_TEST_BLENDER").expect("BLENDERBASE_TEST_BLENDER");
        // The registered path may be the Windows launcher stub.
        let console = resolve_blender_console_executable(Path::new(&executable));
        let info = crate::core::probe_blender_build_info(&console, 30).await.unwrap();
        let series = info.version.split('.').take(2).collect::<Vec<_>>().join(".");
        let dir = std::env::temp_dir().join(format!("blenderbase-setup-trip-{}", uuid::Uuid::new_v4()));
        let target_in = |name: &str| SetupTarget {
            series: series.clone(),
            version: info.version.clone(),
            executable: console.clone(),
            user_resources: Some(dir.join(name)),
        };
        let (source, destination) = (target_in("source"), target_in("destination"));
        for target in [&source, &destination] {
            std::fs::create_dir_all(target.series_directory().unwrap().join("config")).unwrap();
        }
        // Two addons of the source: a legacy package and, from 4.2 on, an extension.
        let source_directory = source.series_directory().unwrap();
        let legacy = source_directory.join("scripts").join("addons").join("bb_trip_legacy");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(
            legacy.join("__init__.py"),
            "bl_info = {\"name\": \"BB Trip Legacy\", \"version\": (1, 0, 0), \"blender\": (3, 0, 0), \"category\": \"Development\"}\ndef register():\n    pass\ndef unregister():\n    pass\n",
        )
        .unwrap();
        let has_extensions = compare_versions(&info.version, "4.2.0") != std::cmp::Ordering::Less;
        if has_extensions {
            let extension = source_directory.join("extensions").join("user_default").join("bb_trip_ext");
            std::fs::create_dir_all(&extension).unwrap();
            std::fs::write(
                extension.join("blender_manifest.toml"),
                "schema_version = \"1.0.0\"\nid = \"bb_trip_ext\"\nversion = \"1.0.0\"\nname = \"BB Trip Extension\"\ntagline = \"Round trip test\"\nmaintainer = \"test\"\ntype = \"add-on\"\nblender_version_min = \"4.2.0\"\nlicense = [\"SPDX:GPL-3.0-or-later\"]\n",
            )
            .unwrap();
            std::fs::write(extension.join("__init__.py"), "def register():\n    pass\ndef unregister():\n    pass\n").unwrap();
        }
        let tweak = TWEAK_PY.replace("__MARKER__", BLENDERBASE_JSON_MARKER);
        run_blender_python_with_env(&console, &tweak, 120, &source.envs()).await.unwrap();
        // With the network allowed, one small extension of extensions.blender.org travels by id.
        let online = std::env::var("BLENDERBASE_TEST_ONLINE").is_ok() && has_extensions;
        if online {
            let args: Vec<String> = ["--online-mode", "--command", "extension", "install", "--sync", "--enable", "blender_org.matwerk_picker"]
                .iter()
                .map(|s| s.to_string())
                .collect();
            run_blender_with_env(&console, &args, 600, &source.envs()).await.unwrap();
        }

        let progress: SetupProgress = Arc::new(|message: String| println!("  {}", message));
        let options = SetupExportOptions { include_addon_files: true };
        let export = |target: SetupTarget, name: &'static str| {
            let (dir, progress, options) = (dir.clone(), progress.clone(), options.clone());
            async move {
                SetupServiceImpl::export_into(
                    String::from("test"),
                    Vec::new(),
                    vec![target],
                    &dir.join(format!("{}.bbsetup", name)),
                    &dir.join(format!("work-{}", name)),
                    &options,
                    progress,
                )
                .await
                .unwrap()
            }
        };
        let saved = export(source.clone(), "source").await;
        let section = saved.manifest.series.get(&series).expect("the series was captured").clone();
        assert!(section.preferences.is_some() && section.theme.is_some());
        assert!(section.keymap.is_some(), "changed shortcuts travel as a keymap");
        assert!(section.addons.iter().any(|a| a.source == SetupAddonSource::Core));
        let packed = |module: &str| section.addons.iter().find(|a| a.module == module).and_then(|a| a.file.as_ref()).is_some();
        assert!(packed("bb_trip_legacy"), "the legacy addon is packed into the bundle");
        if has_extensions {
            assert!(packed("bl_ext.user_default.bb_trip_ext"), "the extension is packed into the bundle");
        }
        if online {
            assert!(section.addons.iter().any(|a| a.source == SetupAddonSource::Repo && a.package.as_deref() == Some("matwerk_picker")));
        }
        let json = serde_json::to_string(&saved.manifest).unwrap();
        assert!(!json.contains(":\\\\") && !json.contains("/Users/") && !json.contains("/home/"), "no local path may leak into the manifest");

        let report = apply_series(
            &dir.join("source.bbsetup"),
            &section,
            &destination,
            &dir.join("backups"),
            &dir.join("work-apply"),
            &SeriesApplyChoice::everything(&series),
            &progress,
        )
        .await
        .unwrap();
        println!("{:?}", report);
        assert!(report.preferences_set >= 5, "the five changed preferences are written");
        assert!(report.preferences_skipped.is_empty() && report.warnings.is_empty(), "{:?}", report.warnings);
        assert!(report.theme_applied);
        assert_eq!(report.keymaps_applied, vec![String::from("3D View")]);
        assert!(report.addons_failed.is_empty(), "{:?}", report.addons_failed);
        let expected_installed = 1 + usize::from(has_extensions) + usize::from(online);
        assert_eq!(report.addons_installed.len(), expected_installed, "{:?}", report.addons_installed);

        let arrived = export(destination.clone(), "destination").await;
        let arrived_section = arrived.manifest.series.get(&series).unwrap();
        assert_eq!(arrived_section.preferences.as_ref().map(|b| &b.blob), section.preferences.as_ref().map(|b| &b.blob), "every portable preference arrived");
        assert_eq!(arrived_section.theme.as_ref().map(|b| &b.blob), section.theme.as_ref().map(|b| &b.blob), "the theme arrived unchanged");
        let keymap = arrived_section.keymap.as_ref().expect("the key configuration is saved again from the second computer");
        assert_eq!(keymap.name.as_deref(), Some("Blenderbase_setup"));
        let enabled = |module: &str| arrived_section.addons.iter().any(|a| a.module == module && a.enabled);
        assert!(enabled("bb_trip_legacy"), "the legacy addon is installed and enabled on the second computer");
        if has_extensions {
            assert!(enabled("bl_ext.user_default.bb_trip_ext"), "the extension is installed and enabled on the second computer");
        }
        if online {
            assert!(enabled("bl_ext.blender_org.matwerk_picker"), "the repository extension is installed by id");
        }

        // Undo puts the empty configuration back.
        let restored = undo_last_apply(&destination, &dir.join("backups")).await.unwrap();
        println!("undo restored {} files", restored);
        let presets = destination.series_directory().unwrap().join("scripts").join("presets");
        assert!(!presets.join("keyconfig").join("Blenderbase_setup.py").exists(), "undo removes the presets the setup added");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_an_existing_setup_file_counts_as_a_startup_argument() {
        let dir = std::env::temp_dir().join(format!("blenderbase-setup-arg-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("mine.BBSETUP");
        std::fs::write(&file, b"x").unwrap();
        let missing = dir.join("gone.bbsetup").to_string_lossy().to_string();
        let args = vec![String::from("--flag"), missing, String::from("notes.txt"), file.to_string_lossy().to_string()];
        assert_eq!(setup_file_argument(args.into_iter()), Some(file.to_string_lossy().to_string()));
        assert_eq!(setup_file_argument(vec![String::from("--flag")].into_iter()), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_bundle_extension_is_added_once() {
        assert_eq!(SetupServiceImpl::bundle_path("C:/x/my setup"), PathBuf::from("C:/x/my setup.bbsetup"));
        assert_eq!(SetupServiceImpl::bundle_path("C:/x/my.BBSETUP"), PathBuf::from("C:/x/my.BBSETUP"));
    }
}
