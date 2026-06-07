/// Print ASCII art version of app name and creator signature.
/// Use example
/// ``` rs
/// py_exp = format!(r#"
/// {}
/// "#,
/// PRINT_APP_NAME_AND_CREATOR_SIGNATURE.replace("{}", chrono::Utc::now().year())
/// );
/// ```
/// Result example:
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
/// import bpy
/// import addon_utils
/// import json
/// import re
/// import platform
/// from pathlib import Path
///
/// Use example
/// ``` rs
/// py_exp = format!(r#"
/// {}
/// "#,
/// MODULE_IMPORT.replace("{}", "bpy")
/// );
/// ```
/// Result example:
/// ``` py
/// # Python code
/// import bpy
/// ```
pub const IMPORT_MODULE: &str = r#"
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
/// Result example:
/// ```py
/// # Python code
/// from pathlib import Path
/// ```
pub const FROM_MODULE_IMPORT_PART: &str = r#"
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
/// Result example:
/// ```py
/// # Python code
/// print("example")
/// ```
pub const PRINT: &str = r#"
print({})
"#;
