//! The one file Blenderbase places in a Blender series' user startup folder
//! (`<series>/scripts/startup/blenderbase_activity.py`), and its install and removal. Blender
//! imports every module in that folder at launch and calls its `register()`, so the script
//! needs no enabling and does not appear in the addon list. The script's own text is the single
//! source of truth for its version.

use std::path::{Path, PathBuf};

use crate::core::py_string_literal;

pub const ACTIVITY_SCRIPT_FILE_NAME: &str = "blenderbase_activity.py";
const SCRIPT_TEMPLATE: &str = include_str!("blenderbase_activity.py");
const LOG_DIR_PLACEHOLDER: &str = "{{LOG_DIR}}";
const VERSION_LINE_PREFIX: &str = "SCRIPT_VERSION = ";

/// The version of the script this build ships.
pub fn bundled_script_version() -> i64 {
    script_version_of(SCRIPT_TEMPLATE).unwrap_or(0)
}

/// The `SCRIPT_VERSION = n` line of a script's text, if it has one.
pub fn script_version_of(text: &str) -> Option<i64> {
    text.lines().find_map(|line| {
        line.strip_prefix(VERSION_LINE_PREFIX)
            .and_then(|rest| rest.split('#').next())
            .and_then(|value| value.trim().parse::<i64>().ok())
    })
}

/// The script with the log folder written in as a Python string literal. On Windows the path
/// is spelled with backslashes throughout, whatever mix the environment handed over.
pub fn render_script(log_dir: &Path) -> String {
    let mut path = log_dir.to_string_lossy().to_string();
    if cfg!(windows) {
        path = path.replace('/', "\\");
    }
    SCRIPT_TEMPLATE.replace(LOG_DIR_PLACEHOLDER, &py_string_literal(&path))
}

pub fn script_path(series_dir: &Path) -> PathBuf {
    series_dir
        .join("scripts")
        .join("startup")
        .join(ACTIVITY_SCRIPT_FILE_NAME)
}

/// Writes the script for one series unless an identical copy is there already. An older copy,
/// or one pointing at another log folder, is replaced; a newer one (from a newer Blenderbase
/// running beside this build) is left alone. Returns whether the file was written.
pub fn install_script(series_dir: &Path, log_dir: &Path) -> Result<bool, String> {
    let path = script_path(series_dir);
    let wanted = render_script(log_dir);
    if let Ok(existing) = std::fs::read_to_string(&path) {
        if existing == wanted {
            return Ok(false);
        }
        if script_version_of(&existing).unwrap_or(0) > bundled_script_version() {
            return Ok(false);
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create {}: {}", parent.display(), e))?;
    }
    std::fs::write(&path, wanted).map_err(|e| format!("Could not write {}: {}", path.display(), e))?;
    Ok(true)
}

/// Removes the script from one series. Returns whether there was one.
pub fn remove_script(series_dir: &Path) -> Result<bool, String> {
    let path = script_path(series_dir);
    if !path.exists() {
        return Ok(false);
    }
    std::fs::remove_file(&path).map_err(|e| format!("Could not remove {}: {}", path.display(), e))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("blenderbase-activity-{}-{}", tag, uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_bundled_script_carries_a_version_and_the_placeholder() {
        assert!(bundled_script_version() >= 1);
        assert!(SCRIPT_TEMPLATE.contains(LOG_DIR_PLACEHOLDER));
        assert_eq!(script_version_of("x = 1\nSCRIPT_VERSION = 7  # note\n"), Some(7));
        assert_eq!(script_version_of("nothing here"), None);
    }

    #[test]
    fn rendering_writes_the_log_folder_as_a_python_literal() {
        let rendered = render_script(Path::new("C:\\Users\\me\\AppData\\Roaming\\com.physicaladdons.blenderbase\\activity"));
        assert!(!rendered.contains(LOG_DIR_PLACEHOLDER));
        assert!(rendered.contains("LOG_DIR = \"C:\\\\Users\\\\me\\\\AppData\\\\Roaming\\\\com.physicaladdons.blenderbase\\\\activity\""));
        assert!(rendered.contains("if bpy.app.background:"), "headless runs must not count");
    }

    #[test]
    fn install_writes_once_replaces_older_and_remove_deletes() {
        let series = temp_dir("series");
        let logs = temp_dir("logs");
        let path = script_path(&series);
        assert!(install_script(&series, &logs).unwrap(), "first install writes");
        assert!(path.is_file());
        assert!(!install_script(&series, &logs).unwrap(), "an identical copy is left alone");

        std::fs::write(&path, "SCRIPT_VERSION = 0\n").unwrap();
        assert!(install_script(&series, &logs).unwrap(), "an older copy is replaced");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), render_script(&logs));

        let other_logs = temp_dir("logs2");
        assert!(install_script(&series, &other_logs).unwrap(), "another log folder means a rewrite");

        std::fs::write(&path, "SCRIPT_VERSION = 999\n").unwrap();
        assert!(!install_script(&series, &logs).unwrap(), "a newer copy is kept");

        assert!(remove_script(&series).unwrap());
        assert!(!path.exists());
        assert!(!remove_script(&series).unwrap(), "nothing left to remove");
        let _ = std::fs::remove_dir_all(&series);
        let _ = std::fs::remove_dir_all(&logs);
        let _ = std::fs::remove_dir_all(&other_logs);
    }
}
