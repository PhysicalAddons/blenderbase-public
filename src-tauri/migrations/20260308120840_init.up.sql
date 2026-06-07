CREATE TABLE blender_installation_location (
    id TEXT PRIMARY KEY NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    full_control INTEGER NOT NULL DEFAULT 0,
    modify INTEGER NOT NULL DEFAULT 0,
    read_and_execute INTEGER NOT NULL DEFAULT 0,
    list_folder_contents INTEGER NOT NULL DEFAULT 0,
    read INTEGER NOT NULL DEFAULT 0,
    write INTEGER NOT NULL DEFAULT 0,
    special_permissions INTEGER NOT NULL DEFAULT 0,
    directory_path TEXT NOT NULL,
    created_by TEXT NULL,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE blender_version_build_type (
    id INTEGER PRIMARY KEY NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE blend_file (
    id TEXT PRIMARY KEY NOT NULL,
    file_path TEXT NOT NULL,
    file_name TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    created_datetime TEXT NOT NULL,
    modified_datetime TEXT NOT NULL, 
    accessed_datetime TEXT NOT NULL,
    last_used_blender_version_id TEXT NULL,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (last_used_blender_version_id) REFERENCES blender_version(id) ON DELETE SET NULL
);

CREATE TABLE blender_series (
    id TEXT PRIMARY KEY NOT NULL,
    is_collapsed INTEGER NOT NULL DEFAULT 0,
    series TEXT NULL,
    config_directory_path TEXT NOT NULL,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

create TABLE blend_file_blender_series (
    blend_file_id TEXT NOT NULL,
    blender_series_id TEXT NOT NULL,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (blend_file_id) REFERENCES blend_file(id),
    FOREIGN KEY (blender_series_id) REFERENCES blender_series(id)
);

CREATE TABLE addon (
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

CREATE TABLE app_setting_type (
    id INTEGER PRIMARY KEY NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    parent_app_setting_type_id TEXT NULL,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_app_setting_type_id) REFERENCES app_setting_type(id) ON DELETE SET NULL
);

CREATE TABLE app_setting_action_type (
    id INTEGER PRIMARY KEY NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE input_value_type (
    id INTEGER PRIMARY KEY NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE app_setting (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    disclaimer TEXT NULL,
    is_enabled INTEGER NOT NULL DEFAULT 0,
    is_read_on_app_launch INTEGER NOT NULL DEFAULT 0,
    is_read_on_app_exit INTEGER NOT NULL DEFAULT 0,
    default_int_value INTEGER NULL DEFAULT 0,
    int_value INTEGER NULL DEFAULT 0, -- true, false, integer values. Depends on the setting.
    min_int_value INTEGER NULL,
    max_int_value INTEGER NULL,
    is_signed_int_value INTEGER NULL,
    default_text_value TEXT NULL,
    text_value TEXT NULL, -- text values, file path, possibly object.
    min_length_text_value INTEGER NULL,
    max_length_text_value INTEGER NULL,
    regex_format_text_value TEXT NULL,
    
    default_range_int_value_from INTEGER NULL,
    default_range_int_value_to INTEGER NULL,
    range_int_value_from INTEGER NULL,
    range_int_value_to INTEGER NULL,

    min_range_int_value_from INTEGER NULL,
    min_range_int_value_to INTEGER NULL,
    max_range_int_value_from INTEGER NULL,
    max_range_int_value_to INTEGER NULL,
    
    is_signed_range_int_value_from INTEGER NULL,
    is_signed_range_int_value_to INTEGER NULL,
    
    default_range_text_value_from TEXT NULL,
    default_range_text_value_to TEXT NULL,
    range_text_value_from TEXT NULL,
    range_text_value_to TEXT NULL,
    min_length_range_text_value_from INTEGER NULL,
    max_length_range_text_value_from INTEGER NULL,
    min_length_range_text_value_to INTEGER NULL,
    max_length_range_text_value_to INTEGER NULL,
    regex_format_range_text_value_from TEXT NULL,
    regex_format_range_text_value_to TEXT NULL,
    control_title TEXT NULL, -- Used as a tooltip or title for the UI control (name on a button or a tooltip for a toggle).
    parent_app_setting_id TEXT NULL,
    app_setting_type_id INTEGER NOT NULL,
    app_setting_action_type_id INTEGER NOT NULL,
    measurement_unit_type_id INTEGER NULL,
    input_value_type_id INTEGER NOT NULL,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_app_setting_id) REFERENCES app_setting(id) ON DELETE SET NULL,
    FOREIGN KEY (app_setting_type_id) REFERENCES app_setting_type(id) ON DELETE SET NULL,
    FOREIGN KEY (app_setting_action_type_id) REFERENCES app_setting_action_type(id) ON DELETE SET NULL,
    FOREIGN KEY (measurement_unit_type_id) REFERENCES measurement_unit_type(id) ON DELETE SET NULL,
    FOREIGN KEY (input_value_type_id) REFERENCES input_value_type(id) ON DELETE SET NULL
);

CREATE TABLE download_status_type (
    id INTEGER PRIMARY KEY NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    parent_download_status_type_id TEXT NULL,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_download_status_type_id) REFERENCES download_status_type(id) ON DELETE SET NULL
);

CREATE TABLE blender_version (
    id TEXT PRIMARY KEY NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    custom_name TEXT NULL,
    url TEXT NULL, -- download_url 
    app TEXT NULL,
    version TEXT NULL,
    series TEXT NULL, -- ?????
    risk_id TEXT NULL, -- variant_type
    branch TEXT NULL,
    patch_url TEXT NULL, -- pull_request_link_url
    patch TEXT NULL, -- pull_request_number
    hash_url TEXT NULL, -- commit_link_url
    hash TEXT NULL, -- commit_hash
    platform TEXT NULL,
    architecture TEXT NULL,
    bitness INTEGER NULL,
    file_mtime INTEGER NULL, -- release_date 
    file_name TEXT NULL,
    file_size INTEGER NOT NULL,--app_size INTEGER NOT NULL,
    file_extension TEXT NULL, -- file_extension 
    release_cycle TEXT NULL,
    checksum TEXT NULL,
    installation_directory_path TEXT NOT NULL,
    executable_file_path TEXT NULL,
    blender_installation_location_id TEXT NOT NULL,
    download_status_type_id INTEGER NOT NULL,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (blender_installation_location_id) REFERENCES blender_installation_location(id) ON DELETE CASCADE,
    FOREIGN KEY (download_status_type_id) REFERENCES download_status_type(id) ON DELETE SET NULL
);

CREATE TABLE measurement_unit_type (
    id INTEGER PRIMARY KEY NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_unique_blend_file_path ON blend_file(file_path);
CREATE UNIQUE INDEX idx_unique_addon_main_python_file_path ON addon(main_python_file_path);
CREATE UNIQUE INDEX idx_unique_blender_installation_location_directory_path ON blender_installation_location(directory_path);
CREATE UNIQUE INDEX idx_unique_blender_version_version_download_status_type_id ON blender_version(version, download_status_type_id) WHERE executable_file_path IS NULL;
CREATE UNIQUE INDEX idx_unique_blender_version_executable_file_path ON blender_version(executable_file_path) WHERE executable_file_path IS NOT NULL;
-- CREATE UNIQUE INDEX idx_unique_blender_version_executable_file_path ON blender_version(executable_file_path);
-- CREATE UNIQUE INDEX idx_unique_blender_version_executable_file_path ON blender_version(version, executable_file_path, download_status_type_id); -- WHERE executable_file_path IS NOT NULL;
CREATE UNIQUE INDEX idx_unique_blender_series_config_directory_path ON blender_series(config_directory_path);
CREATE UNIQUE INDEX idx_unique_blend_file_blender_series ON blend_file_blender_series(blend_file_id, blender_series_id);
-- Create a trigger for blend_file deletion
CREATE TRIGGER trigger_delete_blend_file_blender_series_on_blend_file_delete
AFTER DELETE ON blend_file
FOR EACH ROW
BEGIN
    DELETE FROM blend_file_blender_series
    WHERE blend_file_id = OLD.id;
END;
-- Create a trigger for blender_series deletion
CREATE TRIGGER trigger_delete_blend_file_blender_series_on_blender_series_delete
AFTER DELETE ON blender_series
FOR EACH ROW
BEGIN
    DELETE FROM blend_file_blender_series
    WHERE blender_series_id = OLD.id;
END;

INSERT INTO measurement_unit_type
(id, code,      name,                   description,                                created,           modified)
VALUES
-- Data Storage Units (Decimal, base-10)
(1,  'BIT',     'Bit',                  'Binary digit, the smallest unit of data',  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(2,  'BYTE',    'Byte',                 '8 bits',                                   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(3,  'KB',      'Kilobyte',             '1000 bytes (decimal)',                     CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(4,  'MB',      'Megabyte',             '1000 kilobytes (decimal)',                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(5,  'GB',      'Gigabyte',             '1000 megabytes (decimal)',                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(6,  'TB',      'Terabyte',             '1000 gigabytes (decimal)',                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(7,  'PB',      'Petabyte',             '1000 terabytes (decimal)',                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(8,  'EB',      'Exabyte',              '1000 petabytes (decimal)',                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(9,  'ZB',      'Zettabyte',            '1000 exabytes (decimal)',                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(10, 'YB',      'Yottabyte',            '1000 zettabytes (decimal)',                CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
-- Data Storage Units (Binary, base-2)
(11, 'KIB',     'Kibibyte',             '1024 bytes (binary)',                      CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(12, 'MIB',     'Mebibyte',             '1024 kibibytes (binary)',                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(13, 'GIB',     'Gibibyte',             '1024 mebibytes (binary)',                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(14, 'TIB',     'Tebibyte',             '1024 gibibytes (binary)',                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(15, 'PIB',     'Pebibyte',             '1024 tebibytes (binary)',                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(16, 'EIB',     'Exbibyte',             '1024 pebibytes (binary)',                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
-- Data Transfer Rates (Decimal)
(17, 'BPS',     'Bits per second',      'Data transfer rate in bits per second',    CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(18, 'B/S',     'Bytes per second',     'Data transfer rate in bytes per second',   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(19, 'KBS',     'Kilobits per second',  '1000 bits per second',                     CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(20, 'KB/S',    'Kilobytes per second', '1000 bytes per second',                    CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(21, 'MBS',     'Megabits per second',  '1000 kilobits per second',                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(22, 'MB/S',    'Megabytes per second', '1000 kilobytes per second',                CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(23, 'GBS',     'Gigabits per second',  '1000 megabits per second',                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(24, 'GB/S',    'Gigabytes per second', '1000 megabytes per second',                CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(25, 'TBS',     'Terabits per second',  '1000 gigabits per second',                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(26, 'TB/S',    'Terabytes per second', '1000 gigabytes per second',                CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
-- Data Transfer Rates (Binary)
(27, 'KIBS',    'Kibibits per second',  '1024 bits per second',                     CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(28, 'KIB/S',   'Kibibytes per second', '1024 bytes per second',                    CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(29, 'MIBS',    'Mebibits per second',  '1024 kibibits per second',                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(30, 'MIB/S',   'Mebibytes per second', '1024 kibibytes per second',                CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(31, 'GIBS',    'Gibibits per second',  '1024 mebibits per second',                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(32, 'GIB/S',   'Gibibytes per second', '1024 mebibytes per second',                CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
-- Time Units
(33, 'NS',      'Nanosecond',           '1e-9 seconds',                             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(34, 'US',      'Microsecond',          '1e-6 seconds',                             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(35, 'MS',      'Millisecond',          '1e-3 seconds',                             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(36, 'S',       'Second',               'Base unit of time',                        CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(37, 'MIN',     'Minute',               '60 seconds',                               CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(38, 'H',       'Hour',                 '60 minutes',                               CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(39, 'D',       'Day',                  '24 hours',                                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(40, 'W',       'Week',                 '7 days',                                   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(41, 'M',       'Month',                'Approximately 30 days',                    CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(42, 'Y',       'Year',                 '365 days',                                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(43, 'VER',     'Version',              'Software application version',             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(44, 'SER',     'Series',               'Software application version series',      CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);


INSERT INTO input_value_type
(id,    code,           name,           description,                                        created,           modified)
VALUES
(1,     'NULL',         'Null',         'Absence of a value',                               CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(2,     'INTEGER',      'Integer',      'Whole number (positive, negative, or zero)',       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(3,     'STRING',       'String',       'Sequence of characters (text)',                    CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(4,     'FLOAT',        'Float',        'Floating-point number (single-precision)',         CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(5,     'BYTE',         'Byte',         'Single byte (8-bit integer, 0-255)',               CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(6,     'CHAR',         'Character',    'Single character',                                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(7,     'JSON',         'JSON',         'JavaScript Object Notation (structured text)',     CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(8,     'XML',          'XML',          'eXtensible Markup Language (structured text)',     CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(9,     'BOOLEAN',      'Boolean',      'True or false value',                              CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(10,    'BIT',          'Bit',          'Binary digit (0 or 1)',                            CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(11,    'DATE',         'Date',         'Calendar date (year, month, day)',                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(12,    'TIME',         'Time',         'Time of day (hour, minute, second)',               CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(13,    'DATETIME',     'DateTime',     'Combined date and time',                           CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(14,    'TIMESTAMP',    'Timestamp',    'Date and time with timezone or precision',         CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(15,    'BINARY',       'Binary',       'Raw binary data (fixed-length)',                   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(16,    'UUID',         'UUID',         'Universally Unique Identifier',                    CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(17,    'ARRAY',        'Array',        'Ordered list of values',                           CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(18,    'OBJECT',       'Object',       'Key-value pairs (structured data)',                CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

INSERT INTO blender_version_build_type
(id, is_default, code,       name,                   description,                                created,           modified) 
VALUES
(1,  1,          'RELEASE',  'Release build type',   'Release build type for Blender versions',  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(2,  0,          'DAILY',    'Daily build type',     'Daily build type for Blender versions',    CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(3,  0,          'PATCH',    'Patch build type',     'Patch build type for Blender versions',    CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

INSERT INTO app_setting_type
(id, code,                          name,                                       description,                                            parent_app_setting_type_id, created,           modified) 
VALUES 
(1, 'APP_GENERAL',                  'General App Settings',                     'General settings for the app.',                        NULL,                       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(2, 'APP_UI',                       'UI Settings',                              'Settings for the app UI.',                             NULL,                       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(3, 'DATABASE',                     'Database Settings',                        'Settings for the app''s local database.',              NULL,                       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(4, 'UPDATER',                      'Updater Settings',                         'Settings for the app''s updater.',                     NULL,                       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(5, 'NETWORK',                      'Network Settings',                         'Settings for the app''s network interactions.',        NULL,                       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(6, 'BLENDER_VERSION_INSTALLED',    'Installed Blender Version Settings',       'Settings for the installed Blender versions.',         NULL,                       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(7, 'BLENDER_VERSION_DOWNLOAD',     'Blender Version Download Settings',        'Settings for the Blender versions download.',          NULL,                       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(8, 'BLENDER_ADDON',                'Installed Blender Version Addon Settings', 'Settings for the installed Blender version addons.',   NULL,                       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(9, 'BLEND_FILE',                   'Blend File Settings',                      'Settings for the .blend files.',                       NULL,                       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

INSERT INTO app_setting_action_type
(id, code,               name,                   description,                                    created,           modified) 
VALUES 
(1,  'INPUT_BUTTON',     'Button action',        'Performs an action',                           CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(2,  'INPUT_TOGGLE',           'Toggle action',        'True/False settings',                          CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(3,  'INPUT_SELECTION',        'Selection action',     'Dropdown selection settings',                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(4,  'INPUT_TEXT',       'Text input action',    'Text input field',                             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(5,  'INPUT_DECIMAL',    'Decimal input action', 'Decimal number (int, float) input field',      CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(6,  'INPUT_FILE',       'File input action',    'File input field',                             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(7,  'INPUT_RANGE',      'Range input action',   'Range input field',                            CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

INSERT INTO app_setting
(code,                                                   name,                                                                  description,                                                                                                                                                disclaimer,                                                         is_enabled, is_read_on_app_launch,  is_read_on_app_exit, default_int_value, int_value, min_int_value, max_int_value,  is_signed_int_value, default_text_value, text_value, min_length_text_value, max_length_text_value, regex_format_text_value,       default_range_int_value_from, default_range_int_value_to, range_int_value_from, range_int_value_to, min_range_int_value_from, min_range_int_value_to, max_range_int_value_from, max_range_int_value_to, is_signed_range_int_value_from, is_signed_range_int_value_to, default_range_text_value_from, default_range_text_value_to, range_text_value_from, range_text_value_to, min_length_range_text_value_from, max_length_range_text_value_from, min_length_range_text_value_to, max_length_range_text_value_to, regex_format_range_text_value_from,            regex_format_range_text_value_to,             control_title,      parent_app_setting_id,  app_setting_type_id, app_setting_action_type_id, measurement_unit_type_id, input_value_type_id, created,           modified)
VALUES                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               
('RESET_DATABASE',                                      'Reset the entire local database or parts of it',                       'Removes Blenderbase database entries (excluding the actual Blender versions and their add-on files) and restarts the app.',                                NULL,                                                               0,          0,                      0,                   NULL,              NULL,      NULL,          NULL,           NULL,                NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                             NULL,                             NULL,                           NULL,                           NULL,                                          NULL,                                         'Reset',            NULL,                   1,                   1,                          NULL,                     1,                   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('SET_CHECK_INTERNET_CONNECTION_TIMEOUT',               'Set the cooldown for checking internet connection',                    'Set a cooldown for checking an existing internet connection. Will check internet connection after the amount of time has passed since last check.',        NULL,                                                               1,          0,                      0,                   5,                 5,         3,             3600,           0,                   NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                             NULL,                             NULL,                           NULL,                           NULL,                                          NULL,                                         'Set timeout',      NULL,                   5,                   5,                          36,                       2,                   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('CHECK_FOR_UPDATE_ON_LAUNCH',                          'Check for updates when app is launched',                               'Automatically checks for new Blenderbase versions in the public release page when launching app.',                                                         NULL,                                                               0,          1,                      0,                   1,                 1,         NULL,          NULL,           NULL,                NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                             NULL,                             NULL,                           NULL,                           NULL,                                          NULL,                                         'Check',            NULL,                   4,                   2,                          NULL,                     10,                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('CHECK_FOR_UPDATE',                                    'Check for update',                                                     'Manually checks for a new Blenderbase version in the public release page.',                                                                                NULL,                                                               0,          0,                      0,                   NULL,              NULL,      NULL,          NULL,           NULL,                NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                             NULL,                             NULL,                           NULL,                           NULL,                                          NULL,                                         'Check',            NULL,                   4,                   1,                          NULL,                     10,                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('OPEN_APP_VERSION_ONLINE_REPOSITORY',                  'Open Blenderbase version online respository',                          'Opens the Blenderbase version online repository, to view the version selection and release details',                                                       NULL,                                                               1,          0,                      0,                   NULL,              NULL,      NULL,          NULL,           NULL,                NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                             NULL,                             NULL,                           NULL,                           NULL,                                          NULL,                                         'Open respository', NULL,                   4,                   1,                          NULL,                     1,                   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('OPEN_SYSTEM_CONSOLE_ON_LAUNCH',                       'Open system console when launching Blender',                           'Ensures system console opens when launching Blender using the ''-con'' launch argument.',                                                                  NULL,                                                               0,          0,                      0,                   1,                 1,         NULL,          NULL,           NULL,                NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                             NULL,                             NULL,                           NULL,                           NULL,                                          NULL,                                         'Open',             NULL,                   6,                   2,                          NULL,                     10,                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('CHANGE_ACTIVE_BLENDER_VERSION_THROUGH_TABLE',         'Change active Blender version through the Installed Blender Table',    'When clicking on a version in the installed Blender version table, that Blender version will be set as the active version.',                               NULL,                                                               0,          0,                      0,                   0,                 0,         NULL,          NULL,           NULL,                NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                             NULL,                             NULL,                           NULL,                           NULL,                                          NULL,                                         'Change',           NULL,                   6,                   2,                          NULL,                     10,                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('MINIMIZE_BLENDERBASE_ON_LAUNCH',                      'Minimize Blenderbase when launching Blender',                          'Minimizes the Blenderbase window when a new Blender instance is launched.',                                                                                NULL,                                                               0,          0,                      0,                   1,                 1,         NULL,          NULL,           NULL,                NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                             NULL,                             NULL,                           NULL,                           NULL,                                          NULL,                                         'Minimize',         NULL,                   6,                   2,                          NULL,                     10,                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('PROCESS_ADDON_ON_LAUNCH',                             'Process add-ons when launching Blender',                               'Minimizes the Blenderbase window when a new Blender instance is launched.',                                                                                NULL,                                                               0,          0,                      0,                   0,                 0,         NULL,          NULL,           NULL,                NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                             NULL,                             NULL,                           NULL,                           NULL,                                          NULL,                                         'Process',          NULL,                   6,                   2,                          NULL,                     10,                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('SET_INSTALLED_BLENDER_VERSION_LIMIT_FOR_SCRAPING',    'Set the installed Blender version range that should be scraped',       'Set the Blender version range for installed Blender versions, that should be scraped (including).',                                                        NULL,                                                               0,          0,                      0,                   NULL,              NULL,      NULL,          NULL,           NULL,                NULL,               NULL,        NULL,                 NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                              100,                             NULL,                            100,                           '^[A-Za-z0-9]+\.[A-Za-z0-9]+\.[A-Za-z0-9]+$',  '^[A-Za-z0-9]+\.[A-Za-z0-9]+\.[A-Za-z0-9]+$', 'Set range',        NULL,                   6,                   7,                          43,                       3,                   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),                                                                                                                                                                                                                           
('INIT_BLENDER_DATA_ON_FIRST_INSTANCE_LAUNCH',          'Initialize Blender version data after first instance launch',          'Initialize Blender version data after first instance launch to fill in missing data for Blender version entry.',                                           'Skipping this will result in missing informational data.',         0,          0,                      0,                   1,                 1,         NULL,          NULL,           NULL,                NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                             NULL,                             NULL,                           NULL,                           NULL,                                          NULL,                                         'Get data',         NULL,                   6,                   2,                          NULL,                     10,                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('REFRESH_BLENDER_DOWNLOAD_ON_LAUNCH',                  'Refresh downloadable Blender database when launching Blenderbase',     'Automatically updates the downloadable Blender version database when launching Blenderbase.',                                                              NULL,                                                               0,          1,                      0,                   1,                 1,         NULL,          NULL,           NULL,                NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                             NULL,                             NULL,                           NULL,                           NULL,                                          NULL,                                         'Refresh',          NULL,                   7,                   2,                          NULL,                     10,                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('DOWNLOAD_BLENDER_IN_DEFAULT_INSTALLATION_LOCATION',   'Download Blender in default installation location',                    'Automatically downloads Blender version in a set default installation location, skipping download location pop-up.',                                       NULL,                                                               0,          0,                      0,                   1,                 1,         NULL,          NULL,           NULL,                NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                             NULL,                             NULL,                           NULL,                           NULL,                                          NULL,                                         'Check',            NULL,                   7,                   2,                          NULL,                     10,                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('SET_DOWNLOADABLE_BLENDER_VERSION_LIMIT_FOR_SCRAPING', 'Set the downloadable Blender version range that should be scraped',    'Set the Blender version range for downloadable Blender versions, that should be scraped (including).',                                                     NULL,                                                               0,          0,                      0,                   NULL,              NULL,      NULL,          NULL,           NULL,                NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         '4.0.0',                       NULL,                       '4.0.0',                NULL,                NULL,                              100,                             NULL,                            100,                           '^[A-Za-z0-9]+\.[A-Za-z0-9]+\.[A-Za-z0-9]+$',  '^[A-Za-z0-9]+\.[A-Za-z0-9]+\.[A-Za-z0-9]+$', 'Set range',        NULL,                   7,                   7,                          43,                       3,                   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('SET_RECENT_FILES_BLENDER_SERIES_LIMIT_FOR_SCRAPING',  'Set the recent files Blender series range that should be scraped',     'Set the Blender series range for recent files, that should be scraped (including).',                                                                       NULL,                                                               0,          0,                      0,                   NULL,              NULL,      NULL,          NULL,           NULL,                NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                              100,                             NULL,                            100,                           '^[A-Za-z0-9]+\.[A-Za-z0-9]+$',                 '^[A-Za-z0-9]+\.[A-Za-z0-9]+$',              'Set range',        NULL,                   9,                   7,                          44,                       3,                   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('CREATE_APP_LIBRARY_DIRECTORY_AUTOMATICALLY',          'Create Blenderbase library directory if it doesn''t already exist',    'Create Blenderbase library directory that would hold the default Blender installation location for Blender versions downloaded through Blenderbase.',      NULL,                                                               1,          1,                      0,                   1,                 1,         NULL,          NULL,           NULL,                NULL,               NULL,       NULL,                  NULL,                  NULL,                          NULL,                         NULL,                       NULL,                 NULL,               NULL,                     NULL,                   NULL,                     NULL,                   NULL,                           NULL,                         NULL,                          NULL,                        NULL,                  NULL,                NULL,                             Null,                             NULL,                           NULL,                           NULL,                                          NULL,                                         'Check',            NULL,                   1,                   2,                          NULL,                     10,                  CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

INSERT INTO download_status_type
(id,    code,           name,                           description,                                                            parent_download_status_type_id,         created,           modified) 
VALUES 
(1,     'PENDING',      'Download is queued',           'The download is queued and waiting to start',                          NULL,                                   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(2,     'DOWNLOADING',  'Download in progress',         'The file is actively being downloaded',                                NULL,                                   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(3,     'PAUSED',       'The download is paused',       'The download has been temporarily stopped and can be resumed later',   NULL,                                   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(4,     'COMPLETED',    'The download is finished',     'The download is finished and the file is ready to use',                NULL,                                   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(5,     'FAILED',       'The download failed',          'The download was unsuccessful',                                        NULL,                                   CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
