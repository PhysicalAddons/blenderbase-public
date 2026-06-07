use tauri::AppHandle;

use crate::AppState;

pub trait TAddonService {
    async fn insert_addon(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn insert_and_refresh_addons(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn update_addon(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn fetch_addons(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn delete_addon(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn toggle_addon(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn symlink_addon(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn reveal_addon_in_local_file_system(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn update_addon_filter(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
}

pub struct AddonServiceImpl;

impl TAddonService for AddonServiceImpl {
    async fn insert_addon(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        Ok(())
    }
    async fn insert_and_refresh_addons(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        Ok(())
    }
    async fn update_addon(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        Ok(())
    }
    async fn fetch_addons(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        Ok(())
    }
    async fn delete_addon(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        Ok(())
    }
    async fn toggle_addon(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        Ok(())
    }
    async fn symlink_addon(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        Ok(())
    }
    async fn reveal_addon_in_local_file_system(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        Ok(())
    }
    async fn update_addon_filter(
        &self,
        _app: AppHandle,
        _state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        Ok(())
    }
}
