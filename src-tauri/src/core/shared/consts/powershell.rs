pub const CREATE_NO_WINDOW_FLAG: u32 = 0x08000000;
pub const WINDOW_STYLE: &str = "-WindowStyle";
pub const HIDDEN: &str = "Hidden";
pub const COMMAND: &str = "-Command";

pub const GET_PATH_PERMISSIONS_PS1_EXPRESSION: &str = r#"
# Get the ACL for the specified path
$acl = Get-Acl -Path $Path
# Initialize the result object
$result = @{
    full_control = $false
    modify = $false
    read_and_execute = $false
    list_folder_contents = $false
    read = $false
    write = $false
    special_permissions = $false
}
# Iterate through each access rule
foreach ($rule in $acl.Access) {
    # Only consider FileSystemRights
    if ($rule.FileSystemRights) {
        $rights = $rule.FileSystemRights
        if ($rights -band [System.Security.AccessControl.FileSystemRights]::FullControl) {
            $result.full_control = $true
        }
        if ($rights -band [System.Security.AccessControl.FileSystemRights]::Modify) {
            $result.modify = $true
        }
        if ($rights -band [System.Security.AccessControl.FileSystemRights]::ReadAndExecute) {
            $result.read_and_execute = $true
        }
        if ($rights -band [System.Security.AccessControl.FileSystemRights]::ListDirectory) {
            $result.list_folder_contents = $true
        }
        if ($rights -band [System.Security.AccessControl.FileSystemRights]::ReadData) {
            $result.read = $true
        }
        if ($rights -band [System.Security.AccessControl.FileSystemRights]::WriteData) {
            $result.write = $true
        }
        if ($rights -band [System.Security.AccessControl.FileSystemRights]::ReadPermissions) {
            $result.special_permissions = $true
        }
    }
}
# Convert to JSON for easy serialization
$json = $result | ConvertTo-Json -Compress
Write-Output $json
"#;
