pub const POWERSHELL: &str = "powershell";
pub const CREATE_NO_WINDOW_FLAG: u32 = 0x08000000;
pub const WINDOW_STYLE: &str = "-WindowStyle";
pub const HIDDEN: &str = "Hidden";
pub const COMMAND: &str = "-Command";
/// Retrieves the registry key for a specific Blender executable file.
/// Need to add the `$blender_version` value through Rust code.
///
/// $blender_version = "4.2.0"
/// $blenderRegKey = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*' |
/// Where-Object {
///     $_.DisplayName -eq 'Blender' -and
///     $_.Publisher -eq 'Blender Foundation' -and
///     $_.DisplayVersion -eq $blenderVersion
/// } |
/// Select-Object -ExpandProperty InstallLocation
///
/// if ($blenderRegKey) {
///     Write-Output $blenderRegKey
/// } else {
///     Write-Output "0"
/// }   
///
#[cfg(target_os = "windows")]
pub const RETRIEVE_BLENDER_REG_KEY_POWERSHELL_EXPRESSION: &str = r#"

$blenderRegKey = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*' |
Where-Object { 
    $_.DisplayName -eq 'Blender' -and 
    $_.Publisher -eq 'Blender Foundation' -and 
    $_.DisplayVersion -eq $blenderVersion 
} |
Select-Object -ExpandProperty InstallLocation

if ($blenderRegKey) {
    Write-Output $blenderRegKey
} else {
    Write-Output "0"
}   
"#;

/// Runs an instance for a specific Blender versions executables uninstaller (at least .msi installers).
/// Need to add the `$blender_version` value through Rust code.
///
/// $blender_version = "4.2.0"
/// $uninstallString = (Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*" |
///     Where-Object {
///         $_.DisplayName -eq 'Blender' -and
///         $_.Publisher -eq 'Blender Foundation' -and
///         $_.DisplayVersion -eq $blenderVersion
///     }).UninstallString
/// if ($uninstallString) {
///     # Start-Process -FilePath "cmd.exe" -ArgumentList "/c $uninstallString" -Wait -Verb RunAs
///     Start-Process -FilePath "cmd.exe" -ArgumentList "/c $uninstallString" -Wait -Verb RunAs -WindowStyle Hidden
/// } else {
///     Write-Output "-"
/// }
///
#[cfg(target_os = "windows")]
pub const UNINSTALL_INSTALLED_BLENDER_POWERSHELL_EXPRESSION: &str = r#"

$uninstallString = (Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*" |
    Where-Object { 
        $_.DisplayName -eq 'Blender' -and 
        $_.Publisher -eq 'Blender Foundation' -and 
        $_.DisplayVersion -eq $blenderVersion 
    }).UninstallString
if ($uninstallString) {
    # Start-Process -FilePath "cmd.exe" -ArgumentList "/c $uninstallString" -Wait -Verb RunAs
    Start-Process -FilePath "cmd.exe" -ArgumentList "/c $uninstallString" -Wait -Verb RunAs -WindowStyle Hidden
} else {
    Write-Output "-"
}
"#;

/// Symlinks 2 OS directories, if Blenderbase (and the subsequent Powershell instance)
/// was opened `as admin`. Otherwise, the symlink wont be created.
#[cfg(target_os = "windows")]
pub const SYMLINK_DIRECTORIES_PS1: &str = r#"
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")
if ($isAdmin) {
    New-Item -ItemType SymbolicLink -Path $dst -Value $src
    Write-Output "1"
} else {
    Write-Output "0"
}
"#;

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

/// Checks the permissions on a directory path, and returns the first negative one it encounters.
#[cfg(target_os = "windows")]
pub const CHECK_DIRECTORY_PATH_PERMISSIONS_POWERSHELL_EXPRESSIONS: &str = r#"
# Check if directory exists
if (Test-Path $directoryPath -PathType Container) {
    # Check if directory is read-only
    $isReadOnly = (Get-Item $directoryPath).Attributes -band [System.IO.FileAttributes]::ReadOnly
    if ($isReadOnly) {
        Write-Host 'ronly'#'Directory is read-only'
        return
    }
    # Check if directory is writable
    try {
        $testFile = Join-Path -Path $directoryPath -ChildPath 'test.txt'
        New-Item -Path $testFile -ItemType File -ErrorAction Stop | Out-Null
        Remove-Item -Path $testFile -ErrorAction SilentlyContinue
        Write-Host 'ok'#'Directory is writable'
        return
    } catch {
        Write-Host 'nwrite'#'Directory is not writable'
        return
    }
} else {
    Write-Host 'nexists'#'Directory does not exist: $directoryPath'
    return
}
"#;
