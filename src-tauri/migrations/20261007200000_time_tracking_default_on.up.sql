-- The stats switch becomes "Time tracking and achievements" and is on by default: everything it
-- counts stays on this computer, and the Stats view, the hours on the Blender rows and the
-- badges all hang off it. Turning it off under Settings > Launch removes the script from the
-- Blender series folders and hides the view; the numbers already counted are kept.
UPDATE app_setting
SET name = 'Time tracking and achievements',
    description = 'Counts the hours in each Blender version and unlocks achievements. Everything stays on this computer.',
    control_title = 'Track',
    default_int_value = 1,
    int_value = 1,
    modified = CURRENT_TIMESTAMP
WHERE code = 'COUNT_BLENDER_ACTIVITY';
