DROP INDEX IF EXISTS idx_unique_addon_path_per_blender_version;
CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_addon_main_python_file_path ON addon(main_python_file_path);
