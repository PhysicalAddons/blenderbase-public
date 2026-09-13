use tauri::AppHandle;

use crate::{
    core::{
        get_directory_from_file_explorer, get_permission_details, instance_native_ok_dialog_window,
        PermissionDetails,
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
    ) -> Result<(), String>;
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
}

pub struct BlenderInstallationLocationServiceImpl;

impl TBlenderInstallationLocationService for BlenderInstallationLocationServiceImpl {
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
    async fn insert_blender_installation_location(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        let blr = state.blender_installation_location_repository();
        let repo_directory_path_option = match get_directory_from_file_explorer(app.clone()).await {
            Ok(v) => v,
            Err(_) => return Ok(()),
        };
        let repo_directory_path = match repo_directory_path_option {
            Some(v) => v,
            None => return Ok(()),
        };
        let results = match blr
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
            return Ok(());
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
            Ok(_) => Ok(()),
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
        match is_default {
            Some(v) => {
                if results.is_empty() && v == true {
                    instance_native_ok_dialog_window(
                        app.clone(),
                        format!("Please set a default Blender installation location in the Settings tab"),
                        tauri_plugin_dialog::MessageDialogKind::Error,
                    )
                    .await;
                    return Err(format!("'Missing default Blender installation location"));
                }
            }
            None => {}
        }
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
