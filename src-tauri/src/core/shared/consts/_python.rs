/// Print ASCII art version of app name and creator signature.
/// Use example
/// ``` rs
/// py_exp = format!(r#"
/// {}
/// "#,
/// PRINT_APP_NAME_AND_CREATOR_SIGNATURE.replace("{}", chrono::Utc::now().year())
/// );
/// ```
/// Result example
/// ``` py
/// # Python code
/// import bpy
/// print(
///     r"""
///     Launched through...    
///     """,
///     "\033[94m"+
///     r"""
///         ____  __               __          __                  
///        / __ )/ /__  ____  ____/ /__  _____/ /_  ____ _________ 
///       / __  / / _ \/ __ \/ __  / _ \/ ___/ __ \/ __ `/ ___/ _ \
///      / /_/ / /  __/ / / / /_/ /  __/ /  / /_/ / /_/ (__  )  __/
///     /_____/_/\___/_/ /_/\__,_/\___/_/  /_.___/\__,_/____/\___/ 
///     """ 
///     + "\033[0m", 
///     "\033[90m"+
///     r"""                                                        
///     Physical Addons XXXX
///     """
///     + "\033[0m"  # Reset color to default (white)
/// )
/// ```
pub const PRINT_APP_NAME_AND_CREATOR_SIGNATURE: &str = r#"
print(
    r"""
    Launched through...    
    """,
    "\033[94m"+
    r"""
        ____  __               __          __                  
       / __ )/ /__  ____  ____/ /__  _____/ /_  ____ _________ 
      / __  / / _ \/ __ \/ __  / _ \/ ___/ __ \/ __ `/ ___/ _ \
     / /_/ / /  __/ / / / /_/ /  __/ /  / /_/ / /_/ (__  )  __/
    /_____/_/\___/_/ /_/\__,_/\___/_/  /_.___/\__,_/____/\___/ 
    """ 
    + "\033[0m", 
    "\033[90m"+
    r"""                                                        
    Physical Addons {}
    """
    + "\033[0m"  # Reset color to default (white)
)
"#;

/// Imports Python module.
/// 
/// Use example
/// ``` rs
/// py_exp = format!(r#"
/// {}
/// "#,
/// MODULE_IMPORT.replace("{}", "bpy")
/// );
/// ```
/// Result example
/// ``` py
/// # Python code
/// import bpy
/// ```
pub const IMPORT_MODULE: &str = 
r#"
import {}
"#;

/// Imports part of a Python module.
/// 
/// Use example
/// ```rs
/// py_exp = format!(r#"
/// {}
/// "#,
/// FROM_MODULE_IMPORT_PART.replace("{1}", "pathlib").replace("{2}", "Path")
/// );
/// ```
/// Result example
/// ```py
/// # Python code
/// from pathlib import Path
/// ```
pub const FROM_MODULE_IMPORT_PART: &str = 
r#"
from {1} import {2}
"#;

/// Print value.
/// 
/// Use example
/// ```rs
/// py_exp = format!(r#"
/// {}
/// "#,
/// PRINT.replace("{}", "example")
/// );
/// ```
/// Result example
/// ```py
/// # Python code
/// print("example")
/// ```
pub const PRINT: &str =
r#"
print({})
"#;




/// Enables a specific addon, provided its functional name (i.e. `physical-open-water` as 
/// opposed to `Physical Open Water`).
pub const ENABLE_ADDON_FN: &str = r#"
def enable_addon(addon_name):
    bpy.ops.preferences.addon_enable(module=addon_name) 
"#;

/// Disables a specific addon, provided its functional name (i.e. `physical-open-water` as 
/// opposed to `Physical Open Water`).
pub const DISABLE_ADDON_FN: &str = r#"
def disable_addon(addon_name):
    bpy.ops.preferences.addon_disable(module=addon_name)
"#;


pub const CONSTANTS: &str = r#"
series_cutoff = 3.1 # This is the cutoff for the current Blenderbase system, for which Blender versions it supports.
default_variant_type = "Unidentified"

"#; 


/// Retrieve the Blender versions identifiers, which include both the Blender metadata,
/// as well as certain external data and certain descriptive values. 
#[cfg(target_os = "windows")]
pub const BLENDER_IDENTIFIERS: &str = r#"
download_data_path = Path(bpy.app.binary_path).parent / "download_data.json"
temp_arr = bpy.app.version_string.split(" ") # Splits the version_string between the " "

blender_version = ""
blender_series = ""
variant_type = ""
commit_link = "https://projects.blender.org/blender/blender/commit/" + bpy.app.build_hash.decode('utf-8')
commit_hash = bpy.app.build_hash.decode('utf-8')
blender_build_date = bpy.app.build_date.decode('utf-8')
community_addon_directory_path = str(Path(bpy.utils.user_resource('SCRIPTS')) / "addons")

if download_data_path.exists() == True:
    with open(download_data_path, 'r') as json_file:
        download_data = json.load(json_file)
        variant_type = download_data["variant"]
elif len(temp_arr) > 1:
    variant_type = ' '.join(temp_arr[1:])
else: # If all else fails, use the default value.
    if bpy.app.version_cycle == "":   
        variant_type = "Unidentified" # 
    else:
        variant_type = bpy.app.version_cycle[0].upper() + bpy.app.version_cycle[1:]
blender_version = str(temp_arr[0])
blender_series =  str(".".join(temp_arr[0].split(".", 2)[:2]))
"#;

/// Retrieve the Blender versions identifiers, which include both the Blender metadata,
/// as well as certain external data and certain descriptive values. 
#[cfg(target_os = "macos")]
pub const BLENDER_IDENTIFIERS: &str = r#"
download_data_path = Path(bpy.app.binary_path).parent.parent / "download_data.json"
temp_arr = bpy.app.version_string.split(" ") # Splits the version_string between the " "

blender_version = ""
blender_series = ""
variant_type = ""
commit_link = "https://projects.blender.org/blender/blender/commit/" + bpy.app.build_hash.decode('utf-8')
commit_hash = bpy.app.build_hash.decode('utf-8')
blender_build_date = bpy.app.build_date.decode('utf-8')
community_addon_directory_path = str(Path(bpy.utils.user_resource('SCRIPTS')) / "addons")

if len(temp_arr) > 1: # If the Blender executable contains any variant string, use it over anything else.
    variant_type = ' '.join(temp_arr[1:])
elif download_data_path.exists() == True:
    with open(download_data_path, 'r') as json_file:
        download_data = json.load(json_file)
        variant_type = download_data["variant"]
else: # If all else fails, use the passed variant from the Rust side (generic).
    if bpy.app.version_cycle == "":   
        variant_type = "Unidentified" # 
    else:
        variant_type = bpy.app.version_cycle[0].upper() + bpy.app.version_cycle[1:]
blender_version = str(temp_arr[0])
blender_series =  str(".".join(temp_arr[0].split(".", 2)[:2]))


"#;

/// Checks that the version for the Blender is one, that is currently 
pub const VALIDITY_CHECK: &str = r#"
if float(blender_series) < series_cutoff:
    bpy.ops.wm.quit_blender()
"#;


/// Saves the Blender versions metadata (identifiers) to the Blender versions specific database
/// document.
#[cfg(target_os = "windows")]
pub const BLENDER_DATA: &str = r#"
blender_data = {
    "id": id,# blender_version_key, # The file parent directory name + settings directory identifier.
    "active": False,
    "blender_launcher_executable_file_path": blender_launcher_executable_file_path,# str(Path(bpy.app.binary_path).parent / "blender-launcher.exe"),
    "community_addon_directory_path": community_addon_directory_path,
    "metadata": {}, # The Blender versions metadata.
    "download_data": {}, # The downloadable Blender version data, only for the downloaded Blender versions.
}
blender_data["metadata"] = {
    "version": blender_version,
    "series": blender_series,
    "variant_type": variant_type,
    "commit_link": commit_link,
    "commit_hash": commit_hash,
    "build_date": blender_build_date,
}
if download_data_path.exists() == True:
    with open(download_data_path, 'r') as json_file:
        download_data = json.load(json_file)
        blender_data["download_data"] = download_data
print("$$$$$$$$$ BLENDER METADATA $$$$$$$$$")
print(json.dumps(blender_data, indent=4))


with open(blender_data_path , 'w') as json_file:
    json.dump(blender_data, json_file)
"#;

/// Saves the Blender versions metadata (identifiers) to the Blender versions specific database
/// document.
#[cfg(target_os = "macos")]
pub const BLENDER_DATA: &str = r#"
blender_data = {
    "id": id,# blender_version_key, # The file parent directory name + settings directory identifier.
    "active": false,
    "blender_launcher_executable_file_path": blender_launcher_executable_file_path,# str(Path(bpy.app.binary_path).parent / "blender-launcher.exe"),
    "community_addon_directory_path": community_addon_directory_path,
    "metadata": {}, # The Blender versions metadata.
    "download_data": {}, # The downloadable Blender version data, only for the downloaded Blender versions.
}
blender_data["metadata"] = {
    "version": blender_version,
    "series": blender_series,
    "variant_type": variant_type,
    "commit_link": commit_link,
    "commit_hash": commit_hash,
    "build_date": blender_build_date,
}
if download_data_path.exists() == True:
    with open(download_data_path, 'r') as json_file:
        download_data = json.load(json_file)
        blender_data["download_data"] = download_data

print(json.dumps(blender_data, indent=4))


with open(blender_data_path , 'w') as json_file:
    json.dump(blender_data, json_file)
"#;

/// Saves the Blender versions addon data to the addon database documents.
#[cfg(target_os = "windows")]
pub const ADDON_DATA: &str = r#"
print("$$$$$$$$$ ADDON METADATA $$$$$$$$$")
system_username = Path.home().parts[-1]
all_addon_modules = addon_utils.modules()
enabled_addon_modules = [addon.module for addon in bpy.context.preferences.addons]

addon_data_path_obj = Path(addon_data_path)
addon_data_path_obj.mkdir(parents=True, exist_ok=True)
for addon in all_addon_modules:
    addon_data = {}
    if hasattr(addon, "bl_info"):
        addon_file_path = addon.__file__
        addon_version = '.'.join(map(str, addon.bl_info.get("version", [0, 0, 0])))
        addon_blender_version = '.'.join(map(str, addon.bl_info.get("blender", [0, 0, 0])))
        
        enabled_value = (addon.__name__ in enabled_addon_modules)
        pattern = fr"\\{blender_series}\\scripts\\addons(?!_contrib)"
        
        if re.search(pattern, addon_file_path):
            type = "+o"
            addon_variant_type = "official"
            if addon_file_path.startswith(f"C:\\Users\\{system_username}\\AppData"):
                type = "+c"
                addon_variant_type = "community"
            addon_file_path = Path(addon_data_path) / ((str(addon.__name__))+type+".json")
            addon_data = {
                "id": (str(addon.__name__))+type,
                "functional_name": (str(addon.__name__)),
                "addon_file_path": addon.__file__,
                "variant_type": addon_variant_type,
                "enabled": enabled_value,  
                "bl_info": {
                    "name": addon.bl_info.get("name"),
                    "author": addon.bl_info.get("author"),
                    "version": addon_version,
                    "blender": addon_blender_version,
                    "location": addon.bl_info.get("location"),
                    "description": addon.bl_info.get("description"),
                    "warning": addon.bl_info.get("warning"),
                    "doc_url": "To ba added.", 
                    "tracker_url": addon.bl_info.get("tracker_url"),
                    "support": addon.bl_info.get("support"),
                    "category": addon.bl_info.get("category")
                }
            }
            print(json.dumps(addon_data, indent=4))

            with open(addon_file_path, 'w') as json_file:
                json.dump(addon_data, json_file)
"#;

/// Saves the Blender versions addon data to the addon database documents.
#[cfg(target_os = "macos")]
pub const ADDON_DATA: &str = r#"
system_username = Path.home().parts[-1]
all_addon_modules = addon_utils.modules()
enabled_addon_modules = [addon.module for addon in bpy.context.preferences.addons]

addon_data_path_obj = Path(addon_data_path)
addon_data_path_obj.mkdir(parents=True, exist_ok=True)
for addon in all_addon_modules:
    addon_data = {}
    if hasattr(addon, "bl_info"):
        addon_file_path = addon.__file__
        addon_version = '.'.join(map(str, addon.bl_info.get("version", [0, 0, 0])))
        addon_blender_version = '.'.join(map(str, addon.bl_info.get("blender", [0, 0, 0])))
        
        enabled_value = (addon.__name__ in enabled_addon_modules)
        pattern = fr"/{blender_series}/scripts/addons(?!_contrib)"
        if re.search(pattern, addon_file_path):
            type = "+o"
            addon_variant_type = "official"
            if addon_file_path.startswith(f"/Users/{system_username}/Library/Application Support/"):
                type = "+c"
                addon_variant_type = "community"
            addon_file_path = Path(addon_data_path) / ((str(addon.__name__))+type+".json")
            addon_data = {
                "id": (str(addon.__name__))+type,
                "functional_name": (str(addon.__name__)),
                "addon_file_path": addon.__file__,
                "variant_type": addon_variant_type,
                "enabled": enabled_value,  
                "bl_info": {
                    "name": addon.bl_info.get("name"),
                    "author": addon.bl_info.get("author"),
                    "version": addon_version,
                    "blender": addon_blender_version,
                    "location": addon.bl_info.get("location"),
                    "description": addon.bl_info.get("description"),
                    "warning": addon.bl_info.get("warning"),
                    "doc_url": "To ba added.", 
                    "tracker_url": addon.bl_info.get("tracker_url"),
                    "support": addon.bl_info.get("support"),
                    "category": addon.bl_info.get("category")
                }
            }
            print(json.dumps(addon_data, indent=4))

            with open(addon_file_path, 'w') as json_file:
                json.dump(addon_data, json_file)
"#;



/// Processes the database addon data, enabling and disabling addons based on their 
/// `addon.enabled` keys value.
pub const PROCESS_ADDONS: &str = r#"
addon_data_path_obj = Path(addon_data_path)

addon_arr = []
for file_path in addon_data_path_obj.iterdir():
    if file_path.suffix == '.json':
        
        # Open the file and load its contents as JSON
        with open(file_path, 'r') as file:
            try:
                data = json.load(file)
                addon_arr.append(data)
            except json.JSONDecodeError as e:
                print(f"Error decoding JSON in file {file_path.name}: {e}")

for i in addon_arr:
    if i.get("enabled") == True:
            enable_addon(i.get("functional_name"))
    else:
        disable_addon(i.get("functional_name"))
"#;

/// Checks if an addon exists in the Blender user preferences instance.
pub const CHECK_ADDON_EXISTS: &str = r#"
def addon_exists_in_preferences(addon_name):
    # Check if the addon exists in Blender's addons
    return addon_name in bpy.context.preferences.addons
"#;


/// Disables an addon if it exists in the current user preferences instance.
pub const DISABLE_ADDON_AFTER_DELETION: &str = r#"
# Check if the addon exists before attempting to disable it
if addon_exists_in_preferences(addon_name):
    bpy.ops.preferences.addon_disable(module=addon_name)

    # Save user preferences to make the change persistent
    bpy.ops.wm.save_userpref()
    
"#;

/// Opens a selected addons N panel inside of a new Blender instance.
pub const OPEN_N_PANEL: &str = r#"
import bpy

addon_literal_name = "{}" # i.e. "Hello World Addon"
addon_functional_name = "{}" # i.e. "test_addon"

bpy.ops.screen.userpref_show()
bpy.context.preferences.active_section = 'ADDONS'
bpy.data.window_managers["WinMan"].addon_search = addon_literal_name # This step might be obsolete.
bpy.ops.preferences.addon_expand(module=addon_functional_name)
"#;


pub const EXIT_BLENDER: &str = r#"
exit()
"#;
