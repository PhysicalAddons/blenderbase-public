/// Call function by name and passing parameters.
///
pub const FN_CALL_W_PARAMS: &str = r#"
{}({})
"#;

/// Enables an addon with its `functional name` - i.e. `physical-open-water`, as opposed to `Physical Open Water`.
///
/// Use example
/// ``` rs
/// py_exp = format!(r#"
/// {}
/// "#,
/// FN_ENABLE_A.replace("{}", "addon_name")
/// );
/// ```
/// Result example
/// ``` py
/// # Python code
/// def enable_a(addon_name):
///     bpy.ops.preferences.addon_enable(module=addon_name)
/// # Define used variable with the same name as the one in the function.
/// addon_name = "physical-open-water"
/// enable_a(addon_name)
/// ```
pub const FN_DEF_ENABLE_A: &str = r#"
def enable_a({}):
    bpy.ops.preferences.addon_enable(module={}) 
"#;

/// Disables a specific addon, provided its functional name (i.e. `physical-open-water` as
/// opposed to `Physical Open Water`).
pub const DISABLE_ADDON_FN: &str = r#"
###
def disable_a({}):
    bpy.ops.preferences.addon_disable(module={})
"#;
