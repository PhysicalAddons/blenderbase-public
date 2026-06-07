use tauri::AppHandle;

use crate::{
    core::{
        BlendFileServiceImpl, BlenderInstallServiceImpl, BlenderInstallationLocationServiceImpl,
        TBlendFileService, TBlenderInstallService, TBlenderInstallationLocationService,
    },
    database::{
        AddonRepository, AppSettingActionTypeRepository, AppSettingRepository,
        AppSettingTypeRepository, AppState, BlendFileBlenderSeriesRepository, BlendFileRepository,
        BlenderInstallationLocationRepository, BlenderSeriesRepository,
        BlenderVersionBuildTypeRepository, BlenderVersionRepository, DownloadStatusTypeRepository,
        InputValueTypeRepository, MeasurementUnitTypeRepository,
    },
};

impl AppState {
    pub fn addon_repository(&self) -> AddonRepository<'_> {
        AddonRepository::new(&self.pool)
    }
    pub fn app_setting_action_type_repository(&self) -> AppSettingActionTypeRepository<'_> {
        AppSettingActionTypeRepository::new(&self.pool)
    }
    pub fn app_setting_repository(&self) -> AppSettingRepository<'_> {
        AppSettingRepository::new(&self.pool)
    }
    pub fn app_setting_type_repository(&self) -> AppSettingTypeRepository<'_> {
        AppSettingTypeRepository::new(&self.pool)
    }
    pub fn blend_file_blender_series_repository(&self) -> BlendFileBlenderSeriesRepository<'_> {
        BlendFileBlenderSeriesRepository::new(&self.pool)
    }
    pub fn blend_file_repository(&self) -> BlendFileRepository<'_> {
        BlendFileRepository::new(&self.pool)
    }
    pub fn blender_installation_location_repository(
        &self,
    ) -> BlenderInstallationLocationRepository<'_> {
        BlenderInstallationLocationRepository::new(&self.pool)
    }
    pub fn blender_series_repository(&self) -> BlenderSeriesRepository<'_> {
        BlenderSeriesRepository::new(&self.pool)
    }
    pub fn blender_version_build_type_repository(&self) -> BlenderVersionBuildTypeRepository<'_> {
        BlenderVersionBuildTypeRepository::new(&self.pool)
    }
    pub fn blender_version_repository(&self) -> BlenderVersionRepository<'_> {
        BlenderVersionRepository::new(&self.pool)
    }
    pub fn download_status_type_repository(&self) -> DownloadStatusTypeRepository<'_> {
        DownloadStatusTypeRepository::new(&self.pool)
    }
    pub fn input_value_type_repository(&self) -> InputValueTypeRepository<'_> {
        InputValueTypeRepository::new(&self.pool)
    }
    pub fn measurement_unit_type_repository(&self) -> MeasurementUnitTypeRepository<'_> {
        MeasurementUnitTypeRepository::new(&self.pool)
    }
}

pub trait TAppState {
    async fn app_db_init(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
}

pub struct AppStateImpl;

impl TAppState for AppStateImpl {
    async fn app_db_init(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        match BlenderInstallationLocationServiceImpl
            .refresh_blender_installation_locations(app.clone(), state.clone())
            .await
        {
            Ok(_) => {}
            Err(e) => return Err(format!("Failed to refresh blender installation locations: {:?}", e)),
        }
        match BlenderInstallServiceImpl
            .refresh_blender_versions_init(app.clone(), state.clone())
            .await
        {
            Ok(_) => {}
            Err(e) => return Err(format!("Failed to refresh blender versions{:?}", e)),
        }
        match BlendFileServiceImpl
            .refresh_blender_series(app.clone(), state.clone())
            .await
        {
            Ok(_) => {}
            Err(e) => return Err(format!("Failed to refresh blender series: {:?}", e)),
        }
        match BlendFileServiceImpl
            .refresh_blend_files(app.clone(), state.clone())
            .await
        {
            Ok(_) => {}
            Err(e) => return Err(format!("Failed to refresh blend files: {:?}", e)),
        }
        Ok(())
    }
}
