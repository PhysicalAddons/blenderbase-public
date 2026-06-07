use std::str::FromStr;

use tauri::AppHandle;

use crate::{
    core::{launch_executable, open_in_file_explorer, OrderKind, BLENDER, BLENDER_FOUNDATION},
    database::{BlendFile, BlendFileBlenderSeries, BlenderSeries, BlenderVersion},
    AppState,
};

pub trait TBlendFileService {
    async fn refresh_blender_series(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn refresh_blend_files(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn fetch_blend_files(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
        limit: Option<i64>,
        file_path: Option<String>,
        blender_series_id: Option<String>,
        order: &str,
    ) -> Result<Vec<BlendFile>, String>;
    async fn fetch_blender_series(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
        limit: Option<i64>,
        blender_config_directory: Option<String>,
        is_mapped: Option<bool>,
        order: &str,
    ) -> Result<Vec<BlenderSeries>, String>;
    async fn open_blend_file(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        blend_file: BlendFile,
        blender_version: BlenderVersion,
    ) -> Result<(), String>;
    async fn reveal_in_file_explorer(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        file_path: std::path::PathBuf,
    ) -> Result<(), String>;
    async fn delete_blend_file(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
    ) -> Result<(), String>;
    async fn update_blender_series(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_series: BlenderSeries,
    ) -> Result<(), String>;
}

pub struct BlendFileServiceImpl;

impl TBlendFileService for BlendFileServiceImpl {
    async fn refresh_blender_series(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        match Self::refresh_blender_series_inner(app.clone(), state.clone()).await {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("Failed refresh_blender_series: {:?}", e)),
        }
    }
    async fn refresh_blend_files(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        match Self::refresh_blender_series_inner(app.clone(), state.clone()).await {
            Ok(_) => {},
            Err(e) => return Err(format!("Failed refresh_blender_series: {:?}", e)),
        }
        match Self::refresh_blend_files_inner(app.clone(), state.clone()).await {
            Ok(_) => {}
            Err(e) => return Err(format!("Failed refresh_blender_series: {:?}", e)),
        }
        Ok(())
    }
    async fn fetch_blend_files(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
        limit: Option<i64>,
        file_path: Option<String>,
        blender_series_id: Option<String>,
        order: &str,
    ) -> Result<Vec<BlendFile>, String> {
        let repository = state.blend_file_repository();
        let order = match OrderKind::from_str(order) {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed fetch_blend_files: {:?}", e)),
        };
        let mut results = match repository
            .fetch(id, limit, file_path, blender_series_id)
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed fetch_blend_files: {:?}", e)),
        };
        match order {
            // Sort ASC
            OrderKind::Asc => results.sort_by(|a, b| a.accessed_datetime.cmp(&b.accessed_datetime)),
            // Sort DESC
            OrderKind::Desc => {
                results.sort_by(|a, b| b.accessed_datetime.cmp(&a.accessed_datetime))
            }
        }
        Ok(results)
    }
    async fn fetch_blender_series(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<String>,
        limit: Option<i64>,
        blender_config_directory: Option<String>,
        is_mapped: Option<bool>,
        order: &str,
    ) -> Result<Vec<BlenderSeries>, String> {
        let bsr = state.blender_series_repository();
        let bvr = state.blender_version_repository();
        let order = match OrderKind::from_str(order) {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed fetch_blender_series: {:?}", e)),
        };
        let blender_series = match bsr
            .fetch(id, limit, blender_config_directory, is_mapped)
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed fetch_blender_series: {}", e)),
        };
        let mut results: Vec<BlenderSeries> = vec![];
        for bs in blender_series {
            let blender_versions = match bvr
                .fetch(None, None, None, None, Some(bs.series.clone()))
                .await
            {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed fetch_blender_series: {:?}", e)),
            };
            if blender_versions.is_empty() {
                continue;
            }
            results.push(bs);
        }
        match order {
            // Sort ASC
            OrderKind::Asc => results.sort_by(|a, b| a.series.cmp(&b.series)),
            // Sort DESC
            OrderKind::Desc => results.sort_by(|a, b| b.series.cmp(&a.series)),
        }
        Ok(results)
    }
    async fn open_blend_file(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
        blend_file: BlendFile,
        blender_version: BlenderVersion,
    ) -> Result<(), String> {
        match launch_executable(
            std::path::PathBuf::from(match blender_version.executable_file_path {
                Some(v) => v,
                None => return Err(format!("Failed open_blend_file")),
            }),
            Some(vec![blend_file.file_path]),
        ) {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("Failed open_blend_file: {:?}", e)),
        }
    }
    async fn reveal_in_file_explorer(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
        file_path: std::path::PathBuf,
    ) -> Result<(), String> {
        match open_in_file_explorer(std::path::PathBuf::from(file_path)) {
            Ok(_) => Ok(()),
            Err(e) => {
                return Err(format!(
                    "Failed reveal_in_file_explorer: {:?}",
                    e
                ));
            }
        }
    }
    async fn delete_blend_file(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
        _id: Option<String>,
    ) -> Result<(), String> {
        todo!()
    }
    async fn update_blender_series(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_series: BlenderSeries,
    ) -> Result<(), String> {
        let r = state.blender_series_repository();
        match r.update(&blender_series).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                return Err(format!("Failed delete_blend_file: {:?}", e));
            }
        };
    }
}

impl BlendFileServiceImpl {
    async fn refresh_blender_series_inner(
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        let bsr = state.blender_series_repository();
        let bvr = state.blender_version_repository();
        let existing_blender_series = match bsr.fetch(None, None, None, None).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed refresh_blender_series_inner: {}", e)),
        };
        // Delete expired entries.
        for bs in existing_blender_series {
            if !std::path::PathBuf::from(bs.config_directory_path).exists() {
                match bsr.delete(bs.id).await {
                    Ok(_) => {}
                    Err(e) => return Err(format!("Failed refresh_blender_series_inner: {}", e)),
                }
            }
        }
        // Read in new entries.
        let config_directory = match dirs::config_dir() {
            Some(v) => v,
            None => return Err(format!("Failed refresh_blender_series_inner")),
        };
        let blender_foundation_directory = config_directory.join(BLENDER_FOUNDATION).join(BLENDER);
        let directory_entries = match std::fs::read_dir(blender_foundation_directory) {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed refresh_blender_series_inner: {}", e)),
        };
        for dir in directory_entries {
            let dir_entry = match dir {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed refresh_blender_series_inner: {}", e)),
            };
            let blender_versions = match bvr
                .fetch(
                    None,
                    None,
                    None,
                    None,
                    Some(dir_entry.file_name().to_string_lossy().to_string()),
                )
                .await
            {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed refresh_blender_series_inner: {:?}", e)),
            };
            if blender_versions.is_empty() {
                continue;
            }
            // Create new series entries.
            let _blender_series = match Self::insert_blender_series(
                app.clone(),
                state.clone(),
                dir_entry.file_name().to_string_lossy().to_string(),
                dir_entry
                    .path()
                    .join("config")
                    .to_string_lossy()
                    .to_string(),
            )
            .await
            {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed refresh_blender_series_inner: {}", e)),
            };
        }
        Ok(())
    }
    async fn refresh_blend_files_inner(
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        let blend_file_repository = state.blend_file_repository();
        let blender_series_repository = state.blender_series_repository();
        let existing_entries = match blend_file_repository.fetch(None, None, None, None).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed refresh_blend_files_inner: {}", e)),
        };
        // Delete expired entries.
        for entry in existing_entries {
            if !std::path::PathBuf::from(entry.file_path).exists() {
                match blend_file_repository.delete(entry.id).await {
                    Ok(_) => {}
                    Err(e) => return Err(format!("Failed refresh_blend_files_inner: {}", e)),
                }
            }
        }
        let blender_series_entries = match blender_series_repository
            .fetch(None, None, None, None)
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed refresh_blend_files_inner: {}", e)),
        };
        for blender_series in blender_series_entries {
            let recent_files_txt_path =
                std::path::PathBuf::from(blender_series.clone().config_directory_path)
                    .join("recent-files.txt");
            if !recent_files_txt_path.exists() {
                continue;
            }
            // Read in the file.
            let recent_files_txt_content = match std::fs::read_to_string(&recent_files_txt_path) {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed refresh_blend_files_inner: {}", e)),
            };
            // Holds only the blend file paths that are confirmed to exist.
            let mut refreshed_recent_files_txt_content = String::new();
            for line in recent_files_txt_content.lines() {
                let raw_line = line.trim();
                let file_path = std::path::PathBuf::from(raw_line);
                if !file_path.exists() {
                    let mut current_entries = match blend_file_repository
                        .fetch(
                            None,
                            None,
                            Some(file_path.to_string_lossy().to_string()),
                            None,
                        )
                        .await
                    {
                        Ok(v) => v,
                        Err(e) => return Err(format!("Failed refresh_blend_files_inner: {}", e)),
                    };
                    if !current_entries.is_empty() {
                        let entry_to_remove = current_entries.remove(0);
                        match blend_file_repository.delete(entry_to_remove.id).await {
                            Ok(_) => {}
                            Err(e) => return Err(format!("Failed refresh_blend_files_inner: {}", e)),
                        }
                    }
                    continue;
                } else {
                    // Update.
                    refreshed_recent_files_txt_content.push_str(&file_path.to_string_lossy());
                    refreshed_recent_files_txt_content.push('\n');
                }
                // File exists and isnt expired. Should be mapped.
                let mut existing_entries = match blend_file_repository
                    .fetch(
                        None,
                        None,
                        Some(file_path.to_string_lossy().to_string()),
                        None,
                    )
                    .await
                {
                    Ok(v) => v,
                    Err(e) => return Err(format!("Failed refresh_blend_files_inner: {}", e)),
                };
                if existing_entries.is_empty() {
                    let blend_file = match Self::insert_blend_file(
                        app.clone(),
                        state.clone(),
                        file_path,
                    )
                    .await
                    {
                        Ok(v) => v,
                        Err(e) => return Err(format!("Failed refresh_blend_files_inner: {}", e)),
                    };
                    let _blend_file_blender_series = match Self::insert_blend_file_blender_series(
                        app.clone(),
                        state.clone(),
                        blend_file,
                        blender_series.clone(),
                    )
                    .await
                    {
                        Ok(v) => v,
                        Err(e) => return Err(format!("Failed refresh_blend_files_inner: {}", e)),
                    };
                } else {
                    let existing_blend_file = existing_entries.remove(0);
                    let _blend_file_blender_series = match Self::insert_blend_file_blender_series(
                        app.clone(),
                        state.clone(),
                        existing_blend_file,
                        blender_series.clone(),
                    )
                    .await
                    {
                        Ok(v) => v,
                        Err(e) => return Err(format!("Failed refresh_blend_files_inner: {}", e)),
                    };
                }
            }
            // Write refreshed_recent_files_txt_content to recent-files.txt.
            match std::fs::write(recent_files_txt_path, refreshed_recent_files_txt_content) {
                Ok(_) => {}
                Err(e) => return Err(format!("Failed refresh_blend_files_inner: {}", e)),
            }
        }
        Ok(())
    }
    async fn insert_blend_file(
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        file_path: std::path::PathBuf,
    ) -> Result<BlendFile, String> {
        let repository = state.blend_file_repository();
        let metadata = match std::fs::metadata(file_path.clone()) {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed insert_blend_file: {}", e)),
        };
        let creation_date = match metadata.created() {
            Ok(created) => match chrono::DateTime::<chrono::Local>::from(created) {
                date_time => Ok(date_time.format("%d.%m.%Y %H:%M").to_string()),
            },
            Err(e) => Err(format!("Failed insert_blend_file: {}", e)),
        };
        let modification_date = match metadata.modified() {
            Ok(modification) => match chrono::DateTime::<chrono::Local>::from(modification) {
                date_time => Ok(date_time.format("%d.%m.%Y %H:%M").to_string()),
            },
            Err(e) => Err(format!("Failed insert_blend_file: {}", e)),
        };
        let accessed_date = match metadata.accessed() {
            Ok(accessed) => match chrono::DateTime::<chrono::Local>::from(accessed) {
                date_time => Ok(date_time.format("%d.%m.%Y %H:%M").to_string()),
            },
            Err(e) => Err(format!("Failed insert_blend_file: {}", e)),
        };
        let entry = BlendFile {
            id: uuid::Uuid::new_v4().to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            file_name: match file_path.file_name() {
                Some(v) => v.to_string_lossy().to_string(),
                None => {
                    return Err(format!(
                        "Failed insert_blend_file"
                    ))
                }
            },
            file_size: metadata.len() as i64,
            created_datetime: match creation_date {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed insert_blend_file: {}", e)),
            },
            modified_datetime: match modification_date {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed insert_blend_file: {}", e)),
            },
            accessed_datetime: match accessed_date {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed insert_blend_file: {}", e)),
            },
            last_used_blender_version_id: None,
            created: chrono::Utc::now().to_rfc3339(),
            modified: chrono::Utc::now().to_rfc3339(),
        };
        match repository.insert(&entry).await {
            Ok(_) => Ok(entry),
            Err(e) => {
                return Err(format!("Failed insert_blend_file:  {:?}", e));
            }
        }
    }
    async fn insert_blender_series(
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        series: String,
        config_directoy_path: String,
    ) -> Result<BlenderSeries, String> {
        let repository = state.blender_series_repository();
        let entry = BlenderSeries {
            id: uuid::Uuid::new_v4().to_string(),
            is_collapsed: false,
            series: series,
            config_directory_path: config_directoy_path,
            created: chrono::Utc::now().to_rfc3339(),
            modified: chrono::Utc::now().to_rfc3339(),
        };
        match repository.insert(&entry).await {
            Ok(_) => Ok(entry),
            Err(e) => return Err(format!("Failed insert_blender_series: {:?}", e)),
        }
    }
    async fn insert_blend_file_blender_series(
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        blend_file: BlendFile,
        blender_series: BlenderSeries,
    ) -> Result<BlendFileBlenderSeries, String> {
        let repository = state.blend_file_blender_series_repository();
        let entry = BlendFileBlenderSeries {
            blend_file_id: blend_file.id,
            blender_series_id: blender_series.id,
            created: chrono::Utc::now().to_rfc3339(),
            modified: chrono::Utc::now().to_rfc3339(),
        };
        match repository.insert(&entry).await {
            Ok(_) => Ok(entry),
            Err(e) => return Err(format!("Failed insert_blend_file_blender_series: {:?}", e)),
        }
    }
}
