use crate::core::PRINT_APP_NAME_AND_CREATOR_SIGNATURE;

pub fn default_python_expression() -> Result<String, String> {
    let exp = format!(
        r#"
{}   
    "#,
        PRINT_APP_NAME_AND_CREATOR_SIGNATURE
    );
    return Ok(exp);
}
