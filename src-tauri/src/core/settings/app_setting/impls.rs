use std::str::FromStr;

use regex::Regex;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::{
    AppState, core::{
        AppSettingActionKind, AppSettingCodeKind, BLENDERBASE_APPS, BLENDERBASE_LIBRARY, GITHUB_COM_PHYSICALADDONS_BLENDERBASE_PUBLIC, InputValueCodeKind, create_directory_path, get_main_storage_device_root_path
    }, database::{AppSetting, AppSettingType, InputValueType}
};

pub trait TAppSettingsService {
    async fn app_settings_init(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String>;
    async fn fetch_app_setting(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<i64>,
        limit: Option<i64>,
        code: Option<String>,
        is_read_on_app_launch: Option<bool>,
        app_setting_type_id: Option<i64>,
    ) -> Result<Vec<AppSetting>, String>;
    async fn fetch_app_setting_type(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<i64>,
        limit: Option<i64>,
        code: Option<String>,
    ) -> Result<Vec<AppSettingType>, String>;
    async fn fetch_input_value_type(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<i64>,
        limit: Option<i64>,
        code: Option<String>,
    ) -> Result<Vec<InputValueType>, String>;
    async fn handle_setting(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        app_setting: AppSetting,
    ) -> Result<(), String>;
}

pub struct AppSettingsServiceImpl;

impl TAppSettingsService for AppSettingsServiceImpl {
    async fn app_settings_init(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        let r = state.app_setting_repository();
        let app_settings = match r.fetch(None, None, None, Some(true), None).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed app_settings_init{:?}", e)),
        };
        for app_setting in app_settings {
            let code = match AppSettingCodeKind::from_str(&app_setting.code) {
                Ok(v) => v,
                Err(_e) => {
                    continue;
                }
            };
            match code {
                AppSettingCodeKind::CheckForUpdateOnLaunch => {
                    match Self::check_for_update_on_launch().await {
                        Ok(_v) => {},
                        Err(e) => {
                            return Err(format!("Failed app_settings_init: {:?}", e));
                        }
                    };
                }
                AppSettingCodeKind::RefreshBlenderDownloadOnLaunch => {
                    match Self::refresh_blender_download_on_launch().await {
                        Ok(_v) => {},
                        Err(e) => {
                            return Err(format!("Failed app_settings_init: {:?}", e));
                        }
                    };
                }
                AppSettingCodeKind::CreateAppLibraryDirectoryAutomatically => {
                    match Self::create_app_library_directory_automatically(
                        app.clone(),
                        state.clone(),
                    )
                    .await
                    {
                        Ok(_v) => {},
                        Err(e) => {
                            return Err(format!(
                                "Failed app_settings_init: {:?}",
                                e
                            )); 
                        }
                    };
                }
                _ => {
                    return Err(format!(
                        "Failed app_settings_init: AppSettingCode {} read on app launch is not implemented",
                        code
                    ));
                }
            }
        }
        Ok(())
    }
    async fn fetch_app_setting(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<i64>,
        limit: Option<i64>,
        code: Option<String>,
        is_read_on_app_launch: Option<bool>,
        app_setting_type_id: Option<i64>,
    ) -> Result<Vec<AppSetting>, String> {
        let r = state.app_setting_repository();
        let mut results = match r
            .fetch(id, limit, code, is_read_on_app_launch, app_setting_type_id)
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed fetch_app_setting: {:?}", e)),
        };
        results.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(results)
    }
    async fn fetch_app_setting_type(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<i64>,
        limit: Option<i64>,
        code: Option<String>,
    ) -> Result<Vec<AppSettingType>, String> {
        let r = state.app_setting_type_repository();
        let mut results = match r.fetch(id, limit, code).await {
            Ok(v) => v,
            Err(e) => return Err(format!("fetch_app_setting_type: {:?}", e)),
        };
        results.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(results)
    }
    async fn fetch_input_value_type(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: Option<i64>,
        limit: Option<i64>,
        code: Option<String>,
    ) -> Result<Vec<InputValueType>, String> {
        let r = state.input_value_type_repository();
        let mut results = match r.fetch(id, limit, code).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed fetch_input_value_type: {:?}", e)),
        };
        results.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(results)
    }
    async fn handle_setting(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        app_setting: AppSetting,
    ) -> Result<(), String> {
        let asatr = state.app_setting_action_type_repository();
        let mut app_setting_action_types = match asatr
            .fetch(Some(app_setting.app_setting_action_type_id), None, None)
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed handle_setting: {}", e)),
        };
        if app_setting_action_types.is_empty() {
            return Err(format!(
                "Failed to get app_setting_action_type with Id '{}'",
                app_setting.app_setting_action_type_id
            ));
        }
        let app_setting_action_type = app_setting_action_types.remove(0);
        let action = match AppSettingActionKind::from_str(&app_setting_action_type.code) {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed handle_setting:  {:?}", e))
        };
        match action {
            AppSettingActionKind::InputButton => {
                match Self::input_button(app, state, app_setting).await {
                    Ok(v) => Ok(v),
                    Err(e) => return Err(format!("Failed handle_setting: {:?}", e)),
                }
            }
            AppSettingActionKind::InputToggle => {
                println!("handle_setting - app_setting.code -  {:?} - {:?}", app_setting.code, app_setting.int_value);
                match Self::input_toggle(app, state, app_setting).await {
                    Ok(v) => Ok(v),
                    Err(e) => return Err(format!("Failed handle_setting: {:?}", e)),
                }
            }
            AppSettingActionKind::InputSelection => {
                return Ok(());
            }
            AppSettingActionKind::InputText => {
                return Ok(());
            }
            AppSettingActionKind::InputDecimal => {
                match Self::input_decimal(app, state, app_setting).await {
                    Ok(v) => Ok(v),
                    Err(e) => return Err(format!("Failed handle_setting: {:?}", e)),
                }
            }
            AppSettingActionKind::InputFile => {
                return Ok(());
            }
            AppSettingActionKind::InputRange => {
                match Self::input_range(app, state, app_setting).await {
                    Ok(v) => Ok(v),
                    Err(e) => return Err(format!("Failed handle_setting: {:?}", e)),
                }
            }
            _ => {
                return Ok(());
            }
        }
    }
}

impl AppSettingsServiceImpl {
    async fn input_button(
        app: AppHandle,
        _state: tauri::State<'_, AppState>,
        app_setting: AppSetting,
    ) -> Result<(), String> {
        let code = match AppSettingCodeKind::from_str(&app_setting.code) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("Failed input_button: {:?}", e));
            }
        };
        match code {
            AppSettingCodeKind::ResetDatabase => {
                match Self::reset_database().await {
                    Ok(v) => return Ok(v),
                    Err(e) => {
                        return Err(format!("Failed input_button: {:?}", e));
                    }
                };
            }
            AppSettingCodeKind::CheckForUpdate => {
                match Self::check_for_update().await {
                    Ok(v) => return Ok(v),
                    Err(e) => {
                        return Err(format!("Failed input_button: {:?}", e));
                    }
                };
            }
            AppSettingCodeKind::OpenAppVersionOnlineRepository => {
                match Self::open_app_version_online_repository(app.clone()).await {
                    Ok(v) => return Ok(v),
                    Err(e) => {
                        return Err(format!("Failed input_button: {:?}", e));
                    }
                };
            }
            _ => {
                return Err(format!("Failed input_button"));
            }
        }
    }
    async fn input_toggle(
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        app_setting: AppSetting,
    ) -> Result<(), String> {
        let r = state.app_setting_repository();
        // let mut results = match repository.fetch(Some(app_setting.id), None, None, None).await {
        //     Ok(v) => v,
        //     Err(e) => {
        //         return Err(format!("Failed to fetch app settings: {:?}", e));
        //     }
        // };
        // if results.is_empty() {
        //     return Err(format!("Failed to fetch setting by ID"));
        // }
        // let mut entry = results.remove(0);
        // entry.int_value = Some(!(entry.int_value != Some(0)) as i64);
        // match repository.update(&entry).await {
        //     Ok(v) => return Ok(v),
        //     Err(e) => {
        //         return Err(format!("Failed to toggle app setting: {:?}", e));
        //     }
        // };
        match r.update(&app_setting).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                return Err(format!("Failed to toggle app setting: {:?}", e));
            }
        };
    }
    async fn input_decimal(
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        app_setting: AppSetting,
    ) -> Result<(), String> {
        let r = state.app_setting_repository();
        // let mut results = match repository.fetch(Some(app_setting.id), None, None, None).await {
        //     Ok(v) => v,
        //     Err(e) => {
        //         return Err(format!("Failed to fetch app settings: {:?}", e));
        //     }
        // };
        // if results.is_empty() {
        //     return Err(format!("Failed to fetch setting by ID"));
        // }
        // let mut entry = results.remove(0);
        // entry.int_value = app_setting.int_value;
        // match repository.update(&entry).await {
        //     Ok(v) => return Ok(v),
        //     Err(e) => {
        //         return Err(format!("Failed to toggle app setting: {:?}", e));
        //     }
        // };
        match r.update(&app_setting).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                return Err(format!("Failed to save decimal input: {:?}", e));
            }
        };
    }
    async fn input_range(
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        app_setting: AppSetting,
    ) -> Result<(), String> {
        let asr = state.app_setting_repository();
        let ivtr = state.input_value_type_repository();
        let mut input_value_types = match ivtr
            .fetch(Some(app_setting.input_value_type_id), None, None)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("Failed to fetch app settings: {:?}", e));
            }
        };
        if input_value_types.is_empty() {
            return Err(format!("Failed to fetch input value type by ID"));
        }
        let input_value_type = input_value_types.remove(0);
        let code = match InputValueCodeKind::from_str(&input_value_type.code) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("Failed to fetch app settings: {:?}", e));
            }
        };
        match code {
            InputValueCodeKind::INTEGER => {
                // TODO MIN
                if app_setting.range_int_value_from < app_setting.min_range_int_value_from {
                    return Err(format!("From value exceeds minimum value range"));
                }
                if app_setting.range_int_value_to < app_setting.min_range_int_value_to {
                    return Err(format!("To value exceeds minimum value range"));
                }
                // TODO MAX
                // TODO IS SIGNED
            }
            InputValueCodeKind::STRING => {
                let range_text_value_from = match app_setting.clone().range_text_value_from {
                    Some(v) => v,
                    None => String::new(),
                };
                let range_text_value_to = match app_setting.clone().range_text_value_to {
                    Some(v) => v,
                    None => String::new(),
                };
                if !range_text_value_from.is_empty() {
                    if let Some(min) = app_setting.min_length_range_text_value_from {
                        if range_text_value_from.chars().count() < min as usize {
                            return Err("From value is below minimum length".to_string());
                        }
                    }
                    if let Some(max) = app_setting.max_length_range_text_value_from {
                        if range_text_value_from.chars().count() > max as usize {
                            return Err("From value exceeds maximum length".to_string());
                        }
                    }
                    if let Some(regex_pattern) = &app_setting.regex_format_range_text_value_from {
                        let re = Regex::new(regex_pattern)
                            .map_err(|e| format!("Invalid regex pattern: {:?}", e))?;

                        if !re.is_match(range_text_value_from.as_str()) {
                            return Err(format!(
                                "From value '{}' does not match required format",
                                range_text_value_from
                            ));
                        }
                    }
                }
                if !range_text_value_to.is_empty() {
                    if let Some(min) = app_setting.min_length_range_text_value_to {
                        if range_text_value_to.chars().count() < min as usize {
                            return Err("To value is below minimum length".to_string());
                        }
                    }
                    if let Some(max) = app_setting.max_length_range_text_value_to {
                        if range_text_value_to.chars().count() > max as usize {
                            return Err("To value exceeds maximum length".to_string());
                        }
                    }
                    if let Some(regex_pattern) = &app_setting.regex_format_range_text_value_to {
                        let re = Regex::new(regex_pattern)
                            .map_err(|e| format!("Invalid regex pattern: {:?}", e))?;

                        if !re.is_match(range_text_value_to.as_str()) {
                            return Err(format!(
                                "To value '{}' does not match required format",
                                range_text_value_to
                            ));
                        }
                    }
                }
            }
            _ => {
                return Err(format!("Failed to identify InputValueCode"));
            }
        }
        match asr.update(&app_setting).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                return Err(format!("Failed to toggle app setting: {:?}", e));
            }
        };
    }
    async fn reset_database() -> Result<(), String> {
        Ok(())
    }
    async fn check_for_update() -> Result<(), String> {
        Ok(())
    }
    async fn open_app_version_online_repository(app: AppHandle) -> Result<(), String> {
        app.opener()
            .open_url(GITHUB_COM_PHYSICALADDONS_BLENDERBASE_PUBLIC, None::<&str>)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    async fn check_for_update_on_launch() -> Result<(), String> {
        Ok(())
    }
    async fn refresh_blender_download_on_launch() -> Result<(), String> {
        Ok(())
    }
    async fn create_app_library_directory_automatically(
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
    ) -> Result<(), String> {
        let r = state.app_setting_repository();
        let mut app_settings = match r
            .fetch(
                None,
                None,
                Some(AppSettingCodeKind::CreateAppLibraryDirectoryAutomatically.to_string()),
                None,
                None,
            )
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("{}", e)),
        };
        if app_settings.is_empty() {
            return Err(format!(
                "Failed to access setting with code '{}'",
                AppSettingCodeKind::CreateAppLibraryDirectoryAutomatically.to_string()
            ));
        }
        let app_setting = app_settings.remove(0);
        let int_value = match app_setting.int_value {
            Some(v) => v,
            None => {
                return Err(format!(
                    "Failed to access app setting int_value '{}'",
                    AppSettingCodeKind::CreateAppLibraryDirectoryAutomatically.to_string()
                ))
            }
        };
        if int_value == 0 {
            return Ok(());
        }
        let main_storage_device_root_path = match get_main_storage_device_root_path().await {
            Ok(v) => v,
            Err(e) => return Err(format!("{:?}", e)),
        };
        match create_directory_path(
            main_storage_device_root_path
                .join(BLENDERBASE_LIBRARY)
                .join(BLENDERBASE_APPS),
        )
        .await
        {
            Ok(()) => Ok(()),
            Err(e) => return Err(format!("{}", e)),
        }
    }
}
