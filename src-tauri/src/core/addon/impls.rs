use std::collections::HashSet;

use serde::Deserialize;
use tauri::AppHandle;

use crate::{
    core::{
        extract_json_payload, open_in_file_explorer, py_string_literal,
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
    bpy.ops.preferences.addon_remove(module=module)
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
        let config_directory = Self::config_directory(&state, &blender_version).await?;

        let is_extension = source.join("blender_manifest.toml").is_file();
        let (parent, module) = if is_extension {
            (
                config_directory.join("extensions").join("user_default"),
                format!("bl_ext.user_default.{}", name),
            )
        } else {
            (config_directory.join("scripts").join("addons"), name.clone())
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

    /// The user configuration directory of the version's series, where Blender keeps
    /// `scripts/addons` and `extensions`. Falls back to the platform default location.
    async fn config_directory(
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
            return Ok(std::path::PathBuf::from(entry.config_directory_path));
        }
        #[cfg(target_os = "windows")]
        let base = dirs::config_dir().map(|d| d.join("Blender Foundation").join("Blender"));
        #[cfg(target_os = "macos")]
        let base = dirs::config_dir().map(|d| d.join("Blender"));
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let base = dirs::config_dir().map(|d| d.join("blender"));
        match base {
            Some(b) => Ok(b.join(series)),
            None => Err(String::from("Could not determine the Blender config directory")),
        }
    }
}
