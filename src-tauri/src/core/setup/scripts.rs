/// Reads one series' setup out of Blender: portable preferences, the active theme, changed
/// keymaps, extension repositories and every addon with where it came from. Nothing is
/// written to the user's configuration. Files land in `__OUT_DIR__`; the rest is JSON.
pub const CAPTURE_SETUP_PY: &str = r#"
import bpy, addon_utils, json, os, sys

out_dir = __OUT_DIR__
warnings = []

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

# Preferences: every writable value of the portable groups. Paths are skipped, they
# mean nothing on another computer.
PATH_SUBTYPES = {"FILE_PATH", "DIR_PATH", "FILE_NAME"}

def _walk(struct, depth):
    found = {}
    for prop in struct.bl_rna.properties:
        pid = prop.identifier
        if pid == "rna_type":
            continue
        kind = prop.type
        try:
            if kind == "POINTER":
                child = getattr(struct, pid) if depth < 3 else None
                value = _walk(child, depth + 1) if child is not None else None
                if not value:
                    continue
            elif prop.is_readonly:
                continue
            elif kind in ("BOOLEAN", "INT", "FLOAT"):
                value = getattr(struct, pid)
                if getattr(prop, "is_array", False):
                    value = list(value)
            elif kind == "STRING":
                if prop.subtype in PATH_SUBTYPES:
                    continue
                value = getattr(struct, pid)
            elif kind == "ENUM":
                value = getattr(struct, pid)
                if prop.is_enum_flag:
                    value = sorted(value)
            else:
                continue
        except Exception as e:
            warnings.append("preference %s: %s" % (pid, e))
            continue
        found[pid] = value
    return found

prefs = bpy.context.preferences
preferences = {}
for group in __PREFERENCE_GROUPS__:
    struct = getattr(prefs, group, None)
    if struct is not None:
        preferences[group] = _walk(struct, 0)

# Theme: the same XML Blender writes for a theme preset.
theme = None
try:
    try:
        import _rna_xml as rna_xml
    except ImportError:
        import rna_xml
    menu = getattr(bpy.types, "USERPREF_MT_interface_theme_presets", None)
    xml_map = getattr(menu, "preset_xml_map", None) or (
        ("preferences.themes[0]", "Theme"),
        ("preferences.ui_styles[0]", "ThemeStyle"),
    )
    path = os.path.join(out_dir, "theme.xml")
    rna_xml.xml_file_write(bpy.context, path, xml_map)
    theme = {"name": str(getattr(prefs.themes[0], "name", "") or ""), "file": path}
except Exception as e:
    warnings.append("theme: %s" % e)

# Keymap: background mode starts without the default key configuration, so it is
# loaded first; only then can Blender tell which keymaps the user changed.
keymap = None
try:
    wm = bpy.context.window_manager
    bpy.utils.keyconfig_init()
    wm.keyconfigs.update()
    modified = [km.name for km in wm.keyconfigs.user.keymaps if km.is_user_modified]
    keymap = {"name": wm.keyconfigs.active.name, "modified": modified, "file": None}
    if modified:
        from bl_keymap_utils.io import keyconfig_export_as_data
        path = os.path.join(out_dir, "keymap.py")
        keyconfig_export_as_data(wm, wm.keyconfigs.active, path, all_keymaps=False)
        keymap["file"] = path
except Exception as e:
    warnings.append("keymap: %s" % e)

# Extension repositories (Blender 4.2 and newer). Access tokens are never read.
repos = []
extensions = getattr(prefs, "extensions", None)
if extensions is not None:
    for repo in extensions.repos:
        remote = bool(getattr(repo, "use_remote_url", False))
        repos.append({
            "module": repo.module,
            "name": repo.name,
            "enabled": bool(repo.enabled),
            "remote_url": str(getattr(repo, "remote_url", "") or "") if remote else "",
            "source": str(getattr(repo, "source", "USER") or "USER"),
            "needs_token": bool(getattr(repo, "use_access_token", False)),
        })

user_paths = []
for kind in ("SCRIPTS", "EXTENSIONS"):
    try:
        v = bpy.utils.user_resource(kind)
        if v:
            user_paths.append(_norm(v))
    except Exception:
        pass

enabled = set(a.module for a in prefs.addons)
addons = []
for mod in addon_utils.modules(refresh=False):
    try:
        info = addon_utils.module_bl_info(mod)
    except Exception:
        info = {}
    name = mod.__name__
    path = getattr(mod, "__file__", "") or ""
    directory = os.path.dirname(path)
    repo = ""
    package = ""
    if name.startswith("bl_ext."):
        kind = "extension"
        parts = name.split(".")
        repo = parts[1] if len(parts) > 1 else ""
        package = parts[2] if len(parts) > 2 else ""
    elif any(_norm(path).startswith(u) for u in user_paths):
        kind = "addon"
    else:
        kind = "core"
    addons.append({
        "module": name,
        "kind": kind,
        "repo": repo,
        "package": package,
        "name": str(info.get("name", "") or name),
        "version": _v(info.get("version")),
        "enabled": name in enabled,
        "is_symlink": bool(_islink(directory) or _islink(path)),
        "file": path,
        "dir": directory,
    })

result = {
    "blender_version": ".".join(str(x) for x in bpy.app.version),
    "preferences": preferences,
    "theme": theme,
    "keymap": keymap,
    "repos": repos,
    "addons": addons,
    "warnings": warnings,
}
# Written through the original stdout: an addon that replaces sys.stdout (e.g. a tee logger)
# would otherwise echo this line twice and corrupt the payload.
_out = getattr(sys, "__stdout__", None) or sys.stdout
_out.write("__MARKER__" + json.dumps(result) + "\n")
_out.flush()
"#;
