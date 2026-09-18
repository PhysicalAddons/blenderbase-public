use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::{
    backup::{backup_series, list_backups, restore_backup},
    bundle::extract_bundle_blob,
    impls::{SetupProgress, PORTABLE_PREFERENCE_GROUPS},
    impls::compare_versions,
    manifest::{SetupAddon, SetupAddonSource, SetupBlob, SetupSeries, SETUP_ADDON_REASON_FILES_NOT_INCLUDED},
    scripts::{APPLY_SETUP_PY, RESTORE_ADDONS_PY},
};
use crate::core::{
    blender_config_root, extract_json_payload, py_string_literal, run_blender_python_with_env,
    run_blender_with_env, ADDON_KIND_EXTENSION, BLENDERBASE_JSON_MARKER, BLENDER_USER_RESOURCES,
    COM_PHYSICALADDONS_BLENDERBASE,
};

/// Seconds Blender may take to write one series' preferences, theme and keymap.
const APPLY_TIMEOUT_SECS: u64 = 120;
/// Seconds for installing the addons that come from files: no network, but Blender imports
/// each one once.
const RESTORE_ADDONS_TIMEOUT_SECS: u64 = 300;
/// Seconds for downloading and installing extensions from their repositories.
const REPOSITORY_INSTALL_TIMEOUT_SECS: u64 = 900;
/// The first Blender with extension repositories and the extension command line.
const FIRST_EXTENSIONS_VERSION: &str = "4.2.0";
/// The repository Blender installs extensions into when they come from a file.
const USER_DEFAULT_REPOSITORY: &str = "user_default";
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

/// What to take from one series of a setup.
#[derive(Debug, Clone, Deserialize)]
pub struct SeriesApplyChoice {
    pub series: String,
    #[serde(default = "yes")]
    pub preferences: bool,
    #[serde(default = "yes")]
    pub theme: bool,
    #[serde(default = "yes")]
    pub keymap: bool,
    #[serde(default = "yes")]
    pub addons: bool,
}

impl SeriesApplyChoice {
    pub fn everything(series: &str) -> Self {
        Self { series: series.to_string(), preferences: true, theme: true, keymap: true, addons: true }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SetupApplyOptions {
    /// The series to apply and what of each; every series with everything when empty.
    #[serde(default)]
    pub choices: Vec<SeriesApplyChoice>,
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
    /// Names of addons installed from files or repositories.
    pub addons_installed: Vec<String>,
    /// `name: reason` for each addon that could not be installed.
    pub addons_failed: Vec<String>,
    /// `name (reason)` for each addon the user has to install by hand.
    pub addons_manual: Vec<String>,
    pub repositories_added: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RestoredAddonFailure {
    module: String,
    error: String,
}

#[derive(Debug, Deserialize)]
struct RestoredAddons {
    #[serde(default)]
    repositories_added: Vec<String>,
    /// Remote URL to the local module that serves it, after adding.
    #[serde(default)]
    repositories: HashMap<String, String>,
    #[serde(default)]
    installed: Vec<String>,
    #[serde(default)]
    failed: Vec<RestoredAddonFailure>,
    #[serde(default)]
    missing: Vec<String>,
    #[serde(default)]
    warnings: Vec<String>,
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
    choice: &SeriesApplyChoice,
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
        (wanted(choice.preferences, &section.preferences), String::from("preferences.json")),
        (wanted(choice.theme, &section.theme), format!("{}.xml", restored_theme_name(&theme_name))),
        (wanted(choice.keymap, &section.keymap), String::from("keymap.py")),
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

    if choice.addons {
        restore_addons(bundle_path, section, target, work_directory, progress, &mut report).await?;
    }
    Ok(report)
}

/// Addons in three groups: files from the bundle and enabled states go through one Blender
/// run; extensions of remote repositories are then downloaded by Blender's own extension
/// command line; whatever cannot be restored is listed for the user.
async fn restore_addons(
    bundle_path: &Path,
    section: &SetupSeries,
    target: &SetupTarget,
    work_directory: &Path,
    progress: &SetupProgress,
    report: &mut SeriesApplyReport,
) -> Result<(), String> {
    let name_of = |addon: &SetupAddon| if addon.name.trim().is_empty() { addon.module.clone() } else { addon.name.clone() };
    let mut files: Vec<serde_json::Value> = Vec::new();
    let mut states: BTreeMap<String, bool> = BTreeMap::new();
    let mut by_repository: Vec<&SetupAddon> = Vec::new();
    for addon in &section.addons {
        match addon.source {
            SetupAddonSource::Core => {
                states.insert(addon.module.clone(), addon.enabled);
            }
            SetupAddonSource::Repo => by_repository.push(addon),
            SetupAddonSource::File => match &addon.file {
                Some(file) => {
                    let file_name = file.name.clone().unwrap_or_default();
                    let destination = work_directory.join(&target.series).join("addons").join(&file_name);
                    let (bundle_path, reference, moved) = (bundle_path.to_path_buf(), file.blob.clone(), destination.clone());
                    tokio::task::spawn_blocking(move || extract_bundle_blob(&bundle_path, &reference, &moved))
                        .await
                        .map_err(|e| format!("Failed to read the setup file: {:?}", e))??;
                    // An extension from a file always lands in the user repository, whatever
                    // repository it sat in on the other computer.
                    let module = if addon.kind.as_deref() == Some(ADDON_KIND_EXTENSION) {
                        let package = addon
                            .package
                            .clone()
                            .unwrap_or_else(|| addon.module.rsplit('.').next().unwrap_or_default().to_string());
                        format!("bl_ext.{}.{}", USER_DEFAULT_REPOSITORY, package)
                    } else {
                        addon.module.clone()
                    };
                    files.push(serde_json::json!({ "path": destination.to_string_lossy(), "module": module }));
                    states.insert(module, addon.enabled);
                }
                None => report.addons_manual.push(format!(
                    "{} ({})",
                    name_of(addon),
                    addon.reason.clone().unwrap_or_else(|| String::from(SETUP_ADDON_REASON_FILES_NOT_INCLUDED))
                )),
            },
            SetupAddonSource::Manual => report.addons_manual.push(format!(
                "{} ({})",
                name_of(addon),
                addon.reason.clone().unwrap_or_else(|| String::from("cannot be restored automatically"))
            )),
        }
    }
    let mut repositories: Vec<serde_json::Value> = Vec::new();
    for repository in &section.repositories {
        if repository.needs_token {
            report.warnings.push(format!(
                "Repository {} needs an access token: add it in Blender's preferences, then apply again",
                repository.name
            ));
            continue;
        }
        repositories.push(serde_json::json!({ "module": repository.module, "name": repository.name, "url": repository.url }));
    }

    let as_json = |value: &serde_json::Value| py_string_literal(&value.to_string());
    let script = RESTORE_ADDONS_PY
        .replace("__MARKER__", BLENDERBASE_JSON_MARKER)
        .replace("__REPOSITORIES__", &as_json(&serde_json::Value::Array(repositories)))
        .replace("__FILES__", &as_json(&serde_json::Value::Array(files)))
        .replace("__STATES__", &as_json(&serde_json::to_value(&states).unwrap_or_default()));
    progress(format!("Blender {}: installing addons from the setup file…", target.series));
    let stdout = run_blender_python_with_env(&target.executable, &script, RESTORE_ADDONS_TIMEOUT_SECS, &target.envs()).await?;
    let restored: RestoredAddons = match serde_json::Deserializer::from_str(extract_json_payload(&stdout)?.trim())
        .into_iter::<RestoredAddons>()
        .next()
    {
        Some(Ok(v)) => v,
        Some(Err(e)) => return Err(format!("Blender reported an unreadable addon result: {}", e)),
        None => return Err(String::from("Blender reported no addon result")),
    };
    let name_by_module: HashMap<&str, String> = section
        .addons
        .iter()
        .map(|a| (a.module.as_str(), name_of(a)))
        .collect();
    let display = |module: &str| {
        name_by_module
            .get(module)
            .cloned()
            .or_else(|| {
                // A file extension's module changed to the user repository; find it by id.
                let package = module.rsplit('.').next().unwrap_or_default();
                section
                    .addons
                    .iter()
                    .find(|a| a.package.as_deref() == Some(package))
                    .map(name_of)
            })
            .unwrap_or_else(|| module.to_string())
    };
    report.repositories_added = restored.repositories_added;
    report.addons_installed.extend(restored.installed.iter().map(|m| display(m)));
    report.addons_failed.extend(restored.failed.iter().map(|f| format!("{}: {}", display(&f.module), f.error)));
    report.warnings.extend(restored.missing.iter().map(|m| format!("{} was not found after installing", display(m))));
    report.warnings.extend(restored.warnings);

    if by_repository.is_empty() {
        return Ok(());
    }
    if compare_versions(&target.version, FIRST_EXTENSIONS_VERSION) == std::cmp::Ordering::Less {
        report.addons_manual.extend(by_repository.iter().map(|a| format!("{} (extensions need Blender 4.2 or newer)", name_of(a))));
        return Ok(());
    }
    // The manifest names repositories by their module on the other computer; here the same
    // URL may sit under another module, so ids are built from what this Blender reports.
    let local_module = |repository: &str| -> String {
        section
            .repositories
            .iter()
            .find(|r| r.module == repository)
            .and_then(|r| restored.repositories.get(r.url.trim_end_matches('/')))
            .cloned()
            .unwrap_or_else(|| repository.to_string())
    };
    for enabled in [true, false] {
        let group: Vec<&SetupAddon> = by_repository.iter().copied().filter(|a| a.enabled == enabled).collect();
        if group.is_empty() {
            continue;
        }
        let ids: Vec<String> = group
            .iter()
            .filter_map(|a| {
                let (repository, package) = (a.repository.as_deref()?, a.package.as_deref()?);
                Some(format!("{}.{}", local_module(repository), package))
            })
            .collect();
        progress(format!(
            "Blender {}: downloading {} from repositories…",
            target.series,
            if ids.len() == 1 { String::from("1 extension") } else { format!("{} extensions", ids.len()) }
        ));
        // `--online-mode` lets this one run download although the preference may still be off
        // on a fresh installation; the preference itself is left alone.
        let mut args = vec![
            String::from("--online-mode"),
            String::from("--command"),
            String::from("extension"),
            String::from("install"),
            String::from("--sync"),
        ];
        if enabled {
            args.push(String::from("--enable"));
        }
        args.push(ids.join(","));
        match run_blender_with_env(&target.executable, &args, REPOSITORY_INSTALL_TIMEOUT_SECS, &target.envs()).await {
            Ok(_) => report.addons_installed.extend(group.iter().map(|a| name_of(a))),
            Err(e) => report.addons_failed.extend(group.iter().map(|a| format!("{}: {}", name_of(a), e))),
        }
    }
    Ok(())
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
