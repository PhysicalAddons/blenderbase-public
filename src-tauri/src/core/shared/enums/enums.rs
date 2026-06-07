#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum AppSettingActionKind {
    InputButton,
    InputToggle,
    InputSelection,
    InputText,
    InputDecimal,
    InputFile,
    InputRange,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum BlenderBuildKind {
    Release,
    Daily,
    Patch,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum OrderKind {
    Asc,
    Desc,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum DownloadStatusKind {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum AppSettingCodeKind {
    ResetDatabase,
    SetCheckInternetConnectionTimeout,
    CheckForUpdateOnLaunch,
    CheckForUpdate,
    OpenAppVersionOnlineRepository,
    OpenSystemConsoleOnLaunch,
    ChangeActiveBlenderVersionThroughTable,
    MinimizeBlenderbaseOnLaunch,
    ProcessAddonOnLaunch,
    SetInstalledBlenderVersionLimitForScraping,
    InitBlenderDataOnFirstInstanceLaunch,
    RefreshBlenderDownloadOnLaunch,
    DownloadBlenderInDefaultInstallationLocation,
    SetDownloadableBlenderVersionLimitForScraping,
    SetRecentFilesBlenderSeriesLimitForScraping,
    CreateAppLibraryDirectoryAutomatically,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum InputValueCodeKind {
    Null,
    INTEGER,
    STRING,
    FLOAT,
    BYTE,
    CHAR,
    JSON,
    XML,
    BOOLEAN,
    BIT,
    DATE,
    TIME,
    DATETIME,
    TIMESTAMP,
    BINARY,
    UUID,
    ARRAY,
    OBJECT,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum AddonVariantKind {
    ALL,
    COMMUNITY,
    OFFICIAL,
}

// let download_status_kinds: Vec<DownloadStatusKind> = match download_status_types {
//     Some(v) => v.into_iter().map(|x| DownloadStatusKind::from_str(&x)).collect::<Result<Vec<_>, _>>().map_err(|e| format!("{:?}", e))?,
//     None => Vec::new(),
// };
