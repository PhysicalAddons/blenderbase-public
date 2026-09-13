-- Addons are cached per Blender version. Two versions of the same series share one
-- user config directory, so the same addon path may legitimately appear for both.
DROP INDEX IF EXISTS idx_unique_addon_main_python_file_path;
CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_addon_path_per_blender_version
    ON addon(main_python_file_path, parent_blender_version_id);
