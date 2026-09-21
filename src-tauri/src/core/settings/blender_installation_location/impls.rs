use tauri::AppHandle;

use crate::{
    core::{
        detect_blender_installations, get_directory_from_file_explorer, get_permission_details,
        same_location_path, BlenderInstallServiceImpl, PermissionDetails, TBlenderInstallService,
    },
    database::BlenderInstallationLocation,
    AppState,
};

pub trait TBlenderInstallationLocationService {
    async fn refresh_blender_installation_locations(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn insert_blender_installation_location(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<Option<BlenderInstallationLocation>, String>;
    async fn set_blender_installation_location_as_default(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
        is_default: bool,
    ) -> Result<(), String>;
    async fn fetch_blender_installation_locations(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
        limit: Option<i64>,
        directory_path: Option<String>,
        is_default: Option<bool>,
    ) -> Result<Vec<BlenderInstallationLocation>, String>;
    async fn delete_blender_installation_location(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
    ) -> Result<(), String>;
    /// Confirms where Blender versions are installed (first-download prompt). The directory is
    /// created when missing, and becomes the default location.
    async fn confirm_blender_installation_location(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
        directory_path: String,
    ) -> Result<BlenderInstallationLocation, String>;
    /// Registers `directory_path` as an installation location without a
    /// picker: the folder is created when missing, checked for access, and
    /// becomes the default when no other default exists. An already
    /// registered folder is returned as is.
    async fn register_blender_installation_location(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        directory_path: String,
    ) -> Result<BlenderInstallationLocation, String>;
    /// Registers the folders where the Blender installer, Steam and the
    /// package managers put Blender on this machine (see
    /// [`detect_blender_installations`]) and scans the versions found there.
    /// None of them becomes the default for downloads: Program Files and its
    /// relatives are not writable. With `only_when_unregistered` the sweep is
    /// skipped as soon as any location exists (the first-launch case), so a
    /// folder the user removed is not added back on every start.
    async fn sweep_blender_installations(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        only_when_unregistered: bool,
    ) -> Result<BlenderInstallationSweep, String>;
}

/// What a sweep found and did; the frontend turns it into a status line.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct BlenderInstallationSweep {
    /// True when `only_when_unregistered` was set and a location already
    /// existed: nothing was looked at.
    pub skipped: bool,
    /// Every folder that holds Blender versions, registered now or before.
    pub locations: Vec<SweptBlenderLocation>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SweptBlenderLocation {
    pub directory_path: String,
    /// Where the folder comes from: "Program Files", "Steam", "Applications", ...
    pub label: String,
    /// Version folders the sweep found inside it.
    pub version_count: usize,
    /// True when this sweep registered the folder, false when it already was a location.
    pub is_new: bool,
}

/// Where Blender versions go when the user has not chosen a folder. The
/// first download offers it, with the option to pick another folder.
pub fn default_installation_directory() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Matches the layout existing installs already use.
        let system_drive = std::env::var_os("SystemDrive").unwrap_or_else(|| "C:".into());
        std::path::PathBuf::from(format!("{}\\", system_drive.to_string_lossy()))
            .join(crate::core::BLENDERBASE_APPS)
    }
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/"))
            .join("Applications")
            .join("Blenderbase")
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("blenderbase")
            .join("apps")
    }
}

pub struct BlenderInstallationLocationServiceImpl;

impl BlenderInstallationLocationServiceImpl {
    /// Inserts `path`, which exists, as a confirmed location after probing its
    /// access rights. It becomes the default only when `may_become_default`
    /// is set and no default exists yet.
    async fn insert_confirmed_location(
        state: &tauri::State<'_, AppState>,
        path: String,
        may_become_default: bool,
    ) -> Result<BlenderInstallationLocation, String> {
        let blr = state.blender_installation_location_repository();
        let all = match blr.fetch(None, None, None, None).await {
            Ok(v) => v,
            Err(e) => return Err(format!("{:?}", e)),
        };
        // The ACL probe spawns PowerShell and waits for it; keep that off the async runtime.
        let probe_path = path.clone();
        let permission_details: PermissionDetails =
            match tokio::task::spawn_blocking(move || get_permission_details(&probe_path)).await {
                Ok(Ok(v)) => v,
                Ok(Err(e)) => return Err(format!("{:?}", e)),
                Err(e) => return Err(format!("{:?}", e)),
            };
        let entry = BlenderInstallationLocation {
            id: uuid::Uuid::new_v4().to_string(),
            is_default: may_become_default && !all.iter().any(|l| l.is_default),
            full_control: permission_details.full_control,
            modify: permission_details.modify,
            read_and_execute: permission_details.read_and_execute,
            list_folder_contents: permission_details.list_folder_contents,
            read: permission_details.read,
            write: permission_details.write,
            special_permissions: permission_details.special_permissions,
            is_confirmed: true,
            directory_path: path,
            created_by: whoami::username().ok(),
            created: chrono::Utc::now().to_rfc3339(),
            modified: chrono::Utc::now().to_rfc3339(),
        };
        match blr.insert(&entry).await {
            Ok(_) => Ok(entry),
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}

impl TBlenderInstallationLocationService for BlenderInstallationLocationServiceImpl {
    async fn register_blender_installation_location(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        directory_path: String,
    ) -> Result<BlenderInstallationLocation, String> {
        let path = directory_path.trim().to_string();
        if path.is_empty() {
            return Err(String::from("Failed register blender installation location: no directory given"));
        }
        let blr = state.blender_installation_location_repository();
        let all = match blr.fetch(None, None, None, None).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed register blender installation location: {:?}", e)),
        };
        let has_default = all.iter().any(|l| l.is_default);
        if let Some(found) = all
            .iter()
            .find(|l| same_location_path(&l.directory_path, &path))
            .cloned()
        {
            // A folder the sweep registered is confirmed but never default. Picked
            // in the first-download prompt it has to become the default like a new
            // folder would, or the prompt would come back on every download.
            if found.is_confirmed && (found.is_default || has_default) {
                return Ok(found);
            }
            return self
                .confirm_blender_installation_location(app, state, found.id, path)
                .await;
        }
        if let Err(e) = std::fs::create_dir_all(&path) {
            return Err(format!(
                "Failed register blender installation location: could not create {}: {:?}",
                path, e
            ));
        }
        match Self::insert_confirmed_location(&state, path, !has_default).await {
            Ok(v) => Ok(v),
            Err(e) => Err(format!("Failed register blender installation location: {}", e)),
        }
    }

    async fn sweep_blender_installations(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        only_when_unregistered: bool,
    ) -> Result<BlenderInstallationSweep, String> {
        let blr = state.blender_installation_location_repository();
        let existing = match blr.fetch(None, None, None, None).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed sweep blender installations: {:?}", e)),
        };
        if only_when_unregistered && !existing.is_empty() {
            return Ok(BlenderInstallationSweep { skipped: true, locations: Vec::new() });
        }
        // Folder listings and a registry read: quick, but kept off the async
        // runtime like the other filesystem probes.
        let detected = match tokio::task::spawn_blocking(detect_blender_installations).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed sweep blender installations: {:?}", e)),
        };
        let mut locations: Vec<SweptBlenderLocation> = Vec::with_capacity(detected.len());
        let mut registered_any = false;
        for found in detected {
            let is_known = existing
                .iter()
                .any(|l| same_location_path(&l.directory_path, &found.directory_path));
            if !is_known {
                if let Err(e) =
                    Self::insert_confirmed_location(&state, found.directory_path.clone(), false).await
                {
                    return Err(format!("Failed sweep blender installations: {}", e));
                }
                registered_any = true;
            }
            locations.push(SweptBlenderLocation {
                directory_path: found.directory_path,
                label: found.label,
                version_count: found.version_dirs.len(),
                is_new: !is_known,
            });
        }
        if registered_any {
            // The same pair as `cmd_refresh_blender_versions`: scan the
            // locations, then pick a default version when none is set.
            if let Err(e) = BlenderInstallServiceImpl
                .refresh_blender_versions(app.clone(), state.clone())
                .await
            {
                return Err(format!("Failed sweep blender installations: {}", e));
            }
            if let Err(e) = BlenderInstallServiceImpl
                .set_default_blender_version(app, state)
                .await
            {
                return Err(format!("Failed sweep blender installations: {}", e));
            }
        }
        Ok(BlenderInstallationSweep { skipped: false, locations })
    }

    async fn confirm_blender_installation_location(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
        directory_path: String,
    ) -> Result<BlenderInstallationLocation, String> {
        let path = directory_path.trim().to_string();
        if path.is_empty() {
            return Err(String::from("Failed confirm blender installation location: no directory given"));
        }
        if let Err(e) = std::fs::create_dir_all(&path) {
            return Err(format!("Failed confirm blender installation location: could not create {}: {:?}", path, e));
        }
        // The ACL probe spawns PowerShell and waits for it; keep that off the async runtime.
        let probe_path = path.clone();
        let permission_details: PermissionDetails =
            match tokio::task::spawn_blocking(move || get_permission_details(&probe_path)).await {
                Ok(Ok(v)) => v,
                Ok(Err(e)) => return Err(format!("Failed confirm blender installation location: {:?}", e)),
                Err(e) => return Err(format!("Failed confirm blender installation location: {:?}", e)),
            };
        let blr = state.blender_installation_location_repository();
        if let Err(e) = blr.confirm(&id, &path, &permission_details).await {
            return Err(format!("Failed confirm blender installation location: {:?}", e));
        }
        let mut entries = match blr.fetch(Some(id), None, None, None).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed confirm blender installation location: {:?}", e)),
        };
        if entries.is_empty() {
            return Err(String::from("Failed confirm blender installation location: location not found"));
        }
        Ok(entries.remove(0))
    }

    async fn refresh_blender_installation_locations(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        let bilr = state.blender_installation_location_repository();
        let existing_blender_installation_locations = match bilr.fetch(None, None, None, None).await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed refresh blender installation locations: {}", e)),
        };
        // Delete expired entries.
        for bil in existing_blender_installation_locations {
            if !std::path::PathBuf::from(bil.directory_path).exists() {
                match bilr.delete(bil.id).await {
                    Ok(_) => {}
                    Err(e) => return Err(format!("Failed refresh blender installation locations: {}", e)),
                }
            }
        }
        Ok(())
    }
    /// Opens the folder picker and registers the chosen folder. Returns the
    /// registered location, the existing one when that folder was already
    /// known, or `None` when the user cancelled the picker (not an error).
    async fn insert_blender_installation_location(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<Option<BlenderInstallationLocation>, String> {
        let blr = state.blender_installation_location_repository();
        let repo_directory_path_option = match get_directory_from_file_explorer(app.clone()).await {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };
        let repo_directory_path = match repo_directory_path_option {
            Some(v) => v,
            None => return Ok(None),
        };
        let mut results = match blr
            .fetch(
                None,
                None,
                Some(repo_directory_path.to_string_lossy().to_string()),
                None,
            )
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed insert blender installation location: {:?}", e)),
        };
        if !results.is_empty() {
            return Ok(Some(results.remove(0)));
        }
        let existing_entries = match blr.fetch(None, None, None, None).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed insert blender installation location: {:?}", e)),
        };
        let probe_path = repo_directory_path.to_string_lossy().to_string();
        let permission_details: PermissionDetails =
            match tokio::task::spawn_blocking(move || get_permission_details(&probe_path)).await {
                Ok(Ok(v)) => v,
                Ok(Err(e)) => return Err(format!("Failed insert blender installation location: {:?}", e)),
                Err(e) => return Err(format!("Failed insert blender installation location: {:?}", e)),
            };
        let entry = BlenderInstallationLocation {
            id: uuid::Uuid::new_v4().to_string(),
            is_default: existing_entries.is_empty(),
            full_control: permission_details.full_control,
            modify: permission_details.modify,
            read_and_execute: permission_details.read_and_execute,
            list_folder_contents: permission_details.list_folder_contents,
            read: permission_details.read,
            write: permission_details.write,
            special_permissions: permission_details.special_permissions,
            is_confirmed: false,
            directory_path: repo_directory_path.to_string_lossy().to_string(),
            created_by: match whoami::username() {
                Ok(v) => Some(v.to_owned()),
                Err(e) => return Err(format!("Failed insert blender installation location: {:?}", e)),
            },
            created: chrono::Utc::now().to_rfc3339(),
            modified: chrono::Utc::now().to_rfc3339(),
        };
        match blr.insert(&entry).await {
            Ok(_) => Ok(Some(entry)),
            Err(e) => return Err(format!("Failed insert blender installation location: {:?}", e)),
        }
    }

    async fn set_blender_installation_location_as_default(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
        is_default: bool,
    ) -> Result<(), String> {
        let r = state.blender_installation_location_repository();
        let mut results = match r.fetch(id.clone(), None, None, None).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed set blender installation location as default: {:?}", e)),
        };
        if results.is_empty() {
            return Err(format!("Failed set blender installation location as default"));
        }
        let mut entry = results.remove(0);
        // TODO update with other checks to see if the path is protected.
        if entry.write == false {
            return Err(format!(
                "Failed set blender installation location as default"
            ));
        }
        if is_default == true {
            entry.is_default = false;
            match r.update(&entry).await {
                Ok(_) => Ok(()),
                Err(e) => return Err(format!("Failed set blender installation location as default: {:?}", e)),
            }
        } else {
            let results = match r.fetch(None, None, None, None).await {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed set blender installation location as default: {:?}", e)),
            };
            for mut entry in results {
                let new_default = match &id {
                    Some(v) => entry.id.eq(v),
                    None => false,
                };
                if entry.is_default != new_default {
                    entry.is_default = new_default;
                    match r.update(&entry).await {
                        Ok(_) => {}
                        Err(e) => return Err(format!("Failed set blender installation location as default: {:?}", e)),
                    }
                }
            }
            Ok(())
        }
    }

    async fn fetch_blender_installation_locations(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
        limit: Option<i64>,
        directory_path: Option<String>,
        is_default: Option<bool>,
    ) -> Result<Vec<BlenderInstallationLocation>, String> {
        let repository = state.blender_installation_location_repository();
        let mut results = match repository
            .fetch(id, limit, directory_path, is_default)
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed fetch blender installation locations: {:?}", e)),
        };
        results.sort_by(|a, b| b.directory_path.cmp(&a.directory_path));
        // An empty result is a normal answer, not an error: the UI decides
        // what to do about a missing default (it asks for a folder in place).
        let _ = app;
        Ok(results)
    }

    async fn delete_blender_installation_location(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
    ) -> Result<(), String> {
        let blender_installation_location_repository =
            state.blender_installation_location_repository();
        // ON DELETE CASCADE:
        // - installed blender versions
        //  - addons
        // ...blend_files are exempt - their ON DELETE SET NULL.
        match blender_installation_location_repository.delete(id).await {
            Ok(_) => return Ok(()),
            Err(e) => return Err(format!("Failed delete blender installation location: {:?}", e)),
        };
    }
}
