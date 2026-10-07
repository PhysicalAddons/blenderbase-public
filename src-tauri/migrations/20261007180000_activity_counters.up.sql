-- Counters Blenderbase itself keeps for the achievements: addons installed through the app,
-- setups shared and applied, Blender versions installed. One row per kind.
CREATE TABLE activity_counter (
    kind TEXT PRIMARY KEY NOT NULL,
    value INTEGER NOT NULL DEFAULT 0,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
