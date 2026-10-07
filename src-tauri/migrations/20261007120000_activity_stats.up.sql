-- Stats and achievements, phase 1 (counting). One row per Blender session, imported from the
-- activity logs the startup script writes; one counter per session and event kind; unlocked
-- achievements (filled from phase 3 on). A session keeps its version text so hours survive an
-- uninstall: the foreign key only clears the link.
CREATE TABLE blender_session (
    id TEXT PRIMARY KEY NOT NULL,                  -- the session id from the log's start line
    blender_version_id TEXT NULL,                  -- matched by installation folder, then by build hash
    version TEXT NOT NULL,                         -- bpy.app.version_string, kept if the version is deleted
    series TEXT NOT NULL,                          -- "5.2", from version_tuple
    build_hash TEXT NOT NULL DEFAULT '',
    binary_path TEXT NOT NULL,
    pid INTEGER NOT NULL DEFAULT 0,                -- Blender's process id, for the unclean-exit check
    started_at TEXT NOT NULL,                      -- UTC ISO 8601, the start line
    ended_at TEXT NOT NULL DEFAULT '',              -- the end line, else the last heartbeat
    open_seconds INTEGER NOT NULL DEFAULT 0,       -- "up" of the last line read
    active_seconds INTEGER NOT NULL DEFAULT 0,     -- 60 per heartbeat with active = true
    is_finished INTEGER NOT NULL DEFAULT 0,        -- an end line, or three missed heartbeats
    is_clean_exit INTEGER NOT NULL DEFAULT 0,      -- an end line was seen
    log_file_name TEXT NOT NULL,
    import_offset INTEGER NOT NULL DEFAULT 0,      -- bytes already read, for sessions still running
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (blender_version_id) REFERENCES blender_version(id) ON DELETE SET NULL
);
CREATE INDEX blender_session_started_at ON blender_session(started_at);
CREATE INDEX blender_session_blender_version_id ON blender_session(blender_version_id);

-- kind: file_opened, new_file, file_saved, render_started, render_finished,
--       render_cancelled, render_seconds, undo, redo, cube_deleted, suzanne_added
CREATE TABLE blender_session_counter (
    blender_session_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    value INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (blender_session_id, kind),
    FOREIGN KEY (blender_session_id) REFERENCES blender_session(id) ON DELETE CASCADE
);

CREATE TABLE achievement_unlock (
    achievement_id TEXT PRIMARY KEY NOT NULL,      -- the id in the catalogue, e.g. cube_slayer
    unlocked_at TEXT NOT NULL,                     -- ended_at of the session that crossed the line
    blender_session_id TEXT NULL,
    is_seen INTEGER NOT NULL DEFAULT 0,            -- set when the Stats view opens; clears the badge
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (blender_session_id) REFERENCES blender_session(id) ON DELETE SET NULL
);

-- The switch, off until the user turns it on under Settings > Launch. Turning it on puts the
-- script into every Blender series' startup folder; turning it off removes it and keeps the rows.
INSERT INTO app_setting
(code, name, description, disclaimer, is_enabled, is_read_on_app_launch, is_read_on_app_exit, default_int_value, int_value, control_title, parent_app_setting_id, app_setting_type_id, app_setting_action_type_id, measurement_unit_type_id, input_value_type_id, created, modified)
VALUES
('COUNT_BLENDER_ACTIVITY', 'Count time and events in Blender', 'A small script in each Blender series'' startup folder counts hours and a few events. The numbers stay on this computer.', NULL, 0, 0, 0, 0, 0, 'Count', NULL, 6, 2, NULL, 10, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
