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
    # A key configuration other than the default carries its bindings itself, changed or not.
    if modified or wm.keyconfigs.active.name != "Blender":
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

/// Writes preferences, theme and keymap of a setup into one series and saves the user
/// preferences. Every input is data from outside: preference names are looked up before they
/// are set, the theme goes through Blender's own installer, and the keymap file is parsed as
/// literals, never executed.
pub const APPLY_SETUP_PY: &str = r#"
import bpy, json, os, sys

preferences_file = __PREFERENCES_FILE__
theme_file = __THEME_FILE__
keymap_file = __KEYMAP_FILE__
keymap_name = __KEYMAP_NAME__
report = {"preferences": {"set": 0, "skipped": []}, "theme": None, "keymap": None, "warnings": []}

# Preferences: a value the running series does not know, or refuses, is skipped on its own.
def _apply(struct, values, trail):
    for pid, value in values.items():
        where = trail + "." + pid
        prop = struct.bl_rna.properties.get(pid)
        if prop is None:
            report["preferences"]["skipped"].append(where)
            continue
        try:
            if isinstance(value, dict):
                if prop.type == "POINTER" and getattr(struct, pid) is not None:
                    _apply(getattr(struct, pid), value, where)
                else:
                    report["preferences"]["skipped"].append(where)
                continue
            if prop.is_readonly or prop.type in ("POINTER", "COLLECTION"):
                report["preferences"]["skipped"].append(where)
                continue
            if prop.type == "STRING" and prop.subtype in ("FILE_PATH", "DIR_PATH", "FILE_NAME"):
                report["preferences"]["skipped"].append(where)
                continue
            if prop.type == "ENUM" and prop.is_enum_flag:
                value = set(value)
            if getattr(struct, pid) != value and not (isinstance(value, list) and list(getattr(struct, pid)) == value):
                setattr(struct, pid, value)
                report["preferences"]["set"] += 1
        except Exception as e:
            report["preferences"]["skipped"].append("%s (%s)" % (where, e))

prefs = bpy.context.preferences
if preferences_file:
    with open(preferences_file, "r", encoding="utf-8") as f:
        groups = json.load(f)
    for group, values in groups.items():
        struct = getattr(prefs, group, None)
        if struct is not None and isinstance(values, dict) and group in __PREFERENCE_GROUPS__:
            _apply(struct, values, group)
        else:
            report["preferences"]["skipped"].append(group)

# Theme: Blender's own installer, which only lets a theme file touch theme types.
if theme_file:
    try:
        result = bpy.ops.preferences.theme_install(filepath=theme_file, overwrite=True)
        report["theme"] = "FINISHED" in result
        if not report["theme"]:
            report["warnings"].append("theme: %s" % sorted(result))
    except Exception as e:
        report["warnings"].append("theme: %s" % e)

# Keymap: a keyconfig file is Python, and this one comes from outside. It is never executed:
# its two literal assignments are read as data, handed to Blender, and the preset Blender
# runs on later starts is one Blender wrote itself from that data.
if keymap_file:
    try:
        import ast
        from bl_keymap_utils.io import keyconfig_import_from_data, keyconfig_export_as_data
        with open(keymap_file, "r", encoding="utf-8") as f:
            tree = ast.parse(f.read())
        found = {}
        for node in tree.body:
            if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
                if node.targets[0].id in ("keyconfig_version", "keyconfig_data"):
                    found[node.targets[0].id] = ast.literal_eval(node.value)
        data = found.get("keyconfig_data")
        if not isinstance(data, list):
            raise ValueError("no keyconfig data")
        version = found.get("keyconfig_version")
        wm = bpy.context.window_manager
        bpy.utils.keyconfig_init()
        kc = keyconfig_import_from_data(keymap_name, data, keyconfig_version=tuple(version) if version else bpy.app.version)
        wm.keyconfigs.active = kc
        wm.keyconfigs.update()
        target = bpy.utils.user_resource("SCRIPTS", path=os.path.join("presets", "keyconfig"), create=True)
        keyconfig_export_as_data(wm, kc, os.path.join(target, keymap_name + ".py"), all_keymaps=False)
        report["keymap"] = [km.name for km in kc.keymaps]
    except Exception as e:
        report["warnings"].append("keymap: %s" % e)

bpy.ops.wm.save_userpref()
_out = getattr(sys, "__stdout__", None) or sys.stdout
_out.write("__MARKER__" + json.dumps(report) + "\n")
_out.flush()
"#;

/// Adds the setup's remote repositories, installs addons from files, and sets enabled states.
/// Extensions of remote repositories are not touched here; the extension command line
/// installs those in a separate run. The three inputs are JSON text.
pub const RESTORE_ADDONS_PY: &str = r#"
import bpy, addon_utils, json, os, sys, zipfile

# The three inputs arrive as JSON text, so true/false/null need no translation.
repositories = json.loads(__REPOSITORIES__)
files = json.loads(__FILES__)
states = json.loads(__STATES__)
report = {"repositories_added": [], "installed": [], "failed": [], "enabled": [], "disabled": [], "missing": [], "warnings": []}

prefs = bpy.context.preferences
extensions = getattr(prefs, "extensions", None)

# Remote repositories are matched by URL; a module name that is taken gets a suffix.
if extensions is not None:
    known_urls = set()
    modules = set()
    for repo in extensions.repos:
        modules.add(repo.module)
        if getattr(repo, "use_remote_url", False):
            known_urls.add((repo.remote_url or "").rstrip("/"))
    for wanted in repositories:
        url = (wanted.get("url") or "").rstrip("/")
        if not url or url in known_urls:
            continue
        module = wanted.get("module") or "repository"
        base, n = module, 2
        while module in modules:
            module = "%s_%d" % (base, n)
            n += 1
        try:
            extensions.repos.new(name=wanted.get("name") or module, module=module, remote_url=wanted["url"])
            modules.add(module)
            known_urls.add(url)
            report["repositories_added"].append(module)
        except Exception as e:
            report["warnings"].append("repository %s: %s" % (wanted.get("name"), e))
    if "user_default" not in modules:
        try:
            extensions.repos.new(name="User Default", module="user_default")
        except Exception as e:
            report["warnings"].append("user_default repository: %s" % e)
    # The module that serves each remote URL here, for the command line that installs by id.
    report["repositories"] = {}
    for repo in extensions.repos:
        if getattr(repo, "use_remote_url", False) and repo.remote_url:
            report["repositories"][repo.remote_url.rstrip("/")] = repo.module
elif repositories:
    report["warnings"].append("this Blender has no extension repositories")

def _is_extension_archive(path):
    if not path.lower().endswith(".zip"):
        return False
    try:
        with zipfile.ZipFile(path) as z:
            return any(n.split("/")[-1] == "blender_manifest.toml" for n in z.namelist())
    except Exception:
        return False

for entry in files:
    path = entry["path"]
    try:
        if _is_extension_archive(path):
            if bpy.app.version < (4, 2, 0):
                raise RuntimeError("extensions need Blender 4.2 or newer")
            result = bpy.ops.extensions.package_install_files(filepath=path, repo="user_default", enable_on_install=False)
        else:
            result = bpy.ops.preferences.addon_install(filepath=path, overwrite=True)
        if "FINISHED" not in result:
            raise RuntimeError("install did not finish: %s" % sorted(result))
        report["installed"].append(entry["module"])
    except Exception as e:
        report["failed"].append({"module": entry["module"], "error": str(e)})

# Extensions exist from 4.2 on; `bpy.ops.extensions` resolves in older versions too, so the
# version decides, not the attribute.
if bpy.app.version >= (4, 2, 0):
    try:
        bpy.ops.extensions.repo_refresh_all()
    except Exception as e:
        report["warnings"].append("repository refresh: %s" % e)
addon_utils.modules(refresh=True)
known = set(m.__name__ for m in addon_utils.modules(refresh=False))
enabled_now = set(a.module for a in prefs.addons)
for module, enabled in states.items():
    if module not in known:
        report["missing"].append(module)
        continue
    if enabled == (module in enabled_now):
        continue
    try:
        result = bpy.ops.preferences.addon_enable(module=module) if enabled else bpy.ops.preferences.addon_disable(module=module)
        if "FINISHED" not in result:
            raise RuntimeError(sorted(result))
        report["enabled" if enabled else "disabled"].append(module)
    except Exception as e:
        report["warnings"].append("%s: %s" % (module, e))

bpy.ops.wm.save_userpref()
_out = getattr(sys, "__stdout__", None) or sys.stdout
_out.write("__MARKER__" + json.dumps(report) + "\n")
_out.flush()
"#;
