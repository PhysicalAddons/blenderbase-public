DROP TRIGGER IF EXISTS trigger_delete_blend_file_blender_series_on_blender_series_delete;
DROP TRIGGER IF EXISTS trigger_delete_blend_file_blender_series_on_blend_file_delete;

DROP INDEX IF EXISTS idx_unique_blend_file_blender_series;
DROP INDEX IF EXISTS idx_unique_blender_series_config_directory_path;
DROP INDEX IF EXISTS idx_unique_blender_version_executable_file_path;
DROP INDEX IF EXISTS idx_unique_blender_version_version_download_status_type_id;
DROP INDEX IF EXISTS idx_unique_blender_installation_location_directory_path;
DROP INDEX IF EXISTS idx_unique_addon_main_python_file_path;
DROP INDEX IF EXISTS idx_unique_blend_file_path;

DROP TABLE IF EXISTS measurement_unit_type;
DROP TABLE IF EXISTS blender_version;
DROP TABLE IF EXISTS download_status_type;
DROP TABLE IF EXISTS app_setting;
DROP TABLE IF EXISTS input_value_type;
DROP TABLE IF EXISTS app_setting_action_type;
DROP TABLE IF EXISTS app_setting_type;
DROP TABLE IF EXISTS addon;
DROP TABLE IF EXISTS blend_file_blender_series;
DROP TABLE IF EXISTS blender_series;
DROP TABLE IF EXISTS blend_file;
DROP TABLE IF EXISTS blender_version_build_type;
DROP TABLE IF EXISTS blender_installation_location;