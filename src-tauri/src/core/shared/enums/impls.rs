use std::{fmt, str::FromStr};

use crate::core::{
    AppSettingActionKind, AppSettingCodeKind, BlenderBuildKind, DownloadStatusKind,
    InputValueCodeKind, OrderKind, ARRAY_UPPERCASE, ASC_LOWERCASE, BINARY_UPPERCASE, BIT_UPPERCASE,
    BOOLEAN_UPPERCASE, BYTE_UPPERCASE, CHANGE_ACTIVE_BLENDER_VERSION_THROUGH_TABLE_UPPERCASE,
    CHAR_UPPERCASE, CHECK_FOR_UPDATE_ON_LAUNCH_UPPERCASE, CHECK_FOR_UPDATE_UPPERCASE,
    COMPLETED_LOWERCASE, CREATE_APP_LIBRARY_DIRECTORY_AUTOMATICALLY_UPPERCASE, DAILY_LOWERCASE,
    DATETIME_UPPERCASE, DATE_UPPERCASE, DESC_LOWERCASE, DOWNLOADING_LOWERCASE,
    DOWNLOAD_BLENDER_IN_DEFAULT_INSTALLATION_LOCATION_UPPERCASE, FAILED_LOWERCASE, FLOAT_UPPERCASE,
    INIT_BLENDER_DATA_ON_FIRST_INSTANCE_LAUNCH_UPPERCASE, INPUT_BUTTON_UPPERCASE,
    INPUT_DECIMAL_UPPERCASE, INPUT_FILE_UPPERCASE, INPUT_RANGE_UPPERCASE,
    INPUT_SELECTION_UPPERCASE, INPUT_TEXT_UPPERCASE, INPUT_TOGGLE_UPPERCASE, INTEGER_UPPERCASE,
    JSON_UPPERCASE, MINIMIZE_BLENDERBASE_ON_LAUNCH_UPPERCASE, NULL_UPPERCASE, OBJECT_UPPERCASE,
    OPEN_APP_VERSION_ONLINE_REPOSITORY_UPPERCASE, OPEN_SYSTEM_CONSOLE_ON_LAUNCH_UPPERCASE,
    PATCH_LOWERCASE, PAUSED_LOWERCASE, PENDING_LOWERCASE, PROCESS_ADDON_ON_LAUNCH_UPPERCASE,
    REFRESH_BLENDER_DOWNLOAD_ON_LAUNCH_UPPERCASE, RELEASE_LOWERCASE, RESET_DATABASE_UPPERCASE,
    SET_CHECK_INTERNET_CONNECTION_TIMEOUT_UPPERCASE,
    SET_DOWNLOADABLE_BLENDER_VERSION_LIMIT_FOR_SCRAPING_UPPERCASE,
    SET_INSTALLED_BLENDER_VERSION_LIMIT_FOR_SCRAPING_UPPERCASE,
    SET_RECENT_FILES_BLENDER_SERIES_LIMIT_FOR_SCRAPING_UPPERCASE, STRING_UPPERCASE,
    TIMESTAMP_UPPERCASE, TIME_UPPERCASE, UUID_UPPERCASE, XML_UPPERCASE,
};

impl FromStr for AppSettingActionKind {
    type Err = String;

    fn from_str(input: &str) -> Result<AppSettingActionKind, String> {
        match input {
            "INPUT_BUTTON" => Ok(AppSettingActionKind::InputButton),
            "INPUT_TOGGLE" => Ok(AppSettingActionKind::InputToggle),
            "INPUT_SELECTION" => Ok(AppSettingActionKind::InputSelection),
            "INPUT_TEXT" => Ok(AppSettingActionKind::InputText),
            "INPUT_DECIMAL" => Ok(AppSettingActionKind::InputDecimal),
            "INPUT_FILE" => Ok(AppSettingActionKind::InputFile),
            "INPUT_RANGE" => Ok(AppSettingActionKind::InputRange),
            _ => Err(format!("Failed to identify AppSettingActionKind")),
        }
    }
}

impl fmt::Display for AppSettingActionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            AppSettingActionKind::InputButton => INPUT_BUTTON_UPPERCASE.to_uppercase(),
            AppSettingActionKind::InputToggle => INPUT_TOGGLE_UPPERCASE.to_uppercase(),
            AppSettingActionKind::InputSelection => INPUT_SELECTION_UPPERCASE.to_uppercase(),
            AppSettingActionKind::InputText => INPUT_TEXT_UPPERCASE.to_uppercase(),
            AppSettingActionKind::InputDecimal => INPUT_DECIMAL_UPPERCASE.to_uppercase(),
            AppSettingActionKind::InputFile => INPUT_FILE_UPPERCASE.to_uppercase(),
            AppSettingActionKind::InputRange => INPUT_RANGE_UPPERCASE.to_uppercase(),
        };
        write!(f, "{}", value)
    }
}

impl FromStr for BlenderBuildKind {
    type Err = String;

    fn from_str(input: &str) -> Result<BlenderBuildKind, String> {
        match input.to_lowercase().as_str() {
            RELEASE_LOWERCASE => Ok(BlenderBuildKind::Release),
            DAILY_LOWERCASE => Ok(BlenderBuildKind::Daily),
            PATCH_LOWERCASE => Ok(BlenderBuildKind::Patch),
            _ => Err(format!("Failed to identify BlenderBuildKind")),
        }
    }
}

impl fmt::Display for BlenderBuildKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            BlenderBuildKind::Release => RELEASE_LOWERCASE.to_uppercase(),
            BlenderBuildKind::Daily => DAILY_LOWERCASE.to_uppercase(),
            BlenderBuildKind::Patch => PATCH_LOWERCASE.to_uppercase(),
        };
        write!(f, "{}", value)
    }
}

impl FromStr for OrderKind {
    type Err = String;

    fn from_str(input: &str) -> Result<OrderKind, String> {
        match input.to_lowercase().as_str() {
            ASC_LOWERCASE => Ok(OrderKind::Asc),
            DESC_LOWERCASE => Ok(OrderKind::Desc),
            _ => Err(format!("Failed to identify DownloadStatusKind")),
        }
    }
}

impl fmt::Display for OrderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            OrderKind::Asc => ASC_LOWERCASE.to_uppercase(),
            OrderKind::Desc => DESC_LOWERCASE.to_uppercase(),
        };
        write!(f, "{}", value)
    }
}

impl FromStr for DownloadStatusKind {
    type Err = String;

    fn from_str(input: &str) -> Result<DownloadStatusKind, String> {
        match input.to_lowercase().as_str() {
            PENDING_LOWERCASE => Ok(DownloadStatusKind::Pending),
            DOWNLOADING_LOWERCASE => Ok(DownloadStatusKind::Downloading),
            PAUSED_LOWERCASE => Ok(DownloadStatusKind::Paused),
            COMPLETED_LOWERCASE => Ok(DownloadStatusKind::Completed),
            FAILED_LOWERCASE => Ok(DownloadStatusKind::Failed),
            _ => Err(format!("Failed to identify DownloadStatusKind")),
        }
    }
}

impl fmt::Display for DownloadStatusKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            DownloadStatusKind::Pending => PENDING_LOWERCASE.to_uppercase(),
            DownloadStatusKind::Downloading => DOWNLOADING_LOWERCASE.to_uppercase(),
            DownloadStatusKind::Paused => PAUSED_LOWERCASE.to_uppercase(),
            DownloadStatusKind::Completed => COMPLETED_LOWERCASE.to_uppercase(),
            DownloadStatusKind::Failed => FAILED_LOWERCASE.to_uppercase(),
        };
        write!(f, "{}", value)
    }
}

impl FromStr for AppSettingCodeKind {
    type Err = String;

    fn from_str(input: &str) -> Result<AppSettingCodeKind, String> {
        match input {
            RESET_DATABASE_UPPERCASE => Ok(AppSettingCodeKind::ResetDatabase),
            SET_CHECK_INTERNET_CONNECTION_TIMEOUT_UPPERCASE => {
                Ok(AppSettingCodeKind::SetCheckInternetConnectionTimeout)
            }
            CHECK_FOR_UPDATE_ON_LAUNCH_UPPERCASE => Ok(AppSettingCodeKind::CheckForUpdateOnLaunch),
            CHECK_FOR_UPDATE_UPPERCASE => Ok(AppSettingCodeKind::CheckForUpdate),
            OPEN_APP_VERSION_ONLINE_REPOSITORY_UPPERCASE => {
                Ok(AppSettingCodeKind::OpenAppVersionOnlineRepository)
            }
            OPEN_SYSTEM_CONSOLE_ON_LAUNCH_UPPERCASE => {
                Ok(AppSettingCodeKind::OpenSystemConsoleOnLaunch)
            }
            CHANGE_ACTIVE_BLENDER_VERSION_THROUGH_TABLE_UPPERCASE => {
                Ok(AppSettingCodeKind::ChangeActiveBlenderVersionThroughTable)
            }
            MINIMIZE_BLENDERBASE_ON_LAUNCH_UPPERCASE => {
                Ok(AppSettingCodeKind::MinimizeBlenderbaseOnLaunch)
            }
            PROCESS_ADDON_ON_LAUNCH_UPPERCASE => Ok(AppSettingCodeKind::ProcessAddonOnLaunch),
            SET_INSTALLED_BLENDER_VERSION_LIMIT_FOR_SCRAPING_UPPERCASE => {
                Ok(AppSettingCodeKind::SetInstalledBlenderVersionLimitForScraping)
            }
            INIT_BLENDER_DATA_ON_FIRST_INSTANCE_LAUNCH_UPPERCASE => {
                Ok(AppSettingCodeKind::InitBlenderDataOnFirstInstanceLaunch)
            }
            REFRESH_BLENDER_DOWNLOAD_ON_LAUNCH_UPPERCASE => {
                Ok(AppSettingCodeKind::RefreshBlenderDownloadOnLaunch)
            }
            DOWNLOAD_BLENDER_IN_DEFAULT_INSTALLATION_LOCATION_UPPERCASE => {
                Ok(AppSettingCodeKind::DownloadBlenderInDefaultInstallationLocation)
            }
            SET_DOWNLOADABLE_BLENDER_VERSION_LIMIT_FOR_SCRAPING_UPPERCASE => {
                Ok(AppSettingCodeKind::SetDownloadableBlenderVersionLimitForScraping)
            }
            SET_RECENT_FILES_BLENDER_SERIES_LIMIT_FOR_SCRAPING_UPPERCASE => {
                Ok(AppSettingCodeKind::SetRecentFilesBlenderSeriesLimitForScraping)
            }
            CREATE_APP_LIBRARY_DIRECTORY_AUTOMATICALLY_UPPERCASE => {
                Ok(AppSettingCodeKind::CreateAppLibraryDirectoryAutomatically)
            }
            _ => Err(format!("Failed to identify AppSettingKind")),
        }
    }
}

impl fmt::Display for AppSettingCodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            AppSettingCodeKind::ResetDatabase => RESET_DATABASE_UPPERCASE,
            AppSettingCodeKind::SetCheckInternetConnectionTimeout => {
                SET_CHECK_INTERNET_CONNECTION_TIMEOUT_UPPERCASE
            }
            AppSettingCodeKind::CheckForUpdateOnLaunch => CHECK_FOR_UPDATE_ON_LAUNCH_UPPERCASE,
            AppSettingCodeKind::CheckForUpdate => CHECK_FOR_UPDATE_UPPERCASE,
            AppSettingCodeKind::OpenAppVersionOnlineRepository => {
                OPEN_APP_VERSION_ONLINE_REPOSITORY_UPPERCASE
            }
            AppSettingCodeKind::OpenSystemConsoleOnLaunch => {
                OPEN_SYSTEM_CONSOLE_ON_LAUNCH_UPPERCASE
            }
            AppSettingCodeKind::ChangeActiveBlenderVersionThroughTable => {
                CHANGE_ACTIVE_BLENDER_VERSION_THROUGH_TABLE_UPPERCASE
            }
            AppSettingCodeKind::MinimizeBlenderbaseOnLaunch => {
                MINIMIZE_BLENDERBASE_ON_LAUNCH_UPPERCASE
            }
            AppSettingCodeKind::ProcessAddonOnLaunch => PROCESS_ADDON_ON_LAUNCH_UPPERCASE,
            AppSettingCodeKind::SetInstalledBlenderVersionLimitForScraping => {
                SET_INSTALLED_BLENDER_VERSION_LIMIT_FOR_SCRAPING_UPPERCASE
            }
            AppSettingCodeKind::InitBlenderDataOnFirstInstanceLaunch => {
                INIT_BLENDER_DATA_ON_FIRST_INSTANCE_LAUNCH_UPPERCASE
            }
            AppSettingCodeKind::RefreshBlenderDownloadOnLaunch => {
                REFRESH_BLENDER_DOWNLOAD_ON_LAUNCH_UPPERCASE
            }
            AppSettingCodeKind::DownloadBlenderInDefaultInstallationLocation => {
                DOWNLOAD_BLENDER_IN_DEFAULT_INSTALLATION_LOCATION_UPPERCASE
            }
            AppSettingCodeKind::SetDownloadableBlenderVersionLimitForScraping => {
                SET_DOWNLOADABLE_BLENDER_VERSION_LIMIT_FOR_SCRAPING_UPPERCASE
            }
            AppSettingCodeKind::SetRecentFilesBlenderSeriesLimitForScraping => {
                SET_RECENT_FILES_BLENDER_SERIES_LIMIT_FOR_SCRAPING_UPPERCASE
            }
            AppSettingCodeKind::CreateAppLibraryDirectoryAutomatically => {
                CREATE_APP_LIBRARY_DIRECTORY_AUTOMATICALLY_UPPERCASE
            }
        };
        write!(f, "{}", value)
    }
}

impl FromStr for InputValueCodeKind {
    type Err = String;

    fn from_str(input: &str) -> Result<InputValueCodeKind, String> {
        match input {
            NULL_UPPERCASE => Ok(InputValueCodeKind::Null),
            INTEGER_UPPERCASE => Ok(InputValueCodeKind::INTEGER),
            STRING_UPPERCASE => Ok(InputValueCodeKind::STRING),
            FLOAT_UPPERCASE => Ok(InputValueCodeKind::FLOAT),
            BYTE_UPPERCASE => Ok(InputValueCodeKind::BYTE),
            CHAR_UPPERCASE => Ok(InputValueCodeKind::CHAR),
            JSON_UPPERCASE => Ok(InputValueCodeKind::JSON),
            XML_UPPERCASE => Ok(InputValueCodeKind::XML),
            BOOLEAN_UPPERCASE => Ok(InputValueCodeKind::BOOLEAN),
            BIT_UPPERCASE => Ok(InputValueCodeKind::BIT),
            DATE_UPPERCASE => Ok(InputValueCodeKind::DATE),
            TIME_UPPERCASE => Ok(InputValueCodeKind::TIME),
            DATETIME_UPPERCASE => Ok(InputValueCodeKind::DATETIME),
            TIMESTAMP_UPPERCASE => Ok(InputValueCodeKind::TIMESTAMP),
            BINARY_UPPERCASE => Ok(InputValueCodeKind::BINARY),
            UUID_UPPERCASE => Ok(InputValueCodeKind::UUID),
            ARRAY_UPPERCASE => Ok(InputValueCodeKind::ARRAY),
            OBJECT_UPPERCASE => Ok(InputValueCodeKind::OBJECT),
            _ => Err(format!("Failed to identify InputValueCodeKind")),
        }
    }
}

impl fmt::Display for InputValueCodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            InputValueCodeKind::Null => NULL_UPPERCASE,
            InputValueCodeKind::INTEGER => INTEGER_UPPERCASE,
            InputValueCodeKind::STRING => STRING_UPPERCASE,
            InputValueCodeKind::FLOAT => FLOAT_UPPERCASE,
            InputValueCodeKind::BYTE => BYTE_UPPERCASE,
            InputValueCodeKind::CHAR => CHAR_UPPERCASE,
            InputValueCodeKind::JSON => JSON_UPPERCASE,
            InputValueCodeKind::XML => XML_UPPERCASE,
            InputValueCodeKind::BOOLEAN => BOOLEAN_UPPERCASE,
            InputValueCodeKind::BIT => BIT_UPPERCASE,
            InputValueCodeKind::DATE => DATE_UPPERCASE,
            InputValueCodeKind::TIME => TIME_UPPERCASE,
            InputValueCodeKind::DATETIME => DATETIME_UPPERCASE,
            InputValueCodeKind::TIMESTAMP => TIMESTAMP_UPPERCASE,
            InputValueCodeKind::BINARY => BINARY_UPPERCASE,
            InputValueCodeKind::UUID => UUID_UPPERCASE,
            InputValueCodeKind::ARRAY => ARRAY_UPPERCASE,
            InputValueCodeKind::OBJECT => OBJECT_UPPERCASE,
        };
        write!(f, "{}", value)
    }
}
