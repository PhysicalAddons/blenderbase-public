-- Whether the user has confirmed this location as the place Blender versions go.
-- Existing rows start unconfirmed, so the one-time prompt shows on the next download.
ALTER TABLE blender_installation_location ADD COLUMN is_confirmed INTEGER NOT NULL DEFAULT 0;
