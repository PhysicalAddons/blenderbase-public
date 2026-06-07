export interface IBuildColumn {
    release_cycle: string,
    patch: string,
}

export interface IDownloadableBlenderVersionDTO {
    releaseBuilds: IDownloadableBlenderVersion[],
    dailyBuilds: IDownloadableBlenderVersion[],
    patchBuilds: IDownloadableBlenderVersion[],
}

export interface IDownloadableBlenderVersion {
    url: string,
    app: string,
    version: string,
    risk_id: string,
    branch: string,
    patch: string | null,
    hash: string,
    platform: string,
    architecture: string,
    bitness: number,
    file_mtime: number,
    file_name: string,
    file_size: number,
    file_extension: string,
    release_cycle: string,
    checksum: string,
}

export interface IInputValueType {
    id: number,
    code: string,
    name: string,
    description: string,
    created: string,
    modified: string,
}

export interface IBlenderVersionBuildType {
    id: string,
    text: string,
    is_default: boolean,
}

export interface IBlenderVersionDownloadBuildTypeFilter {
    id: string,
    text: string,
}

export interface IBlenderVersionInstallBuildTypeFilter {
    id: string,
    text: string,
}

export interface IBlenderInstallationLocation {
    id: string,
    is_default: boolean,
    full_control: boolean,
    modify: boolean,
    read_and_execute: boolean,
    list_folder_contents: boolean,
    read: boolean,
    write: boolean,
    special_permissions: boolean,
    directory_path: string,
    created_by: string | null
    created: string,
    modified: string,
}

export interface IAppSettingType {
    id: number,
    code: string,
    name: string,
    description: string,
    parent_app_setting_type_id: number,
    created: string,
    modified: string,
}

export interface IAppSetting {
    id: number,
    code: string,
    name: string,
    description: string,
    disclaimer: string | null,
    is_enabled: boolean,
    is_read_on_app_launch: boolean,
    is_read_on_app_exit: boolean,
    default_int_value: number | null,
    int_value: number | null,
    min_int_value: number | null,
    max_int_value:number | null,
    is_signed_int_value: boolean | null,
    default_text_value: string | null,
    text_value: string | null,
    min_length_text_value: number | null,
    max_length_text_value: number | null,
    regex_format_text_value: string | null,
    default_range_int_value_from: number | null,
    default_range_int_value_to: number | null,
    range_int_value_from: number | null,
    range_int_value_to: number | null,
    
    min_range_int_value_from: number | null,
    min_range_int_value_to: number | null,
    max_range_int_value_from: number | null,
    max_range_int_value_to: number | null,

    is_signed_range_int_value_from: boolean | null,
    is_signed_range_int_value_to: boolean | null,
    default_range_text_value_from: string | null,
    default_range_text_value_to: string | null,
    range_text_value_from: string | null,
    range_text_value_to: string | null,
    min_length_range_text_value_from: number | null,
    max_length_range_text_value_from: number | null,
    min_length_range_text_value_to: number | null,
    max_length_range_text_value_to: number | null,
    regex_format_range_text_value_from: string | null,
    regex_format_range_text_value_to: string | null,
    control_title: string | null,
    parent_app_setting_id: string | null,
    app_setting_type_id: number,
    app_setting_action_type_id: number,
    measurement_unit_type_id: number | null,
    input_value_type_id: number,
    created: string,
    modified: string,
}

export interface IRangeValues {
    range_int_value_from: number | null,
    range_int_value_to: number | null, 
    range_text_value_from: string | null,
    range_text_value_to: string | null,
}

export interface IBlendFile {
    id: string,
    file_path: string,
    file_name: string,
    file_size: number,
    created_datetime: string,
    modified_datetime: string,
    accessed_datetime: string,
    last_used_blender_version_id: string,
    created: string,
    modified: string,
}

export interface IBlenderSeries {
    id: string,
    is_collapsed: boolean,
    series: string,
    config_directory_path: string,
    created: string,
    modified: string,
}

export interface IBlendFileBlenderSeries {
    blend_file_id: string,
    blender_series_id: string,
    created: string,
    modified: string,
}

export interface IDownloadFileRef {
    build: IDownloadableBlenderVersion,
    url: string,
    fileName: string,
    buttonId: string
}

export interface IDownloadDataSelectedEvent {
    blenderInstallationLocation: IBlenderInstallationLocation,
    blenderVersion: IBlenderVersion
}

export interface IBlenderVersion {
    id: string,
    is_default: boolean,
    custom_name: string | null,
    url: string | null,
    app: string | null,
    version: string | null,
    series: string | null,
    risk_id: string | null, // variant_type: string | null,
    branch: string | null,
    patch: string | null,
    patch_url: string | null,
    hash: string | null,
    hash_url: string | null,
    platform: string | null,
    architecture: string | null,
    bitness: number,
    file_mtime: number,
    file_name: string | null,
    file_size: number,
    file_extension: string | null,
    release_cycle: string | null,
    checksum: string | null,
    installation_directory_path: string,
    executable_file_path: string | null,
    blender_installation_location_id: string,
    download_status_type_id: number
    created: string,
    modified: string,
}
