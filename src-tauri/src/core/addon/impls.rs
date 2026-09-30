use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{
    core::{
        blender_config_root, extract_json_payload, open_in_file_explorer, py_string_literal,
        resolve_blender_console_executable, run_blender_python, symlink_directory,
        BLENDERBASE_JSON_MARKER,
    },
    database::{Addon, BlenderVersion},
    AppState,
};

/// Seconds a headless Blender run may take before it is treated as hung.
const BLENDER_PYTHON_TIMEOUT_SECS: u64 = 120;

pub const ADDON_KIND_EXTENSION: &str = "extension";
pub const ADDON_KIND_ADDON: &str = "addon";
pub const ADDON_KIND_CORE: &str = "core";

/// Lists every addon and extension Blender knows about, with its enabled state, as JSON.
const LIST_ADDONS_PY: &str = r#"
import bpy, addon_utils, json, os

def _v(t):
    if isinstance(t, str):
        return t
    try:
        return ".".join(str(x) for x in t) if t else ""
    except Exception:
        return str(t)

def _islink(p):
    try:
        if os.path.islink(p):
            return True
        f = getattr(os.path, "isjunction", None)
        return bool(f and f(p))
    except Exception:
        return False

def _norm(p):
    return (p or "").replace("\\", "/").lower()

user_paths = []
for kind in ("SCRIPTS", "EXTENSIONS"):
    try:
        v = bpy.utils.user_resource(kind)
        if v:
            user_paths.append(_norm(v))
    except Exception:
        pass

enabled = set(a.module for a in bpy.context.preferences.addons)
out = []
for mod in addon_utils.modules(refresh=False):
    try:
        info = addon_utils.module_bl_info(mod)
    except Exception:
        info = {}
    name = mod.__name__
    path = getattr(mod, "__file__", "") or ""
    directory = os.path.dirname(path)
    npath = _norm(path)
    in_user = any(npath.startswith(u) for u in user_paths)
    if name.startswith("bl_ext."):
        kind = "extension"
    elif in_user:
        kind = "addon"
    else:
        kind = "core"
    is_link = _islink(directory) or _islink(path)
    out.append({
        "module": name,
        "name": str(info.get("name", "") or name),
        "author": str(info.get("author", "") or ""),
        "version": _v(info.get("version")),
        "blender": _v(info.get("blender")),
        "description": str(info.get("description", "") or ""),
        "category": str(info.get("category", "") or ""),
        "location": str(info.get("location", "") or ""),
        "warning": str(info.get("warning", "") or ""),
        "doc_url": str(info.get("doc_url", "") or info.get("wiki_url", "") or ""),
        "tracker_url": str(info.get("tracker_url", "") or ""),
        "support": str(info.get("support", "") or ""),
        "file": path,
        "dir": directory,
        "is_symlink": bool(is_link),
        "enabled": name in enabled,
        "kind": kind,
    })
# Written through the original stdout: an addon that replaces sys.stdout (e.g. a tee logger)
# would otherwise echo this line twice and corrupt the payload.
import sys
_out = getattr(sys, "__stdout__", None) or sys.stdout
_out.write("__MARKER__" + json.dumps(out) + "\n")
_out.flush()
"#;

const TOGGLE_ADDON_PY: &str = r#"
import bpy
module = __MODULE__
if __ENABLE__:
    result = bpy.ops.preferences.addon_enable(module=module)
else:
    result = bpy.ops.preferences.addon_disable(module=module)
if 'FINISHED' not in result:
    raise RuntimeError("Could not %s %s: %s" % ("enable" if __ENABLE__ else "disable", module, sorted(result)))
bpy.ops.wm.save_userpref()
print("__MARKER__" + '{"ok": true}')
"#;

const INSTALL_ADDON_PY: &str = r#"
import bpy, addon_utils, zipfile
path = __PATH__
before = set(m.__name__ for m in addon_utils.modules(refresh=False))
is_extension = False
if path.lower().endswith(".zip"):
    try:
        with zipfile.ZipFile(path) as z:
            is_extension = any(n.split("/")[-1] == "blender_manifest.toml" for n in z.namelist())
    except Exception:
        is_extension = False
if is_extension:
    if not hasattr(bpy.ops, "extensions"):
        raise RuntimeError("Extensions require Blender 4.2 or newer")
    result = bpy.ops.extensions.package_install_files(filepath=path, repo="user_default", enable_on_install=True)
    if 'FINISHED' not in result:
        raise RuntimeError("Extension install did not finish: %s" % sorted(result))
else:
    result = bpy.ops.preferences.addon_install(filepath=path, overwrite=True)
    if 'FINISHED' not in result:
        raise RuntimeError("Addon install did not finish: %s" % sorted(result))
    after = set(m.__name__ for m in addon_utils.modules(refresh=True))
    for name in sorted(after - before):
        try:
            bpy.ops.preferences.addon_enable(module=name)
        except Exception as e:
            print("Blenderbase: could not enable", name, e)
bpy.ops.wm.save_userpref()
print("__MARKER__" + '{"ok": true}')
"#;

const ENABLE_MODULE_PY: &str = r#"
import bpy, addon_utils
module = __MODULE__
if module.startswith("bl_ext.") and hasattr(bpy.ops, "extensions"):
    try:
        bpy.ops.extensions.repo_refresh_all()
    except Exception as e:
        print("Blenderbase: repo refresh failed", e)
addon_utils.modules(refresh=True)
result = bpy.ops.preferences.addon_enable(module=module)
if 'FINISHED' not in result:
    raise RuntimeError("Could not enable %s: %s" % (module, sorted(result)))
bpy.ops.wm.save_userpref()
print("__MARKER__" + '{"ok": true}')
"#;

const REMOVE_ADDON_PY: &str = r#"
import bpy
module = __MODULE__
repo_directory = __REPO_DIR__
try:
    bpy.ops.preferences.addon_disable(module=module)
except Exception as e:
    print("Blenderbase: disable failed", e)
if module.startswith("bl_ext.") and hasattr(bpy.ops, "extensions"):
    bpy.ops.extensions.package_uninstall(repo_directory=repo_directory, pkg_id=module.split(".")[-1])
else:
    try:
        bpy.ops.preferences.addon_remove(module=module)
    except Exception as e:
        # Headless, the operator deletes the files and then fails redrawing a UI area that
        # does not exist (Blender 5.x: context.area is None). Whether it removed the addon
        # is checked below instead.
        print("Blenderbase: addon_remove raised", e)
    import addon_utils
    if any(m.__name__ == module for m in addon_utils.modules(refresh=True)):
        raise RuntimeError("Could not remove %s: it is still installed" % module)
bpy.ops.wm.save_userpref()
print("__MARKER__" + '{"ok": true}')
"#;

const DISABLE_MODULE_PY: &str = r#"
import bpy
module = __MODULE__
try:
    bpy.ops.preferences.addon_disable(module=module)
except Exception as e:
    print("Blenderbase: disable failed", e)
bpy.ops.wm.save_userpref()
print("__MARKER__" + '{"ok": true}')
"#;

#[derive(Debug, Deserialize)]
struct ScannedAddon {
    module: String,
    name: String,
    author: String,
    version: String,
    blender: String,
    description: String,
    category: String,
    location: String,
    warning: String,
    doc_url: String,
    tracker_url: String,
    support: String,
    file: String,
    dir: String,
    is_symlink: bool,
    enabled: bool,
    kind: String,
}

/// What `apply_addon` did, for the caller to act on.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ApplyAddonOutcome {
    /// The addon is in place for the target series and enabled there.
    Applied,
    /// Nothing was changed: the target series already has something at `path`. Calling
    /// again with `replace` set swaps it for the source's copy.
    Exists { path: String },
}

/// How an addon reaches another series: by files placed in that series' folder, or, for
/// what Blender ships itself, by enabling the same module there.
#[derive(Debug, PartialEq)]
enum Placement {
    EnableOnly,
    Place {
        /// The addon as one unit in its own series: the package directory or the single .py file.
        unit: std::path::PathBuf,
        /// Where that unit goes in the target series.
        destination: std::path::PathBuf,
    },
}

pub trait TAddonService {
    async fn fetch_addons(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_version_id: String,
    ) -> Result<Vec<Addon>, String>;
    async fn refresh_addons(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_version_id: String,
    ) -> Result<Vec<Addon>, String>;
    async fn toggle_addon(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
        is_enabled: bool,
    ) -> Result<Addon, String>;
    async fn install_addon(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_version_id: String,
        file_path: String,
    ) -> Result<Vec<Addon>, String>;
    async fn symlink_addon(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_version_id: String,
        directory_path: String,
    ) -> Result<Vec<Addon>, String>;
    async fn delete_addon(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
    ) -> Result<Vec<Addon>, String>;
    async fn reveal_addon_in_file_explorer(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
    ) -> Result<(), String>;
    async fn apply_addon(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
        blender_version_id: String,
        replace: bool,
    ) -> Result<ApplyAddonOutcome, String>;
}

pub struct AddonServiceImpl;

impl TAddonService for AddonServiceImpl {
    async fn fetch_addons(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_version_id: String,
    ) -> Result<Vec<Addon>, String> {
        match state
            .addon_repository()
            .fetch_by_blender_version(&blender_version_id)
            .await
        {
            Ok(v) => Ok(v),
            Err(e) => Err(format!("Failed fetch_addons: {:?}", e)),
        }
    }

    async fn refresh_addons(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_version_id: String,
    ) -> Result<Vec<Addon>, String> {
        let blender_version = Self::fetch_blender_version(&state, &blender_version_id).await?;
        let executable = Self::executable_path(&blender_version)?;
        let script = LIST_ADDONS_PY.replace("__MARKER__", BLENDERBASE_JSON_MARKER);
        let stdout = run_blender_python(&executable, &script, BLENDER_PYTHON_TIMEOUT_SECS)
            .await
            .map_err(|e| format!("Failed refresh_addons: {}", e))?;
        let payload = extract_json_payload(&stdout)
            .map_err(|e| format!("Failed refresh_addons: {}", e))?;
        let scanned: Vec<ScannedAddon> = match serde_json::Deserializer::from_str(payload.trim())
            .into_iter::<Vec<ScannedAddon>>()
            .next()
        {
            Some(Ok(v)) => v,
            Some(Err(e)) => return Err(format!("Failed refresh_addons: invalid addon data: {:?}", e)),
            None => return Err(String::from("Failed refresh_addons: Blender reported no addon data")),
        };

        let repository = state.addon_repository();
        let existing = repository
            .fetch_by_blender_version(&blender_version_id)
            .await
            .map_err(|e| format!("Failed refresh_addons: {:?}", e))?;

        // Upserts and deletes land together or not at all, so a failure half
        // way through cannot leave the cached list in a mixed state.
        let mut tx = state
            .pool
            .begin()
            .await
            .map_err(|e| format!("Failed refresh_addons: {:?}", e))?;
        let mut seen: HashSet<String> = HashSet::new();
        for item in scanned {
            if item.file.is_empty() || !seen.insert(item.file.clone()) {
                continue;
            }
            let id = existing
                .iter()
                .find(|e| e.main_python_file_path == item.file)
                .map(|e| e.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let addon = Addon {
                id,
                is_enabled: item.enabled,
                is_symbolic_link: item.is_symlink,
                main_python_file_path: item.file,
                installation_directory: item.dir,
                variant_type: Some(item.kind),
                functional_name: Some(item.module),
                name: Some(item.name),
                author: Self::opt(item.author),
                version: Self::opt(item.version),
                blender_version: Self::opt(item.blender),
                location: Self::opt(item.location),
                description: Self::opt(item.description),
                warning: Self::opt(item.warning),
                documentation_url: Self::opt(item.doc_url),
                tracker_url: Self::opt(item.tracker_url),
                support: Self::opt(item.support),
                category: Self::opt(item.category),
                parent_blender_version_id: Some(blender_version_id.clone()),
                created: String::new(),
                modified: String::new(),
            };
            repository
                .insert_with(&mut *tx, &addon)
                .await
                .map_err(|e| format!("Failed refresh_addons: {:?}", e))?;
        }
        // Drop cached rows for addons Blender no longer reports.
        for previous in existing {
            if !seen.contains(&previous.main_python_file_path) {
                repository
                    .delete_with(&mut *tx, &previous.id)
                    .await
                    .map_err(|e| format!("Failed refresh_addons: {:?}", e))?;
            }
        }
        tx.commit()
            .await
            .map_err(|e| format!("Failed refresh_addons: {:?}", e))?;
        repository
            .fetch_by_blender_version(&blender_version_id)
            .await
            .map_err(|e| format!("Failed refresh_addons: {:?}", e))
    }

    async fn toggle_addon(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
        is_enabled: bool,
    ) -> Result<Addon, String> {
        let mut addon = Self::fetch_addon(&state, &id).await?;
        let module = Self::module_name(&addon)?;
        let blender_version =
            Self::fetch_blender_version(&state, &Self::parent_id(&addon)?).await?;
        let executable = Self::executable_path(&blender_version)?;
        let script = TOGGLE_ADDON_PY
            .replace("__MARKER__", BLENDERBASE_JSON_MARKER)
            .replace("__MODULE__", &py_string_literal(&module))
            .replace("__ENABLE__", if is_enabled { "True" } else { "False" });
        run_blender_python(&executable, &script, BLENDER_PYTHON_TIMEOUT_SECS)
            .await
            .map_err(|e| format!("Failed toggle_addon: {}", e))?;
        state
            .addon_repository()
            .update_is_enabled(&id, is_enabled)
            .await
            .map_err(|e| format!("Failed toggle_addon: {:?}", e))?;
        addon.is_enabled = is_enabled;
        Ok(addon)
    }

    async fn install_addon(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_version_id: String,
        file_path: String,
    ) -> Result<Vec<Addon>, String> {
        let path = std::path::PathBuf::from(&file_path);
        if !path.is_file() {
            return Err(format!("Failed install_addon: {} is not a file", file_path));
        }
        let blender_version = Self::fetch_blender_version(&state, &blender_version_id).await?;
        let executable = Self::executable_path(&blender_version)?;
        let script = INSTALL_ADDON_PY
            .replace("__MARKER__", BLENDERBASE_JSON_MARKER)
            .replace("__PATH__", &py_string_literal(&file_path));
        run_blender_python(&executable, &script, BLENDER_PYTHON_TIMEOUT_SECS)
            .await
            .map_err(|e| format!("Failed install_addon: {}", e))?;
        self.refresh_addons(app, state, blender_version_id).await
    }

    async fn symlink_addon(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        blender_version_id: String,
        directory_path: String,
    ) -> Result<Vec<Addon>, String> {
        let source = std::path::PathBuf::from(&directory_path);
        if !source.is_dir() {
            return Err(format!(
                "Failed symlink_addon: {} is not a directory",
                directory_path
            ));
        }
        let name = match source.file_name() {
            Some(v) => v.to_string_lossy().to_string(),
            None => return Err(String::from("Failed symlink_addon: directory has no name")),
        };
        let blender_version = Self::fetch_blender_version(&state, &blender_version_id).await?;
        let executable = Self::executable_path(&blender_version)?;
        let series_directory = Self::series_directory(&state, &blender_version).await?;

        let is_extension = source.join("blender_manifest.toml").is_file();
        let (parent, module) = if is_extension {
            (
                series_directory.join("extensions").join("user_default"),
                format!("bl_ext.user_default.{}", name),
            )
        } else {
            (series_directory.join("scripts").join("addons"), name.clone())
        };
        if let Err(e) = std::fs::create_dir_all(&parent) {
            return Err(format!("Failed symlink_addon: {:?}", e));
        }
        let destination = parent.join(&name);
        if destination.exists() || std::fs::symlink_metadata(&destination).is_ok() {
            return Err(format!(
                "Failed symlink_addon: {} already exists",
                destination.to_string_lossy()
            ));
        }
        // May raise a UAC prompt and wait for the answer: run it on the blocking pool.
        {
            let source = source.clone();
            let destination = destination.clone();
            match tokio::task::spawn_blocking(move || symlink_directory(&source, &destination)).await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => return Err(format!("Failed symlink_addon: {}", e)),
                Err(e) => return Err(format!("Failed symlink_addon: {:?}", e)),
            }
        }

        let script = ENABLE_MODULE_PY
            .replace("__MARKER__", BLENDERBASE_JSON_MARKER)
            .replace("__MODULE__", &py_string_literal(&module));
        if let Err(e) = run_blender_python(&executable, &script, BLENDER_PYTHON_TIMEOUT_SECS).await {
            // The link is in place; report why Blender could not enable it but keep the list current.
            let _ = self.refresh_addons(app, state, blender_version_id).await;
            return Err(format!("Symlinked, but Blender could not enable it: {}", e));
        }
        self.refresh_addons(app, state, blender_version_id).await
    }

    async fn delete_addon(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
    ) -> Result<Vec<Addon>, String> {
        let addon = Self::fetch_addon(&state, &id).await?;
        if addon.variant_type.as_deref() == Some(ADDON_KIND_CORE) {
            return Err(String::from(
                "Failed delete_addon: addons bundled with Blender cannot be removed",
            ));
        }
        let module = Self::module_name(&addon)?;
        let blender_version_id = Self::parent_id(&addon)?;
        let blender_version = Self::fetch_blender_version(&state, &blender_version_id).await?;
        let executable = Self::executable_path(&blender_version)?;

        if addon.is_symbolic_link {
            // Only the link is removed; the linked source directory stays untouched.
            let script = DISABLE_MODULE_PY
                .replace("__MARKER__", BLENDERBASE_JSON_MARKER)
                .replace("__MODULE__", &py_string_literal(&module));
            if let Err(e) = run_blender_python(&executable, &script, BLENDER_PYTHON_TIMEOUT_SECS).await {
                return Err(format!("Failed delete_addon: {}", e));
            }
            let link = Self::link_path(&addon);
            let removed = std::fs::remove_dir(&link).or_else(|_| std::fs::remove_file(&link));
            if let Err(e) = removed {
                return Err(format!(
                    "Failed delete_addon: could not remove link {}: {:?}",
                    link.to_string_lossy(),
                    e
                ));
            }
        } else {
            let repo_directory = std::path::Path::new(&addon.installation_directory)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let script = REMOVE_ADDON_PY
                .replace("__MARKER__", BLENDERBASE_JSON_MARKER)
                .replace("__MODULE__", &py_string_literal(&module))
                .replace("__REPO_DIR__", &py_string_literal(&repo_directory));
            run_blender_python(&executable, &script, BLENDER_PYTHON_TIMEOUT_SECS)
                .await
                .map_err(|e| format!("Failed delete_addon: {}", e))?;
        }
        state
            .addon_repository()
            .delete(&id)
            .await
            .map_err(|e| format!("Failed delete_addon: {:?}", e))?;
        self.refresh_addons(app, state, blender_version_id).await
    }

    async fn reveal_addon_in_file_explorer(
        &self,
        _app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
    ) -> Result<(), String> {
        let addon = Self::fetch_addon(&state, &id).await?;
        let target = if addon.is_symbolic_link || addon.installation_directory.is_empty() {
            Self::link_path(&addon)
        } else {
            std::path::PathBuf::from(&addon.installation_directory)
        };
        open_in_file_explorer(target)
            .map_err(|e| format!("Failed reveal_addon_in_file_explorer: {}", e))
    }

    /// Makes an addon of one Blender version available to another. Blender keeps user addons
    /// per series, so the files land in the target's series folder: a copy of the package
    /// directory or single file, or, for a symlinked addon, a link to the same source folder.
    /// Core addons and system extensions ship with every Blender, so those are only enabled
    /// on the target. The target build then enables the module and its cached list is re-read.
    async fn apply_addon(
        &self,
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: String,
        blender_version_id: String,
        replace: bool,
    ) -> Result<ApplyAddonOutcome, String> {
        let addon = Self::fetch_addon(&state, &id).await?;
        let module = Self::module_name(&addon)?;
        let label = addon
            .name
            .clone()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| module.clone());
        let source_version =
            Self::fetch_blender_version(&state, &Self::parent_id(&addon)?).await?;
        let target_version = Self::fetch_blender_version(&state, &blender_version_id).await?;
        let target_series = match &target_version.series {
            Some(v) if !v.trim().is_empty() => v.trim().to_string(),
            _ => return Err(String::from("Failed apply_addon: Blender version has no series")),
        };
        if source_version.series.as_deref().map(str::trim) == Some(target_series.as_str()) {
            return Err(format!(
                "Failed apply_addon: Blender {} and {} are both {} builds and share their addons already",
                source_version.version.clone().unwrap_or_default(),
                target_version.version.clone().unwrap_or_default(),
                target_series
            ));
        }
        let executable = Self::executable_path(&target_version)?;
        let series_directory = Self::series_directory(&state, &target_version).await?;

        match Self::plan_placement(&addon, &module, &target_series, &series_directory)? {
            Placement::EnableOnly => {}
            Placement::Place { unit, destination } => {
                if !unit.exists() {
                    return Err(format!(
                        "Failed apply_addon: {} is no longer at {}; re-read the addons from Blender",
                        label,
                        unit.display()
                    ));
                }
                if std::fs::symlink_metadata(&destination).is_ok() {
                    if !replace {
                        return Ok(ApplyAddonOutcome::Exists {
                            path: destination.to_string_lossy().to_string(),
                        });
                    }
                    Self::remove_at(&destination).map_err(|e| format!("Failed apply_addon: {}", e))?;
                }
                if let Some(parent) = destination.parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        return Err(format!(
                            "Failed apply_addon: could not create {}: {:?}",
                            parent.display(),
                            e
                        ));
                    }
                }
                if addon.is_symbolic_link && unit.is_dir() {
                    // Link the target series to the folder the source series links to, not to
                    // the link itself, so the target keeps working when the source is unlinked.
                    let source = std::fs::read_link(&unit).unwrap_or_else(|_| unit.clone());
                    let link = destination.clone();
                    // May raise a UAC prompt and wait for the answer: run it on the blocking pool.
                    match tokio::task::spawn_blocking(move || symlink_directory(&source, &link)).await {
                        Ok(Ok(())) => {}
                        Ok(Err(e)) => return Err(format!("Failed apply_addon: {}", e)),
                        Err(e) => return Err(format!("Failed apply_addon: {:?}", e)),
                    }
                } else if unit.is_dir() {
                    Self::copy_tree(&unit, &destination).map_err(|e| format!("Failed apply_addon: {}", e))?;
                } else if let Err(e) = std::fs::copy(&unit, &destination) {
                    return Err(format!(
                        "Failed apply_addon: could not copy {}: {:?}",
                        unit.display(),
                        e
                    ));
                }
            }
        }

        let script = ENABLE_MODULE_PY
            .replace("__MARKER__", BLENDERBASE_JSON_MARKER)
            .replace("__MODULE__", &py_string_literal(&module));
        if let Err(e) = run_blender_python(&executable, &script, BLENDER_PYTHON_TIMEOUT_SECS).await {
            // The files are in place; report why Blender could not enable them but keep the list current.
            let _ = self.refresh_addons(app, state, blender_version_id).await;
            return Err(format!(
                "Blender {} could not enable {}: {}",
                target_version.version.clone().unwrap_or_default(),
                label,
                e
            ));
        }
        self.refresh_addons(app, state, blender_version_id).await?;
        Ok(ApplyAddonOutcome::Applied)
    }
}

impl AddonServiceImpl {
    fn opt(value: String) -> Option<String> {
        if value.trim().is_empty() {
            None
        } else {
            Some(value)
        }
    }

    fn module_name(addon: &Addon) -> Result<String, String> {
        match &addon.functional_name {
            Some(v) if !v.is_empty() => Ok(v.clone()),
            _ => Err(String::from("Addon has no module name")),
        }
    }

    fn parent_id(addon: &Addon) -> Result<String, String> {
        match &addon.parent_blender_version_id {
            Some(v) if !v.is_empty() => Ok(v.clone()),
            _ => Err(String::from("Addon is not linked to a Blender version")),
        }
    }

    /// The path that is itself the symlink: the package directory, or the single .py file.
    fn link_path(addon: &Addon) -> std::path::PathBuf {
        let file = std::path::Path::new(&addon.main_python_file_path);
        if file
            .file_name()
            .map(|n| n.to_string_lossy().eq_ignore_ascii_case("__init__.py"))
            .unwrap_or(false)
        {
            std::path::PathBuf::from(&addon.installation_directory)
        } else {
            file.to_path_buf()
        }
    }

    /// Where the addon's files go in the target series, or `EnableOnly` for what Blender ships
    /// itself (core addons, system extensions), which every version carries in its own copy.
    fn plan_placement(
        addon: &Addon,
        module: &str,
        target_series: &str,
        target_series_directory: &std::path::Path,
    ) -> Result<Placement, String> {
        let unit = Self::link_path(addon);
        let unit_name = match unit.file_name() {
            Some(v) => v.to_os_string(),
            None => return Err(String::from("Addon has no file name")),
        };
        match addon.variant_type.as_deref() {
            Some(ADDON_KIND_CORE) => Ok(Placement::EnableOnly),
            Some(ADDON_KIND_EXTENSION) => {
                let (repository, package) = match Self::extension_module_parts(module) {
                    Some(v) => v,
                    None => return Err(format!("{} is not an extension module name", module)),
                };
                if repository == "system" {
                    return Ok(Placement::EnableOnly);
                }
                if !Self::series_supports_extensions(target_series) {
                    return Err(format!(
                        "Extensions need Blender 4.2 or newer; Blender {} cannot use {}",
                        target_series, module
                    ));
                }
                Ok(Placement::Place {
                    unit,
                    destination: target_series_directory
                        .join("extensions")
                        .join(repository)
                        .join(package),
                })
            }
            _ => Ok(Placement::Place {
                unit,
                destination: target_series_directory
                    .join("scripts")
                    .join("addons")
                    .join(unit_name),
            }),
        }
    }

    /// `bl_ext.<repository>.<package>` split into the repository module and the package id.
    fn extension_module_parts(module: &str) -> Option<(&str, &str)> {
        let rest = module.strip_prefix("bl_ext.")?;
        let (repository, package) = rest.split_once('.')?;
        if repository.is_empty() || package.is_empty() || package.contains('.') {
            return None;
        }
        Some((repository, package))
    }

    /// Extensions arrived with Blender 4.2; `series` is `major.minor`.
    fn series_supports_extensions(series: &str) -> bool {
        let mut parts = series.trim().split('.');
        let major: u32 = match parts.next().and_then(|v| v.trim().parse().ok()) {
            Some(v) => v,
            None => return false,
        };
        let minor: u32 = parts.next().and_then(|v| v.trim().parse().ok()).unwrap_or(0);
        (major, minor) >= (4, 2)
    }

    /// Removes whatever is at `path`: a link is unlinked and its target kept, a directory
    /// goes with its contents, a file is deleted.
    fn remove_at(path: &std::path::Path) -> Result<(), String> {
        let metadata = std::fs::symlink_metadata(path)
            .map_err(|e| format!("could not read {}: {:?}", path.display(), e))?;
        let removed = if metadata.file_type().is_symlink() {
            std::fs::remove_dir(path).or_else(|_| std::fs::remove_file(path))
        } else if metadata.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        };
        removed.map_err(|e| format!("could not remove {}: {:?}", path.display(), e))
    }

    /// Copies a directory tree, following links and skipping `__pycache__`. The file-system
    /// utilities have a copier that recreates links, but it is unix-only, and links are of no
    /// use inside an addon package.
    fn copy_tree(source: &std::path::Path, destination: &std::path::Path) -> Result<(), String> {
        std::fs::create_dir_all(destination)
            .map_err(|e| format!("could not create {}: {:?}", destination.display(), e))?;
        let entries = std::fs::read_dir(source)
            .map_err(|e| format!("could not read {}: {:?}", source.display(), e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("could not read {}: {:?}", source.display(), e))?;
            if entry.file_name() == "__pycache__" {
                continue;
            }
            let from = entry.path();
            let to = destination.join(entry.file_name());
            if from.is_dir() {
                Self::copy_tree(&from, &to)?;
            } else if let Err(e) = std::fs::copy(&from, &to) {
                return Err(format!("could not copy {}: {:?}", from.display(), e));
            }
        }
        Ok(())
    }

    /// The console executable to run scripts with (never the launcher, which swallows output).
    fn executable_path(blender_version: &BlenderVersion) -> Result<std::path::PathBuf, String> {
        match &blender_version.executable_file_path {
            Some(v) if !v.is_empty() => Ok(resolve_blender_console_executable(std::path::Path::new(v))),
            _ => Err(format!(
                "Blender {} has no executable path",
                blender_version.version.clone().unwrap_or_default()
            )),
        }
    }

    async fn fetch_addon(state: &tauri::State<'_, AppState>, id: &str) -> Result<Addon, String> {
        let mut entries = state
            .addon_repository()
            .fetch(Some(id.to_string()))
            .await
            .map_err(|e| format!("Failed to fetch addon: {:?}", e))?;
        if entries.is_empty() {
            return Err(String::from("Addon not found"));
        }
        Ok(entries.remove(0))
    }

    async fn fetch_blender_version(
        state: &tauri::State<'_, AppState>,
        id: &str,
    ) -> Result<BlenderVersion, String> {
        let mut entries = state
            .blender_version_repository()
            .fetch(Some(id.to_string()), None, None, None, None)
            .await
            .map_err(|e| format!("Failed to fetch Blender version: {:?}", e))?;
        if entries.is_empty() {
            return Err(String::from("Blender version not found"));
        }
        Ok(entries.remove(0))
    }

    /// The series folder for a stored `blender_series.config_directory_path`. That column
    /// holds the `config` folder inside the series folder (`.../Blender/5.2/config`), so its
    /// parent is returned; a path that does not end in `config` is taken as the series folder.
    fn series_directory_from_config_path(config_directory_path: &str) -> std::path::PathBuf {
        let path = std::path::Path::new(config_directory_path.trim());
        let is_config = path
            .file_name()
            .map(|n| n.to_string_lossy().eq_ignore_ascii_case("config"))
            .unwrap_or(false);
        match path.parent() {
            Some(parent) if is_config && !parent.as_os_str().is_empty() => parent.to_path_buf(),
            _ => path.to_path_buf(),
        }
    }

    /// The user folder of the version's series (`.../Blender/5.2`), directly under which
    /// Blender keeps `scripts/addons` and `extensions` next to `config`. Falls back to the
    /// platform default location.
    async fn series_directory(
        state: &tauri::State<'_, AppState>,
        blender_version: &BlenderVersion,
    ) -> Result<std::path::PathBuf, String> {
        let series = match &blender_version.series {
            Some(v) if !v.is_empty() => v.clone(),
            _ => return Err(String::from("Blender version has no series")),
        };
        let known = state
            .blender_series_repository()
            .fetch(None, None, Some(series.clone()), None)
            .await
            .map_err(|e| format!("Failed to fetch Blender series: {:?}", e))?;
        if let Some(entry) = known
            .into_iter()
            .find(|s| !s.config_directory_path.trim().is_empty())
        {
            return Ok(Self::series_directory_from_config_path(
                &entry.config_directory_path,
            ));
        }
        match blender_config_root() {
            Some(b) => Ok(b.join(series)),
            None => Err(String::from("Could not determine the Blender config directory")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn series_directory_is_the_parent_of_the_stored_config_folder() {
        let series = std::path::PathBuf::from("root").join("Blender").join("5.2");
        let config = series.join("config");
        assert_eq!(
            AddonServiceImpl::series_directory_from_config_path(&config.to_string_lossy()),
            series
        );
        // Blender looks for addons next to `config`, never inside it.
        assert_eq!(
            AddonServiceImpl::series_directory_from_config_path(&config.to_string_lossy())
                .join("scripts")
                .join("addons"),
            series.join("scripts").join("addons")
        );
        let upper = series.join("Config");
        assert_eq!(
            AddonServiceImpl::series_directory_from_config_path(&upper.to_string_lossy()),
            series
        );
    }

    #[test]
    fn series_directory_keeps_a_path_that_is_not_a_config_folder() {
        let series = std::path::PathBuf::from("root").join("Blender").join("5.2");
        assert_eq!(
            AddonServiceImpl::series_directory_from_config_path(&series.to_string_lossy()),
            series
        );
        assert_eq!(
            AddonServiceImpl::series_directory_from_config_path("config"),
            std::path::PathBuf::from("config")
        );
    }

    fn addon(kind: &str, module: &str, file: std::path::PathBuf, directory: std::path::PathBuf) -> Addon {
        Addon {
            variant_type: Some(kind.to_string()),
            functional_name: Some(module.to_string()),
            main_python_file_path: file.to_string_lossy().to_string(),
            installation_directory: directory.to_string_lossy().to_string(),
            ..Default::default()
        }
    }

    fn user(series: &str) -> std::path::PathBuf {
        std::path::PathBuf::from("user").join(series)
    }

    #[test]
    fn a_package_addon_lands_in_the_target_scripts_addons_folder() {
        let source = user("4.4").join("scripts").join("addons").join("my_tool");
        let target = user("5.0");
        let a = addon(ADDON_KIND_ADDON, "my_tool", source.join("__init__.py"), source.clone());
        assert_eq!(
            AddonServiceImpl::plan_placement(&a, "my_tool", "5.0", &target).unwrap(),
            Placement::Place {
                unit: source,
                destination: target.join("scripts").join("addons").join("my_tool"),
            }
        );
    }

    #[test]
    fn a_single_file_addon_is_copied_as_that_file() {
        let addons = user("4.4").join("scripts").join("addons");
        let target = user("5.0");
        let a = addon(ADDON_KIND_ADDON, "quick_tool", addons.join("quick_tool.py"), addons.clone());
        assert_eq!(
            AddonServiceImpl::plan_placement(&a, "quick_tool", "5.0", &target).unwrap(),
            Placement::Place {
                unit: addons.join("quick_tool.py"),
                destination: target.join("scripts").join("addons").join("quick_tool.py"),
            }
        );
    }

    #[test]
    fn an_extension_keeps_its_repository_in_the_target_series() {
        let source = user("4.4").join("extensions").join("user_default").join("my_ext");
        let target = user("4.5");
        let module = "bl_ext.user_default.my_ext";
        let a = addon(ADDON_KIND_EXTENSION, module, source.join("__init__.py"), source.clone());
        assert_eq!(
            AddonServiceImpl::plan_placement(&a, module, "4.5", &target).unwrap(),
            Placement::Place {
                unit: source,
                destination: target.join("extensions").join("user_default").join("my_ext"),
            }
        );
    }

    #[test]
    fn an_extension_is_refused_for_a_series_before_4_2() {
        let source = user("4.4").join("extensions").join("user_default").join("my_ext");
        let module = "bl_ext.user_default.my_ext";
        let a = addon(ADDON_KIND_EXTENSION, module, source.join("__init__.py"), source);
        let error = AddonServiceImpl::plan_placement(&a, module, "3.6", &user("3.6")).unwrap_err();
        assert!(error.contains("4.2"), "{}", error);
    }

    #[test]
    fn what_blender_ships_is_only_enabled_on_the_target() {
        let bundled = std::path::PathBuf::from("blender").join("5.0");
        let core = addon(
            ADDON_KIND_CORE,
            "io_scene_obj",
            bundled.join("io_scene_obj").join("__init__.py"),
            bundled.join("io_scene_obj"),
        );
        assert_eq!(
            AddonServiceImpl::plan_placement(&core, "io_scene_obj", "5.0", &user("5.0")).unwrap(),
            Placement::EnableOnly
        );
        let system = addon(
            ADDON_KIND_EXTENSION,
            "bl_ext.system.hydra_storm",
            bundled.join("hydra_storm").join("__init__.py"),
            bundled.join("hydra_storm"),
        );
        assert_eq!(
            AddonServiceImpl::plan_placement(&system, "bl_ext.system.hydra_storm", "5.0", &user("5.0")).unwrap(),
            Placement::EnableOnly
        );
    }

    #[test]
    fn extension_module_names_split_into_repository_and_package() {
        assert_eq!(
            AddonServiceImpl::extension_module_parts("bl_ext.user_default.my_ext"),
            Some(("user_default", "my_ext"))
        );
        assert_eq!(
            AddonServiceImpl::extension_module_parts("bl_ext.blender_org.node_wrangler"),
            Some(("blender_org", "node_wrangler"))
        );
        assert_eq!(AddonServiceImpl::extension_module_parts("my_tool"), None);
        assert_eq!(AddonServiceImpl::extension_module_parts("bl_ext.user_default"), None);
        assert_eq!(AddonServiceImpl::extension_module_parts("bl_ext.a.b.c"), None);
    }

    #[test]
    fn extensions_need_series_4_2_or_newer() {
        assert!(AddonServiceImpl::series_supports_extensions("4.2"));
        assert!(AddonServiceImpl::series_supports_extensions("4.5"));
        assert!(AddonServiceImpl::series_supports_extensions("5.0"));
        assert!(!AddonServiceImpl::series_supports_extensions("4.1"));
        assert!(!AddonServiceImpl::series_supports_extensions("3.6"));
        assert!(!AddonServiceImpl::series_supports_extensions(""));
    }

    #[test]
    fn copy_tree_copies_files_and_folders_but_not_pycache() {
        let root = std::env::temp_dir().join(format!("blenderbase-copy-{}", uuid::Uuid::new_v4()));
        let source = root.join("src");
        std::fs::create_dir_all(source.join("sub")).unwrap();
        std::fs::create_dir_all(source.join("__pycache__")).unwrap();
        std::fs::write(source.join("__init__.py"), "bl_info = {}").unwrap();
        std::fs::write(source.join("sub").join("ops.py"), "pass").unwrap();
        std::fs::write(source.join("__pycache__").join("x.pyc"), "").unwrap();
        let destination = root.join("dst").join("pkg");
        AddonServiceImpl::copy_tree(&source, &destination).unwrap();
        assert_eq!(
            std::fs::read_to_string(destination.join("__init__.py")).unwrap(),
            "bl_info = {}"
        );
        assert_eq!(
            std::fs::read_to_string(destination.join("sub").join("ops.py")).unwrap(),
            "pass"
        );
        assert!(!destination.join("__pycache__").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn remove_at_deletes_a_folder_with_its_contents_and_a_file() {
        let root = std::env::temp_dir().join(format!("blenderbase-remove-{}", uuid::Uuid::new_v4()));
        let folder = root.join("pkg");
        std::fs::create_dir_all(folder.join("sub")).unwrap();
        std::fs::write(folder.join("sub").join("a.py"), "").unwrap();
        let file = root.join("single.py");
        std::fs::write(&file, "").unwrap();
        AddonServiceImpl::remove_at(&folder).unwrap();
        AddonServiceImpl::remove_at(&file).unwrap();
        assert!(!folder.exists() && !file.exists());
        let _ = std::fs::remove_dir_all(&root);
    }
}
