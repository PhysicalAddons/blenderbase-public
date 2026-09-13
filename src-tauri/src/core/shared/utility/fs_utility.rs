use std::io::{Read, Write};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

use sha2::{Digest, Sha256};

use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

use crate::core::{PermissionDetails, NO_SENTANCE_CASE, YES_SENTANCE_CASE};
#[cfg(windows)]
use crate::core::{
    COMMAND, CREATE_NO_WINDOW_FLAG, GET_PATH_PERMISSIONS_PS1_EXPRESSION, HIDDEN, WINDOW_STYLE,
};

const WIDTH: f64 = 600.0;
const HEIGHT: f64 = 400.0;
const RESIZEABLE: bool = true;
const ALWAYS_ON_TOP: bool = true;
const FOCUSED: bool = true;
const SKIP_TASKBAR: bool = true;

pub async fn instance_popup_window(
    app: AppHandle,
    label: String,
    title: String,
    url_path: String,
    width: Option<f64>,
    height: Option<f64>,
    resizeable: Option<bool>,
    always_on_top: Option<bool>,
    focused: Option<bool>,
    skip_taskbar: Option<bool>,
) -> Result<(), String> {
    match WebviewWindowBuilder::new(&app, label, WebviewUrl::App(url_path.into()))
        .title(title)
        .inner_size(
            match width {
                Some(v) => v,
                None => WIDTH,
            },
            match height {
                Some(v) => v,
                None => HEIGHT,
            },
        )
        .resizable(match resizeable {
            Some(v) => v,
            None => RESIZEABLE,
        })
        .always_on_top(match always_on_top {
            Some(v) => v,
            None => ALWAYS_ON_TOP,
        })
        .focused(match focused {
            Some(v) => v,
            None => FOCUSED,
        })
        .skip_taskbar(match skip_taskbar {
            Some(v) => v,
            None => SKIP_TASKBAR,
        })
        .build()
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format!("Failed instance popup window: {:?}", e)),
    }
}

pub async fn get_directory_from_file_explorer(
    app: AppHandle,
) -> Result<Option<std::path::PathBuf>, String> {
    // TODO set to open Desktop always.
    // The native picker blocks its thread until the user answers, so it runs
    // on the blocking pool instead of stalling the async runtime.
    let directory_path_option =
        match tokio::task::spawn_blocking(move || app.dialog().file().blocking_pick_folder()).await
        {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed get directory from file explorer: {:?}", e)),
        };
    let directory_path_string = match directory_path_option {
        Some(v) => v.to_string(),
        None => return Err(format!("Failed get directory from file explorer")),
    };
    let directory_path = std::path::PathBuf::from(directory_path_string);
    return Ok(Some(directory_path));
}

/// Gets the file size of a file, and then properly formats it,
/// returning the formatted string.
///
/// This is used in the .blend file metadata struct, because the user should
/// be able to see the size of their .blend files.
pub fn format_file_size(file_size_bytes: f64) -> Result<String, String> {
    if file_size_bytes < 1024.0 {
        Ok(format!("{:.2} B", file_size_bytes))
    } else if file_size_bytes < 1_048_576.0 {
        Ok(format!("{:.2} KB", file_size_bytes / 1024.0))
    } else if file_size_bytes < 1_073_741_824.0 {
        Ok(format!("{:.2} MB", file_size_bytes / 1_048_576.0))
    } else {
        Ok(format!("{:.2} GB", file_size_bytes / 1_073_741_824.0))
    }
}

pub fn open_in_file_explorer(file_path: std::path::PathBuf) -> Result<(), String> {
    let parent_directory = match file_path.parent() {
        Some(v) => v,
        None => return Err(format!("Failed open in file explorer")),
    };
    // Windows and macOS can open the parent folder with the item itself
    // selected, which is what "open file location" means to users. Linux
    // file managers have no portable equivalent, so the parent is opened.
    #[cfg(target_os = "windows")]
    {
        let _ = parent_directory;
        let system_root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
        let explorer = std::path::PathBuf::from(system_root).join("explorer.exe");
        // Explorer only accepts `/select,"<path>"` with the quotes around the
        // path alone. Rust's normal argument quoting would wrap the whole
        // token once the path contains a space, and Explorer then ignores it
        // and opens the Documents folder instead. `raw_arg` bypasses that.
        let mut command = std::process::Command::new(explorer);
        command.raw_arg(format!("/select,\"{}\"", file_path.display()));
        match command.spawn() {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("Failed open in file explorer: {:?}", e)),
        }
    }
    #[cfg(target_os = "macos")]
    {
        let _ = parent_directory;
        match std::process::Command::new("open")
            .arg("-R")
            .arg(&file_path)
            .spawn()
        {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("{:?}", e)),
        }
    }
    #[cfg(target_os = "linux")]
    match std::process::Command::new("xdg-open")
        .arg(parent_directory)
        .spawn()
    {
        Ok(_) => Ok(()), // C (3.c.) Ok()
        Err(e) => return Err(format!("{:?}", e)),
    }
}

/// Absolute path to Windows PowerShell, so a `powershell.exe` planted in the
/// working directory or on PATH can never be picked up instead.
#[cfg(windows)]
pub fn powershell_executable() -> std::path::PathBuf {
    let system_root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    std::path::PathBuf::from(system_root).join("System32\\WindowsPowerShell\\v1.0\\powershell.exe")
}

#[cfg(windows)]
pub fn run_ps1_expression_as_string(args: Vec<&str>) -> Result<String, String> {
    run_ps1_expression_with_env(args, &[])
}

#[cfg(not(windows))]
pub fn run_ps1_expression_as_string(_args: Vec<&str>) -> Result<String, String> {
    Err(String::from("PowerShell is only available on Windows"))
}

/// Runs PowerShell with extra environment variables. Any value that comes
/// from a user or the webview must travel this way and be read as `$env:NAME`
/// inside the script. Splicing it into the script text would let a quote in
/// the value end the string literal and run the rest as a command.
#[cfg(windows)]
pub fn run_ps1_expression_with_env(
    args: Vec<&str>,
    envs: &[(&str, &str)],
) -> Result<String, String> {
    let out = std::process::Command::new(powershell_executable())
        .args(args)
        .envs(envs.iter().copied())
        .stdout(std::process::Stdio::piped()) // Pipe stdout to Rust process.
        .stderr(std::process::Stdio::piped()) // Pipe stderr to Rust process.
        .creation_flags(CREATE_NO_WINDOW_FLAG) // CREATE_NO_WINDOW flag - powershell window is hidden.
        .output()
        .map_err(|err| format!("{}", err))?;
    if !out.status.success() {
        return Err(format!(
            "Failed run ps1 expression as string: {:?}",
            out.status.code()
        ));
    }
    // Outputs string that can be serialized in after function call.
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[cfg(not(windows))]
pub fn run_ps1_expression_with_env(
    _args: Vec<&str>,
    _envs: &[(&str, &str)],
) -> Result<String, String> {
    Err(String::from("PowerShell is only available on Windows"))
}

/// Without ACL probing on other platforms the location is assumed usable;
/// any real permission problem surfaces on the first write.
#[cfg(not(windows))]
pub fn get_permission_details(_path: &str) -> Result<PermissionDetails, String> {
    Ok(PermissionDetails {
        read: true,
        write: true,
        ..Default::default()
    })
}

#[cfg(windows)]
pub fn get_permission_details(path: &str) -> Result<PermissionDetails, String> {
    // The path is handed over as an environment variable and never written
    // into the script: a directory name containing `'` or `;` would otherwise
    // close the string literal and execute whatever follows as PowerShell.
    let expr = format!(
        r#"
$Path = $env:BLENDERBASE_PATH
{}
    "#,
        GET_PATH_PERMISSIONS_PS1_EXPRESSION
    );
    let args = vec![WINDOW_STYLE, HIDDEN, COMMAND, expr.as_str()];
    let out = match run_ps1_expression_with_env(args, &[("BLENDERBASE_PATH", path)]) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed get permission detail: {:?}", e)),
    };
    let result: PermissionDetails = match serde_json::from_str(&out) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed get permission detail: {:?}", e)),
    };
    return Ok(result);
}

/// Whether `executable` is named like a Blender executable on this platform:
/// `blender-launcher.exe` / `blender.exe` on Windows, `Blender` inside a
/// `Blender.app/Contents/MacOS` bundle on macOS, `blender` on Linux.
fn is_blender_executable_name(executable: &std::path::Path) -> bool {
    let file_name: String = executable
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    #[cfg(target_os = "windows")]
    {
        let lower = file_name.to_ascii_lowercase();
        lower == "blender-launcher.exe" || lower == "blender.exe"
    }
    #[cfg(target_os = "macos")]
    {
        let in_bundle = executable
            .parent()
            .map(|p| p.ends_with("Blender.app/Contents/MacOS"))
            .unwrap_or(false);
        file_name == "Blender" && in_bundle
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        file_name == "blender"
    }
}

/// Resolves the executable of an installed Blender version and refuses anything
/// that is not a Blender executable (see `is_blender_executable_name`) located
/// inside that version's installation directory. Both values come from the
/// database, but the rows were once accepted from the webview, so they are
/// re-checked here before anything is spawned.
pub fn validate_blender_executable(
    installation_directory_path: &str,
    executable_file_path: &str,
) -> Result<std::path::PathBuf, String> {
    let executable = std::path::PathBuf::from(executable_file_path);
    if !is_blender_executable_name(&executable) {
        return Err(format!(
            "Refusing to launch {}: not a Blender executable",
            executable.display()
        ));
    }
    if installation_directory_path.trim().is_empty() {
        return Err(format!(
            "Refusing to launch {}: the version has no installation directory",
            executable.display()
        ));
    }
    let install_dir = std::path::PathBuf::from(installation_directory_path);
    let canonical_exe = executable
        .canonicalize()
        .map_err(|e| format!("Failed to resolve {}: {}", executable.display(), e))?;
    let canonical_dir = install_dir
        .canonicalize()
        .map_err(|e| format!("Failed to resolve {}: {}", install_dir.display(), e))?;
    if !canonical_exe.starts_with(&canonical_dir) {
        return Err(format!(
            "Refusing to launch {}: it is outside the installation directory {}",
            executable.display(),
            install_dir.display()
        ));
    }
    if !canonical_exe.is_file() {
        return Err(format!("Refusing to launch {}: not a file", executable.display()));
    }
    Ok(executable)
}

pub fn launch_executable(
    executable_file_path: std::path::PathBuf,
    args: Option<Vec<String>>,
) -> Result<(), String> {
    let mut command = std::process::Command::new(executable_file_path);
    let arguments = match args {
        Some(v) => v,
        None => vec![],
    };
    // `spawn` returns as soon as the process starts. `output` would block this
    // thread until Blender exits, which stalled the UI for the whole session.
    match command.args(arguments).spawn() {
        Ok(child) => {
            // Detach: Blender keeps running independently of Blenderbase.
            drop(child);
            Ok(())
        }
        Err(e) => return Err(format!("Failed launch executable: {:?}", e)),
    }
}

/// Creates a directory symbolic link at `dst` pointing to `src` without
/// requiring Blenderbase itself to run as administrator.
///
/// 1. A plain symlink is attempted first. This succeeds when the process is
///    already elevated, or when Windows Developer Mode is enabled (Windows
///    then allows unprivileged symlink creation).
/// 2. If Windows refuses for lack of privilege, only the link creation is
///    re-run through an elevated PowerShell. That triggers the standard UAC
///    consent prompt while the app keeps running. Declining the prompt, or
///    any other failure, is reported as an error.
///
/// The elevated step is Windows-only; other platforms create the link directly.
#[cfg(windows)]
pub fn symlink_directory(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    if !src.is_dir() {
        return Err(format!(
            "Failed symlink directory: source is not a directory: {}",
            src.display()
        ));
    }
    if dst.symlink_metadata().is_ok() {
        return Err(format!(
            "Failed symlink directory: destination already exists: {}",
            dst.display()
        ));
    }
    // ERROR_PRIVILEGE_NOT_HELD: the caller lacks SeCreateSymbolicLinkPrivilege.
    const ERROR_PRIVILEGE_NOT_HELD: i32 = 1314;
    match std::os::windows::fs::symlink_dir(src, dst) {
        Ok(()) => return Ok(()),
        Err(e)
            if e.raw_os_error() == Some(ERROR_PRIVILEGE_NOT_HELD)
                || e.kind() == std::io::ErrorKind::PermissionDenied => {}
        Err(e) => return Err(format!("Failed symlink directory: {:?}", e)),
    }

    // Elevated fallback. The link command is handed to the elevated shell as
    // an `-EncodedCommand` (base64 of UTF-16LE) so that no helper file exists
    // on disk for anything else to swap out before the elevated process reads
    // it, and no path has to be quoted through two shell layers. Inside the
    // script, single quotes are the only character that needs escaping.
    let ps_quote = |p: &std::path::Path| p.to_string_lossy().replace('\'', "''");
    let script = format!(
        "$ErrorActionPreference = 'Stop'\r\ntry {{\r\n    New-Item -ItemType SymbolicLink -Path '{}' -Value '{}' | Out-Null\r\n    exit 0\r\n}} catch {{\r\n    exit 1\r\n}}\r\n",
        ps_quote(dst),
        ps_quote(src)
    );
    let encoded = encode_powershell_command(&script);
    // `-Verb RunAs` is what raises the UAC prompt. `-Wait -PassThru` lets us
    // read the elevated process's exit code once the user has answered it.
    let expr = format!(
        "$p = Start-Process -FilePath '{}' -Verb RunAs -Wait -PassThru -WindowStyle Hidden -ArgumentList '-NoProfile','-NonInteractive','-ExecutionPolicy','Bypass','-EncodedCommand','{}'\r\nexit $p.ExitCode",
        ps_quote(&powershell_executable()),
        encoded
    );
    let args = vec![WINDOW_STYLE, HIDDEN, COMMAND, expr.as_str()];
    let result = run_ps1_expression_as_string(args);

    // The link's presence on disk is the authoritative outcome. A declined
    // UAC prompt makes Start-Process throw, which surfaces as a non-zero exit.
    match dst.symlink_metadata() {
        Ok(meta) if meta.file_type().is_symlink() => Ok(()),
        _ => Err(match result {
            Ok(_) => String::from(
                "Failed symlink directory: the elevated command finished but no link was created",
            ),
            Err(_) => String::from(
                "Failed symlink directory: administrator approval was declined or the elevated command failed",
            ),
        }),
    }
}

#[cfg(not(windows))]
pub fn symlink_directory(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    if !src.is_dir() {
        return Err(format!(
            "Failed symlink directory: source is not a directory: {}",
            src.display()
        ));
    }
    if dst.symlink_metadata().is_ok() {
        return Err(format!(
            "Failed symlink directory: destination already exists: {}",
            dst.display()
        ));
    }
    std::os::unix::fs::symlink(src, dst).map_err(|e| format!("Failed symlink directory: {:?}", e))
}

/// Encodes a script the way `powershell.exe -EncodedCommand` expects it:
/// base64 over the UTF-16LE bytes of the text.
#[cfg(windows)]
fn encode_powershell_command(script: &str) -> String {
    use base64::Engine;
    let mut bytes = Vec::with_capacity(script.len() * 2);
    for unit in script.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// SHA-256 of a file as lowercase hex. Hashing runs on the blocking pool so a
/// 400 MB Blender archive does not stall the async runtime.
pub async fn sha256_of_file(path: std::path::PathBuf) -> Result<String, String> {
    let join = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let mut file = std::fs::File::open(&path)
            .map_err(|e| format!("Failed sha256 of file {}: {}", path.display(), e))?;
        let mut hasher = Sha256::new();
        let mut buf = vec![0u8; 1 << 20];
        loop {
            let n = file
                .read(&mut buf)
                .map_err(|e| format!("Failed sha256 of file {}: {}", path.display(), e))?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    });
    match join.await {
        Ok(v) => v,
        Err(e) => Err(format!("Failed sha256 of file: {:?}", e)),
    }
}

/// Looks `file_name` up in a Blender `*.sha256` listing, whose lines have the
/// form `<hex digest>  <file name>`.
pub fn find_sha256_in_listing(listing: &str, file_name: &str) -> Option<String> {
    listing.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let name = parts.next()?;
        (name == file_name && is_sha256_hex(hash)).then(|| hash.to_ascii_lowercase())
    })
}

pub fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

pub async fn write_file(file_path: std::path::PathBuf, content: String) -> Result<(), String> {
    match std::fs::write(file_path, content) {
        Ok(_) => Ok(()),
        Err(e) => return Err(format!("Failed write file: {:?}", e)),
    }
}

pub async fn delete_file(file_path: std::path::PathBuf) -> Result<(), String> {
    match std::fs::remove_file(file_path) {
        Ok(_) => Ok(()),
        Err(e) => return Err(format!("Failed delete file: {:?}", e)),
    }
}

/// Removes a directory tree on the blocking pool: a Blender install is
/// several thousand files and would otherwise stall the async runtime.
pub async fn delete_directory(directory_path: std::path::PathBuf) -> Result<(), String> {
    let join = tokio::task::spawn_blocking(move || std::fs::remove_dir_all(directory_path));
    match join.await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(format!("Failed delete directory: {:?}", e)),
        Err(e) => Err(format!("Failed delete directory: {:?}", e)),
    }
}

/// Native dialogs block their thread until dismissed, so both run on the
/// blocking pool. `AppHandle` is `Send + Sync`, which makes the move cheap.
pub async fn instance_native_ok_dialog_window(
    app: AppHandle,
    message: String,
    kind: tauri_plugin_dialog::MessageDialogKind,
) -> () {
    let _ = tokio::task::spawn_blocking(move || {
        app.dialog()
            .message(message)
            .kind(kind)
            .buttons(MessageDialogButtons::Ok)
            .blocking_show();
    })
    .await;
    ()
}

pub async fn instance_native_ask_dialog_window(
    app: AppHandle,
    message: String,
    kind: tauri_plugin_dialog::MessageDialogKind,
) -> bool {
    let join = tokio::task::spawn_blocking(move || {
        app.dialog()
            .message(message)
            .kind(kind)
            .buttons(MessageDialogButtons::OkCancelCustom(
                String::from(YES_SENTANCE_CASE),
                String::from(NO_SENTANCE_CASE),
            ))
            .blocking_show()
    });
    // A failed join (panic inside the dialog) counts as "no".
    join.await.unwrap_or(false)
}

/// Extracts `archive_file_path` next to itself and returns the extracted
/// version folder. The work is synchronous archive I/O over hundreds of
/// megabytes, so it runs on the blocking pool.
///
/// Supported per platform: `.zip` everywhere, `.tar.xz` on Linux and macOS,
/// `.dmg` on macOS.
pub async fn open_archive(
    archive_file_path: std::path::PathBuf,
) -> Result<std::path::PathBuf, String> {
    let join = tokio::task::spawn_blocking(move || open_archive_blocking(archive_file_path));
    match join.await {
        Ok(v) => v,
        Err(e) => Err(format!("Failed open archive: {:?}", e)),
    }
}

fn open_archive_blocking(
    archive_file_path: std::path::PathBuf,
) -> Result<std::path::PathBuf, String> {
    let lower_name: String = archive_file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if lower_name.ends_with(".zip") {
        return extract_zip(&archive_file_path);
    }
    #[cfg(not(windows))]
    {
        if lower_name.ends_with(".tar.xz") {
            return extract_tar_xz(&archive_file_path);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if lower_name.ends_with(".dmg") {
            return extract_dmg(&archive_file_path);
        }
    }
    Err(format!(
        "Failed open archive: unsupported archive type for this platform: {}",
        archive_file_path.display()
    ))
}

/// Rejects archive entry paths that are absolute or contain `..`, so an entry
/// can never be written outside the extraction directory.
#[cfg(not(windows))]
fn is_enclosed_archive_path(relative: &std::path::Path) -> bool {
    if relative.as_os_str().is_empty() || relative.is_absolute() {
        return false;
    }
    relative.components().all(|c| match c {
        std::path::Component::Normal(_) | std::path::Component::CurDir => true,
        _ => false,
    })
}

/// Unpacks a `.tar.xz` next to itself. Blender tarballs hold a single
/// top-level folder (`blender-4.5.1-linux-x64/`), which is what is returned;
/// if no single top-level folder can be determined the archive's stem is used.
#[cfg(not(windows))]
fn extract_tar_xz(archive_file_path: &std::path::Path) -> Result<std::path::PathBuf, String> {
    let extract_dir: &std::path::Path = match archive_file_path.parent() {
        Some(parent) => parent,
        None => return Err(format!("Failed open archive")),
    };
    let file = match std::fs::File::open(archive_file_path) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed open archive: {:?}", e)),
    };
    let decoder = liblzma::read::XzDecoder::new(std::io::BufReader::new(file));
    let mut archive = tar::Archive::new(decoder);
    let entries = match archive.entries() {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed open archive: {:?}", e)),
    };
    let mut top_level: Option<std::ffi::OsString> = None;
    let mut single_top_level: bool = true;
    for entry in entries {
        let mut entry = match entry {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed open archive: {:?}", e)),
        };
        let relative: std::path::PathBuf = match entry.path() {
            Ok(v) => v.to_path_buf(),
            Err(e) => return Err(format!("Failed open archive: {:?}", e)),
        };
        // Entry names are attacker-controlled: absolute paths and `..` are
        // rejected; the `starts_with` check is a second line of defence.
        if !is_enclosed_archive_path(&relative) {
            return Err(format!(
                "Failed open archive: entry '{}' would escape the extraction directory",
                relative.display()
            ));
        }
        let outpath: std::path::PathBuf = extract_dir.join(&relative);
        if !outpath.starts_with(extract_dir) {
            return Err(format!(
                "Failed open archive: entry '{}' would escape the extraction directory",
                relative.display()
            ));
        }
        let first_component: Option<&std::ffi::OsStr> = relative.components().find_map(|c| match c {
            std::path::Component::Normal(name) => Some(name),
            _ => None,
        });
        if let Some(first) = first_component {
            match &top_level {
                Some(existing) if existing.as_os_str() != first => single_top_level = false,
                Some(_) => {}
                None => top_level = Some(first.to_os_string()),
            }
        }
        let entry_type = entry.header().entry_type();
        // Links inside an archive are never materialised.
        if entry_type.is_symlink() || entry_type.is_hard_link() {
            continue;
        }
        if entry_type.is_dir() {
            if let Err(e) = std::fs::create_dir_all(&outpath) {
                return Err(format!("Failed open archive: {:?}", e));
            }
        } else if entry_type.is_file() {
            if let Some(parent) = outpath.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            // `unpack` writes the file and applies its mode bits, which keeps
            // the `blender` binary executable.
            if let Err(e) = entry.unpack(&outpath) {
                return Err(format!("Failed open archive: {:?}", e));
            }
        }
    }
    let result_dir: std::path::PathBuf = match (top_level, single_top_level) {
        (Some(name), true) => extract_dir.join(name),
        _ => {
            let file_name: String = archive_file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let stem: &str = file_name
                .strip_suffix(".tar.xz")
                .or_else(|| file_name.strip_suffix(".TAR.XZ"))
                .unwrap_or(file_name.as_str());
            if stem.is_empty() {
                return Err(format!("Failed open archive"));
            }
            extract_dir.join(stem)
        }
    };
    Ok(result_dir)
}

/// Copies a directory tree. Symbolic links are recreated as links (relative
/// targets kept as they are) instead of being followed: the `Frameworks`
/// folders inside a `.app` bundle rely on `Versions/Current` style links.
#[cfg(unix)]
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    if let Err(e) = std::fs::create_dir_all(dst) {
        return Err(format!("Failed copy directory {}: {:?}", dst.display(), e));
    }
    let entries = match std::fs::read_dir(src) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed copy directory {}: {:?}", src.display(), e)),
    };
    for entry in entries {
        let entry = match entry {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed copy directory {}: {:?}", src.display(), e)),
        };
        let src_path: std::path::PathBuf = entry.path();
        let dst_path: std::path::PathBuf = dst.join(entry.file_name());
        let file_type = match std::fs::symlink_metadata(&src_path) {
            Ok(v) => v.file_type(),
            Err(e) => return Err(format!("Failed copy directory {}: {:?}", src_path.display(), e)),
        };
        if file_type.is_symlink() {
            let target: std::path::PathBuf = match std::fs::read_link(&src_path) {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed copy directory {}: {:?}", src_path.display(), e)),
            };
            if let Err(e) = std::os::unix::fs::symlink(&target, &dst_path) {
                return Err(format!("Failed copy directory {}: {:?}", dst_path.display(), e));
            }
        } else if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            // `fs::copy` carries the permission bits over, so binaries stay executable.
            if let Err(e) = std::fs::copy(&src_path, &dst_path) {
                return Err(format!("Failed copy directory {}: {:?}", src_path.display(), e));
            }
        }
    }
    Ok(())
}

/// Mounts a `.dmg` read-only on a private mount point, copies the `.app`
/// inside it to `<extract_dir>/<archive stem>/Blender.app` and detaches the
/// image again. Returns `<extract_dir>/<archive stem>`.
#[cfg(target_os = "macos")]
fn extract_dmg(archive_file_path: &std::path::Path) -> Result<std::path::PathBuf, String> {
    let extract_dir: &std::path::Path = match archive_file_path.parent() {
        Some(parent) => parent,
        None => return Err(format!("Failed open archive")),
    };
    let stem: String = match archive_file_path.file_stem() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(format!("Failed open archive")),
    };
    let target_dir: std::path::PathBuf = extract_dir.join(&stem);
    let mount_dir: std::path::PathBuf =
        std::env::temp_dir().join(format!("blenderbase-dmg-{}", uuid::Uuid::new_v4()));
    if let Err(e) = std::fs::create_dir_all(&mount_dir) {
        return Err(format!("Failed open archive: {:?}", e));
    }
    let attach = std::process::Command::new("hdiutil")
        .arg("attach")
        .arg("-nobrowse")
        .arg("-readonly")
        .arg("-mountpoint")
        .arg(&mount_dir)
        .arg(archive_file_path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output();
    match attach {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            let _ = std::fs::remove_dir(&mount_dir);
            return Err(format!(
                "Failed open archive: hdiutil attach failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Err(e) => {
            let _ = std::fs::remove_dir(&mount_dir);
            return Err(format!("Failed open archive: hdiutil attach failed: {:?}", e));
        }
    }
    let copied: Result<(), String> = (|| {
        let mut app_path: Option<std::path::PathBuf> = None;
        let preferred: std::path::PathBuf = mount_dir.join("Blender.app");
        if preferred.is_dir() {
            app_path = Some(preferred);
        } else if let Ok(entries) = std::fs::read_dir(&mount_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let is_app = path
                    .extension()
                    .map(|e| e.eq_ignore_ascii_case("app"))
                    .unwrap_or(false);
                if is_app && path.is_dir() {
                    app_path = Some(path);
                    break;
                }
            }
        }
        let app_path: std::path::PathBuf = match app_path {
            Some(v) => v,
            None => return Err(String::from("Failed open archive: no .app found in the disk image")),
        };
        if let Err(e) = std::fs::create_dir_all(&target_dir) {
            return Err(format!("Failed open archive: {:?}", e));
        }
        copy_dir_recursive(&app_path, &target_dir.join("Blender.app"))
    })();
    let _ = std::process::Command::new("hdiutil")
        .arg("detach")
        .arg(&mount_dir)
        .arg("-force")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output();
    let _ = std::fs::remove_dir(&mount_dir);
    match copied {
        Ok(()) => Ok(target_dir),
        Err(e) => Err(e),
    }
}

/// Unpacks a `.zip` next to itself and returns `<extract_dir>/<archive stem>`,
/// the single top-level folder Blender release archives contain.
fn extract_zip(archive_file_path: &std::path::Path) -> Result<std::path::PathBuf, String> {
    let file = match std::fs::File::open(&archive_file_path) {
        Ok(v) => v,
        Err(e) => return Err(format!("{:?}", e)),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed open archive: {:?}", e)),
    };
    let archive_dir = match archive_file_path.file_stem() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(format!("Failed open archive")),
    };
    let extract_dir = match archive_file_path.parent() {
        Some(parent) => parent,
        None => return Err(format!("Failed open archive")),
    };
    for i in 0..archive.len() {
        let mut inner_file = match archive.by_index(i) {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed open archive: {:?}", e)),
        };
        // Entry names are attacker-controlled. `enclosed_name` rejects absolute
        // paths and any `..` component; the `starts_with` check is a second
        // line of defence after joining.
        let relative = match inner_file.enclosed_name() {
            Some(v) => v,
            None => {
                return Err(format!(
                    "Failed open archive: entry '{}' would escape the extraction directory",
                    inner_file.name()
                ))
            }
        };
        let outpath = extract_dir.join(relative);
        if !outpath.starts_with(extract_dir) {
            return Err(format!(
                "Failed open archive: entry '{}' would escape the extraction directory",
                inner_file.name()
            ));
        }
        // Symbolic links inside an archive are never materialised.
        const S_IFMT: u32 = 0o170000;
        const S_IFLNK: u32 = 0o120000;
        if inner_file
            .unix_mode()
            .map(|m| m & S_IFMT == S_IFLNK)
            .unwrap_or(false)
        {
            continue;
        }
        if inner_file.is_dir() {
            match std::fs::create_dir_all(&outpath) {
                Ok(_) => {}
                Err(e) => return Err(format!("Failed open archive: {:?}", e)),
            }
        } else {
            if let Some(parent) = outpath.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let mut outfile = match std::fs::File::create(&outpath) {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed open archive: {:?}", e)),
            };
            if let Err(e) = std::io::copy(&mut inner_file, &mut outfile) {
                return Err(format!("Failed open archive: {:?}", e));
            }
        }
    }
    Ok(extract_dir.join(archive_dir))
}

pub fn create_archive(file_path: std::path::PathBuf) -> Result<std::path::PathBuf, String> {
    let file_name = match file_path.file_name() {
        Some(v) => v,
        None => return Err(format!("Failed to archive file")),
    };
    let zip_path = file_path.with_extension("zip");
    let zip_file = match std::fs::File::create(&zip_path) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed create archive: {:?}", e)),
    };
    let mut zip_writer = zip::ZipWriter::new(zip_file);
    let options: zip::write::FileOptions<()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let mut buffer = Vec::new();
    let mut source_file = match std::fs::File::open(&file_path) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed create archive: {:?}", e)),
    };
    match source_file.read_to_end(&mut buffer) {
        Ok(_) => {}
        Err(e) => return Err(format!("Failed create archive: {:?}", e)),
    }
    match zip_writer.start_file(file_name.to_string_lossy().to_string(), options) {
        Ok(_) => {}
        Err(e) => return Err(format!("Failed create archive: {:?}", e)),
    }
    match zip_writer.write_all(&buffer) {
        Ok(_) => {}
        Err(e) => return Err(format!("Failed create archive: {:?}", e)),
    }
    match zip_writer.finish() {
        Ok(_) => {}
        Err(e) => return Err(format!("Failed create archive: {:?}", e)),
    }
    Ok(zip_path)
}

pub async fn create_directory_path(path: std::path::PathBuf) -> Result<(), String> {
    match std::fs::create_dir_all(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed create directory path: {}", e)),
    }
}

/// The directory under which Blender keeps one configuration folder per
/// series (`4.5/config`, `5.0/config`, ...):
///
/// - Windows: `%APPDATA%\Blender Foundation\Blender`
/// - macOS: `~/Library/Application Support/Blender`
/// - Linux: `~/.config/blender`
///
/// Every lookup of Blender's user configuration goes through here.
pub fn blender_config_root() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir().map(|d| {
            d.join(crate::core::BLENDER_FOUNDATION)
                .join(crate::core::BLENDER)
        })
    }
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|d| {
            d.join("Library")
                .join("Application Support")
                .join(crate::core::BLENDER)
        })
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        dirs::config_dir().map(|d| d.join("blender"))
    }
}

/// Get the main storage device path, that is returned from the `dirs::home_dir` associated method.
///
/// Note this function does not use sysinfo to retrieve the storage device data for the host machine,
/// since the order for the drive will be decided by the hierarchy of the partitions (the primary partition being at
/// index `0`. This could lead to the user having his OS on the C drive, but the primary partition being on the
/// D drive, which could lead to different behavior between across user systems.
///
/// Alternative using sysinfo:
/// ```
/// let mut disks: sysinfo::Disks = sysinfo::Disks::new_with_refreshed_list(); // The order is identified by the primary partition,
///                                                                            // which can also not be the one, in which Win is installed in.
/// Ok(PathBuf::from(disks.list()[0].mount_point().to_string_lossy().to_string())) // But the order depends on the Host configuration.
/// ```
///
/// OS: Win.
pub async fn get_main_storage_device_root_path() -> Result<std::path::PathBuf, String> {
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Ok(std::path::PathBuf::from("/"))
    }
    #[cfg(target_os = "windows")]
    {
        match dirs::home_dir() {
            Some(v) => {
                let v_string = match v.to_str() {
                    Some(v) => v,
                    None => return Err(format!("Failed get main storage device root path")),
                };
                let mut root_part: &str = "";
                if let Some(index) = v_string.find('\\') {
                    root_part = &v_string[0..index + 1];
                }
                Ok(std::path::PathBuf::from(root_part))
            }
            None => Err(format!("Failed get main storage device root path")),
        }
    }
    #[cfg(target_os = "macos")]
    {
        match dirs::home_dir() {
            Some(v) => {
                // let ans = val.clone();
                // match val.parent() {
                //     Some(v) => {
                //         Ok(val.to_path_buf())
                //     },
                //     None => {
                //         Err(format!("Failed get main storage device root path"))
                //     }
                // }
                Ok(v)
            }
            None => Err(format!("Failed get main storage device root path")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("blenderbase-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn sha256_listing_lookup_matches_exact_file_name() {
        let listing = "\
2dc8e4bf6fe2ba93027ca752e2dec90c761037b7578204662a564d503ccb40a1  blender-4.5.1-windows-x64.msi
aab6b8a0d0d9d5b3f0e0b7d1c2a3e4f5061728394a5b6c7d8e9f0a1b2c3d4e5f  blender-4.5.1-windows-x64.zip
not-a-hash  blender-4.5.1-linux-x64.tar.xz
";
        assert_eq!(
            find_sha256_in_listing(listing, "blender-4.5.1-windows-x64.zip").as_deref(),
            Some("aab6b8a0d0d9d5b3f0e0b7d1c2a3e4f5061728394a5b6c7d8e9f0a1b2c3d4e5f")
        );
        assert_eq!(find_sha256_in_listing(listing, "blender-4.5.1-linux-x64.tar.xz"), None);
        assert_eq!(find_sha256_in_listing(listing, "blender-4.5.1-windows-x64"), None);
        assert!(is_sha256_hex("2dc8e4bf6fe2ba93027ca752e2dec90c761037b7578204662a564d503ccb40a1"));
        assert!(!is_sha256_hex("2dc8e4bf"));
    }

    /// The path used to be spliced into the script inside single quotes, so a
    /// directory name like this one closed the literal and ran the remainder.
    /// It now travels as an environment variable and must come back as data.
    #[cfg(windows)]
    #[test]
    fn permission_probe_treats_quotes_and_semicolons_as_data() {
        let dir = temp_dir().join("bb 'quote'; Write-Output INJECTED; '");
        std::fs::create_dir_all(&dir).unwrap();
        let details = get_permission_details(&dir.to_string_lossy()).unwrap();
        assert!(details.read, "the probe must report the real ACL of the directory");
        let _ = std::fs::remove_dir_all(dir.parent().unwrap());
    }

    #[tokio::test]
    async fn sha256_of_file_matches_known_digest() {
        let dir = temp_dir();
        let file = dir.join("abc.txt");
        std::fs::write(&file, b"abc").unwrap();
        let digest = sha256_of_file(file).await.unwrap();
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn open_archive_refuses_entries_that_escape_the_target() {
        let outer = temp_dir();
        let inner = outer.join("inner");
        std::fs::create_dir_all(&inner).unwrap();
        let archive = inner.join("evil.zip");
        {
            let file = std::fs::File::create(&archive).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            let options: zip::write::FileOptions<()> = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            writer.start_file("../escaped.txt", options).unwrap();
            writer.write_all(b"owned").unwrap();
            writer.finish().unwrap();
        }
        let result = open_archive(archive).await;
        assert!(result.is_err(), "traversal entry must be rejected");
        assert!(
            !outer.join("escaped.txt").exists(),
            "no file may be written outside the extraction directory"
        );
        let _ = std::fs::remove_dir_all(&outer);
    }

    #[tokio::test]
    async fn open_archive_extracts_well_formed_entries() {
        let dir = temp_dir();
        let archive = dir.join("good.zip");
        {
            let file = std::fs::File::create(&archive).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            let options: zip::write::FileOptions<()> = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            writer.start_file("good/hello.txt", options).unwrap();
            writer.write_all(b"hi").unwrap();
            writer.finish().unwrap();
        }
        let extracted = open_archive(archive).await.unwrap();
        assert_eq!(extracted, dir.join("good"));
        assert_eq!(std::fs::read_to_string(dir.join("good").join("hello.txt")).unwrap(), "hi");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
