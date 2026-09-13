-- Reverse of the up migration: rebuild the table with `is_symbolic_link`
-- declared TEXT again, as in the initial schema.
CREATE TABLE addon_rebuilt (
    id TEXT PRIMARY KEY NOT NULL,
    is_enabled INTEGER NOT NULL DEFAULT 0,
    is_symbolic_link TEXT NOT NULL DEFAULT 0,
    main_python_file_path TEXT NOT NULL,
    installation_directory TEXT NOT NULL,
    variant_type TEXT NULL,
    functional_name TEXT NULL,
    name TEXT NULL,
    author TEXT NULL,
    version TEXT NULL,
    blender_version TEXT NULL,
    location TEXT NULL,
    description TEXT NULL,
    warning TEXT NULL,
    documentation_url TEXT NULL,
    tracker_url TEXT NULL,
    support TEXT NULL,
    category TEXT NULL,
    parent_blender_version_id TEXT NULL,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_blender_version_id) REFERENCES blender_version(id) ON DELETE CASCADE
);

INSERT INTO addon_rebuilt SELECT * FROM addon;

DROP TABLE addon;
ALTER TABLE addon_rebuilt RENAME TO addon;

CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_addon_path_per_blender_version
    ON addon(main_python_file_path, parent_blender_version_id);
