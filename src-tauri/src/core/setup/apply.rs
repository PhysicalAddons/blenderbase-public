use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    backup::{backup_series, list_backups, restore_backup},
    bundle::extract_bundle_blob,
    impls::{SetupProgress, PORTABLE_PREFERENCE_GROUPS},
    manifest::{SetupBlob, SetupSeries},
    scripts::APPLY_SETUP_PY,
};
use crate::core::{
    blender_config_root, extract_json_payload, py_string_literal, run_blender_python_with_env,
    BLENDERBASE_JSON_MARKER, BLENDER_USER_RESOURCES, COM_PHYSICALADDONS_BLENDERBASE,
};

/// Seconds Blender may take to write one series' preferences, theme and keymap.
const APPLY_TIMEOUT_SECS: u64 = 120;
const SETUP_BACKUPS: &str = "setup-backups";
/// Names Blender ships key configurations under; a restored one must not shadow them.
const BUNDLED_KEYCONFIGS: [&str; 3] = ["Blender", "Blender_27x", "Industry_Compatible"];
const RESTORED_KEYCONFIG: &str = "Blenderbase_setup";
const RESTORED_THEME_PREFIX: &str = "Blenderbase";

/// One series on this computer that a setup can be read from or written to.
#[derive(Debug, Clone)]
pub struct SetupTarget {
    pub series: String,
    /// The Blender version behind `executable`, for reports.
    pub version: String,
    pub executable: PathBuf,
    /// Set when Blender's user folder is somewhere else than the user's profile: an isolated
    /// test, and later a class setup that must not touch the personal configuration.
    pub user_resources: Option<PathBuf>,
}

impl SetupTarget {
    /// The folder that holds `config`, `scripts` and `extensions` for this target.
    pub fn series_directory(&self) -> Option<PathBuf> {
        match &self.user_resources {
            Some(v) => Some(v.clone()),
            None => blender_config_root().map(|root| root.join(&self.series)),
        }
    }

    pub fn envs(&self) -> Vec<(String, String)> {
        match &self.user_resources {
            Some(v) => vec![(String::from(BLENDER_USER_RESOURCES), v.to_string_lossy().to_string())],
            None => Vec::new(),
        }
    }
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetupApplyOptions {
    /// Series to apply; every series of the setup when absent.
    #[serde(default)]
    pub series: Option<Vec<String>>,
    #[serde(default = "yes")]
    pub preferences: bool,
    #[serde(default = "yes")]
    pub theme: bool,
    #[serde(default = "yes")]
    pub keymap: bool,
}

impl Default for SetupApplyOptions {
    fn default() -> Self {
        Self { series: None, preferences: true, theme: true, keymap: true }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SeriesApplyReport {
    pub series: String,
    /// Empty when the series was not applied; `skipped_reason` says why.
    pub applied_with: String,
    pub skipped_reason: Option<String>,
    pub backup_path: Option<String>,
    pub preferences_set: u32,
    pub preferences_skipped: Vec<String>,
    pub theme_applied: bool,
    pub keymaps_applied: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AppliedPreferences {
    #[serde(default)]
    set: u32,
    #[serde(default)]
    skipped: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AppliedSeries {
    preferences: AppliedPreferences,
    theme: Option<bool>,
    keymap: Option<Vec<String>>,
    #[serde(default)]
    warnings: Vec<String>,
}

/// Where the backups of one series are kept: next to the app database, one folder per series.
pub fn backups_directory(series: &str) -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join(COM_PHYSICALADDONS_BLENDERBASE).join(SETUP_BACKUPS).join(series))
}

/// Applies one series of a setup. The order is fixed: back up, extract and verify the files,
/// then let Blender write them. A failure before Blender runs leaves the series untouched.
pub async fn apply_series(
    bundle_path: &Path,
    section: &SetupSeries,
    target: &SetupTarget,
    backups_directory: &Path,
    work_directory: &Path,
    options: &SetupApplyOptions,
    progress: &SetupProgress,
) -> Result<SeriesApplyReport, String> {
    let series_directory = target
        .series_directory()
        .ok_or_else(|| String::from("Could not determine the Blender config directory"))?;
    let mut report = SeriesApplyReport {
        series: target.series.clone(),
        applied_with: target.version.clone(),
        ..SeriesApplyReport::default()
    };

    progress(format!("Blender {}: backing up the current configuration…", target.series));
    let backup = {
        let series_directory = series_directory.clone();
        let backups_directory = backups_directory.to_path_buf();
        tokio::task::spawn_blocking(move || backup_series(&series_directory, &backups_directory))
            .await
            .map_err(|e| format!("Failed to back up: {:?}", e))??
    };
    report.backup_path = Some(backup.to_string_lossy().to_string());

    let wanted = |enabled: bool, blob: &Option<SetupBlob>| if enabled { blob.clone() } else { None };
    let theme_name = section
        .theme
        .as_ref()
        .and_then(|t| t.name.clone())
        .unwrap_or_default();
    let files = [
        (wanted(options.preferences, &section.preferences), String::from("preferences.json")),
        (wanted(options.theme, &section.theme), format!("{}.xml", restored_theme_name(&theme_name))),
        (wanted(options.keymap, &section.keymap), String::from("keymap.py")),
    ];
    let mut extracted: Vec<String> = Vec::new();
    for (blob, file_name) in files {
        let Some(blob) = blob else {
            extracted.push(String::new());
            continue;
        };
        let destination = work_directory.join(&target.series).join(&file_name);
        let bundle_path = bundle_path.to_path_buf();
        let moved = destination.clone();
        tokio::task::spawn_blocking(move || extract_bundle_blob(&bundle_path, &blob.blob, &moved))
            .await
            .map_err(|e| format!("Failed to read the setup file: {:?}", e))??;
        extracted.push(destination.to_string_lossy().to_string());
    }

    let keymap_name = restored_keyconfig_name(
        section
            .keymap
            .as_ref()
            .and_then(|k| k.name.as_deref())
            .unwrap_or_default(),
    );
    let groups = format!(
        "({},)",
        PORTABLE_PREFERENCE_GROUPS
            .iter()
            .map(|g| py_string_literal(g))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let script = APPLY_SETUP_PY
        .replace("__MARKER__", BLENDERBASE_JSON_MARKER)
        .replace("__PREFERENCES_FILE__", &py_string_literal(&extracted[0]))
        .replace("__THEME_FILE__", &py_string_literal(&extracted[1]))
        .replace("__KEYMAP_FILE__", &py_string_literal(&extracted[2]))
        .replace("__KEYMAP_NAME__", &py_string_literal(&keymap_name))
        .replace("__PREFERENCE_GROUPS__", &groups);

    progress(format!("Blender {}: applying preferences, theme and keymap…", target.series));
    let stdout = run_blender_python_with_env(&target.executable, &script, APPLY_TIMEOUT_SECS, &target.envs()).await?;
    let payload = extract_json_payload(&stdout)?;
    let applied: AppliedSeries = match serde_json::Deserializer::from_str(payload.trim())
        .into_iter::<AppliedSeries>()
        .next()
    {
        Some(Ok(v)) => v,
        Some(Err(e)) => return Err(format!("Blender reported an unreadable result: {}", e)),
        None => return Err(String::from("Blender reported no result")),
    };
    report.preferences_set = applied.preferences.set;
    report.preferences_skipped = applied.preferences.skipped;
    report.theme_applied = applied.theme.unwrap_or(false);
    report.keymaps_applied = applied.keymap.unwrap_or_default();
    report.warnings = applied.warnings;
    Ok(report)
}

/// Puts the newest backup of a series back and returns the number of restored files.
pub async fn undo_last_apply(target: &SetupTarget, backups_directory: &Path) -> Result<usize, String> {
    let series_directory = target
        .series_directory()
        .ok_or_else(|| String::from("Could not determine the Blender config directory"))?;
    let newest = list_backups(backups_directory)
        .into_iter()
        .next()
        .ok_or_else(|| format!("There is no backup for Blender {}", target.series))?;
    tokio::task::spawn_blocking(move || restore_backup(&newest, &series_directory))
        .await
        .map_err(|e| format!("Failed to restore the backup: {:?}", e))?
}

/// Blender writes its preferences when it quits or saves them, which would undo an apply (or
/// be undone by it), so nothing is written into a profile while a Blender is open.
pub async fn is_blender_running() -> bool {
    #[cfg(target_os = "windows")]
    let probe = {
        let mut command = tokio::process::Command::new("tasklist");
        command.args(["/FI", "IMAGENAME eq blender.exe", "/FO", "CSV", "/NH"]);
        command.creation_flags(crate::core::CREATE_NO_WINDOW_FLAG);
        command
    };
    #[cfg(target_os = "macos")]
    let probe = {
        let mut command = tokio::process::Command::new("pgrep");
        command.args(["-x", "Blender"]);
        command
    };
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let probe = {
        let mut command = tokio::process::Command::new("pgrep");
        command.args(["-x", "blender"]);
        command
    };
    let mut probe = probe;
    match probe.output().await {
        Ok(output) => {
            let listed = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if cfg!(target_os = "windows") {
                listed.contains("blender.exe")
            } else {
                output.status.success() && !listed.trim().is_empty()
            }
        }
        // Without the probe there is no evidence either way; the apply itself stays safe to
        // repeat, so it is not blocked.
        Err(_) => false,
    }
}

/// File name (and so the preset name in Blender's menu) a restored theme is installed under.
fn restored_theme_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_'))
        .collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        String::from(RESTORED_THEME_PREFIX)
    } else if cleaned.starts_with(RESTORED_THEME_PREFIX) {
        cleaned.to_string()
    } else {
        format!("{}_{}", RESTORED_THEME_PREFIX, cleaned.replace(' ', "_"))
    }
}

/// A key configuration is a Python file named after it, so the name is reduced to an
/// identifier, and it never takes the name of one Blender ships.
fn restored_keyconfig_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    let cleaned = cleaned.trim_matches('_');
    if cleaned.is_empty() || BUNDLED_KEYCONFIGS.iter().any(|b| b.eq_ignore_ascii_case(cleaned)) {
        String::from(RESTORED_KEYCONFIG)
    } else {
        cleaned.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restored_names_are_safe_file_names() {
        assert_eq!(restored_theme_name("Default"), "Blenderbase_Default");
        assert_eq!(restored_theme_name("../../x"), "Blenderbase_x");
        assert_eq!(restored_theme_name(""), "Blenderbase");
        assert_eq!(restored_theme_name("Blenderbase_Default"), "Blenderbase_Default");
        assert_eq!(restored_keyconfig_name("Blender"), "Blenderbase_setup");
        assert_eq!(restored_keyconfig_name("industry_compatible"), "Blenderbase_setup");
        assert_eq!(restored_keyconfig_name("my keys"), "my_keys");
        assert_eq!(restored_keyconfig_name("..\\evil"), "evil");
        assert_eq!(restored_keyconfig_name("Blenderbase_setup"), "Blenderbase_setup");
    }

    #[test]
    fn an_isolated_target_stays_inside_its_folder() {
        let target = SetupTarget {
            series: String::from("5.2"),
            version: String::from("5.2.1"),
            executable: PathBuf::from("blender"),
            user_resources: Some(PathBuf::from("/tmp/class")),
        };
        assert_eq!(target.series_directory(), Some(PathBuf::from("/tmp/class")));
        assert_eq!(target.envs(), vec![(String::from("BLENDER_USER_RESOURCES"), String::from("/tmp/class"))]);
    }
}
