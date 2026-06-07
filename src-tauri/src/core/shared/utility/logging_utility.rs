pub async fn format_command_error(function_name: &str, delimieter: &str, e: String) -> String {
    return format!("{}{}{}", function_name, delimieter, e);
}
