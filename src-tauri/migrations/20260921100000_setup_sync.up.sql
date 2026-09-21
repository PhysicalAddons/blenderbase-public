-- The folder a setup is kept in sync through (typically inside a cloud drive such as
-- Dropbox or OneDrive), and the last setup this computer saved there or applied from
-- there, so a newer file in the folder can be noticed. A single row.
CREATE TABLE setup_sync (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    folder_path TEXT NOT NULL DEFAULT '',
    last_synced_hash TEXT NOT NULL DEFAULT '',
    last_synced_at TEXT NOT NULL DEFAULT '',
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO setup_sync (id) VALUES (1);
