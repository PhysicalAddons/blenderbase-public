-- `is_symbolic_link` was declared TEXT in the initial schema. With TEXT
-- affinity SQLite stores a bound boolean as the text '0' / '1', and sqlx then
-- refuses to decode the column into a Rust bool ("Rust type `bool` is not
-- compatible with SQL type `TEXT`"). SQLite cannot change a column's type in
-- place, so the table is rebuilt with the column declared INTEGER, matching
-- `is_enabled`. Existing rows are carried over; the unique index is
-- recreated because dropping the table drops its indexes.
CREATE TABLE addon_rebuilt (
    id TEXT PRIMARY KEY NOT NULL,
    is_enabled INTEGER NOT NULL DEFAULT 0,
    is_symbolic_link INTEGER NOT NULL DEFAULT 0,
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

INSERT INTO addon_rebuilt (
    id, is_enabled, is_symbolic_link, main_python_file_path, installation_directory,
    variant_type, functional_name, name, author, version, blender_version, location,
    description, warning, documentation_url, tracker_url, support, category,
    parent_blender_version_id, created, modified
)
SELECT
    id,
    is_enabled,
    CASE
        WHEN CAST(is_symbolic_link AS INTEGER) <> 0 THEN 1
        WHEN lower(is_symbolic_link) = 'true' THEN 1
        ELSE 0
    END,
    main_python_file_path, installation_directory,
    variant_type, functional_name, name, author, version, blender_version, location,
    description, warning, documentation_url, tracker_url, support, category,
    parent_blender_version_id, created, modified
FROM addon;

DROP TABLE addon;
ALTER TABLE addon_rebuilt RENAME TO addon;

CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_addon_path_per_blender_version
    ON addon(main_python_file_path, parent_blender_version_id);
