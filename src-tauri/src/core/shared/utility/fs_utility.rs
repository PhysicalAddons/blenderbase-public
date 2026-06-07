use std::{
    io::{Read, Write},
    os::windows::process::CommandExt,
};

use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

use crate::core::{
    PermissionDetails, COMMAND, CREATE_NO_WINDOW_FLAG, GET_PATH_PERMISSIONS_PS1_EXPRESSION, HIDDEN,
    NO_SENTANCE_CASE, POWERSHELL, WINDOW_STYLE, YES_SENTANCE_CASE,
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
    let directory_path_option = app.dialog().file().blocking_pick_folder();
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
    #[cfg(target_os = "windows")]
    match std::process::Command::new("explorer")
        .arg(parent_directory)
        .spawn()
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format!("Failed open in file explorer: {:?}", e)),
    }
    #[cfg(target_os = "macos")]
    match std::process::Command::new("open")
        .arg(parent_directory)
        .spawn()
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(format!("{:?}", e)),
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

pub fn run_ps1_expression_as_string(args: Vec<&str>) -> Result<String, String> {
    let out = std::process::Command::new(POWERSHELL)
        .args(args)
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

pub fn get_permission_details(path: &str) -> Result<PermissionDetails, String> {
    let expr = format!(
        r#"
$Path = '{}'
{}
    "#,
        path, GET_PATH_PERMISSIONS_PS1_EXPRESSION
    );
    let args = vec![WINDOW_STYLE, HIDDEN, COMMAND, expr.as_str()];
    let out = match run_ps1_expression_as_string(args) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed get permission detail: {:?}", e)),
    };
    let result: PermissionDetails = match serde_json::from_str(&out) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed get permission detail: {:?}", e)),
    };
    return Ok(result);
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
    let output = command.args(arguments).output();
    match output {
        Ok(_) => Ok(()),
        Err(e) => return Err(format!("Failed launch executable: {:?}", e)),
    }
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

pub async fn delete_directory(directory_path: std::path::PathBuf) -> Result<(), String> {
    match std::fs::remove_dir_all(directory_path) {
        Ok(_) => Ok(()),
        Err(e) => return Err(format!("Failed delete directory: {:?}", e)),
    }
}

pub fn instance_native_ok_dialog_window(
    app: AppHandle,
    message: String,
    kind: tauri_plugin_dialog::MessageDialogKind,
) -> () {
    app.dialog()
        .message(message)
        .kind(kind)
        .buttons(MessageDialogButtons::Ok)
        .blocking_show();
    ()
}

pub fn instance_native_ask_dialog_window(
    app: AppHandle,
    message: String,
    kind: tauri_plugin_dialog::MessageDialogKind,
) -> bool {
    let answer = app
        .dialog()
        .message(message)
        .kind(kind)
        .buttons(MessageDialogButtons::OkCancelCustom(
            // B (2.a.) .buttons(
            String::from(YES_SENTANCE_CASE),
            String::from(NO_SENTANCE_CASE),
        ))
        .blocking_show();
    return answer;
}

pub async fn open_archive(
    archive_file_path: std::path::PathBuf,
) -> Result<std::path::PathBuf, String> {
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
        let outpath = extract_dir.join(inner_file.name());
        if inner_file.name().ends_with('/') {
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
