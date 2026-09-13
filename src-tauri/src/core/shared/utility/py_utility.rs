use std::{process::Stdio, time::Duration};

#[cfg(target_os = "windows")]
use crate::core::CREATE_NO_WINDOW_FLAG;

/// Marker a Blender-side script prints in front of its JSON result line.
pub const BLENDERBASE_JSON_MARKER: &str = "BLENDERBASE_JSON:";

/// Resolves the executable to use for background (headless) runs.
///
/// On Windows registered Blender versions point at `blender-launcher.exe`,
/// which is the right thing to launch for the UI. It is a small GUI stub,
/// though: it starts `blender.exe` and returns at once without forwarding
/// stdout or the exit code, so a script run through it never reports anything
/// back. For background scripts the real binary next to it is used instead.
/// The other platforms have no launcher stub, so the path is used as is.
fn background_executable(path: &std::path::Path) -> std::path::PathBuf {
    resolve_blender_console_executable(path)
}

/// Formats a Rust string as a Python string literal (JSON string syntax is valid Python).
pub fn py_string_literal(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| String::from("\"\""))
}

/// Runs a Python script inside a Blender build in background mode (no window, no UI) and
/// returns its stdout. A non-zero exit or a script exception is reported as an error.
pub async fn run_blender_python(
    executable_file_path: &std::path::Path,
    script: &str,
    timeout_secs: u64,
) -> Result<String, String> {
    let executable_file_path = background_executable(executable_file_path);
    let mut command = tokio::process::Command::new(&executable_file_path);
    command
        .arg("--background")
        .arg("--python-exit-code")
        .arg("1")
        .arg("--python-expr")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW_FLAG);
    let output = match tokio::time::timeout(Duration::from_secs(timeout_secs), command.output()).await {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => return Err(format!("Failed to start Blender: {:?}", e)),
        Err(_) => {
            return Err(format!(
                "Blender did not finish within {} seconds",
                timeout_secs
            ))
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let detail = stderr
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim()
            .to_string();
        return Err(format!(
            "Blender exited with {}{}",
            output.status,
            if detail.is_empty() { String::new() } else { format!(": {}", detail) }
        ));
    }
    Ok(stdout)
}

/// Extracts the JSON payload printed after [`BLENDERBASE_JSON_MARKER`].
pub fn extract_json_payload(stdout: &str) -> Result<String, String> {
    // The marker may be preceded by other output on the same line (an addon
    // printing without a trailing newline), so it is searched for, not
    // matched as a prefix.
    for line in stdout.lines() {
        if let Some(index) = line.find(BLENDERBASE_JSON_MARKER) {
            return Ok(line[index + BLENDERBASE_JSON_MARKER.len()..].to_string());
        }
    }
    Err(String::from("Blender did not report a result"))
}

/// On Windows installed versions are registered with `blender-launcher.exe`, which starts the
/// real process and exits at once without forwarding its output. Scripts and probes use the
/// sibling `blender.exe` whenever it exists. macOS and Linux have no launcher stub, so the
/// registered executable is returned unchanged there.
#[cfg(target_os = "windows")]
pub fn resolve_blender_console_executable(path: &std::path::Path) -> std::path::PathBuf {
    let is_launcher = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase().starts_with("blender-launcher"))
        .unwrap_or(false);
    if is_launcher {
        if let Some(parent) = path.parent() {
            let console = parent.join(crate::core::BLENDER_EXE);
            if console.is_file() {
                return console;
            }
        }
    }
    path.to_path_buf()
}

#[cfg(not(target_os = "windows"))]
pub fn resolve_blender_console_executable(path: &std::path::Path) -> std::path::PathBuf {
    path.to_path_buf()
}

/// What `blender --version` reports about a build.
#[derive(Debug, Default, Clone)]
pub struct BlenderBuildInfo {
    /// First line, e.g. `Blender 5.2.1 LTS` or `Blender 4.5.4 LTS Release Candidate`.
    pub title: String,
    pub build_date: String,
    pub commit_date: String,
    pub hash: String,
    pub branch: String,
    /// Derived from the title: lts / stable / candidate / beta / alpha.
    pub cycle: String,
}

/// Runs `blender --version` (fast, no Python) and parses the build details.
pub async fn probe_blender_build_info(
    executable_file_path: &std::path::Path,
    timeout_secs: u64,
) -> Result<BlenderBuildInfo, String> {
    let mut command = tokio::process::Command::new(executable_file_path);
    command
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW_FLAG);
    let output = match tokio::time::timeout(Duration::from_secs(timeout_secs), command.output()).await {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => return Err(format!("Failed to start Blender: {:?}", e)),
        Err(_) => return Err(format!("Blender did not answer within {} seconds", timeout_secs)),
    };
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let mut info = BlenderBuildInfo::default();
    for line in stdout.lines() {
        let t = line.trim();
        if info.title.is_empty() && t.starts_with("Blender ") {
            info.title = t.to_string();
        } else if let Some(v) = t.strip_prefix("build date:") {
            info.build_date = v.trim().to_string();
        } else if let Some(v) = t.strip_prefix("build commit date:") {
            info.commit_date = v.trim().to_string();
        } else if let Some(v) = t.strip_prefix("build hash:") {
            info.hash = v.trim().to_string();
        } else if let Some(v) = t.strip_prefix("build branch:") {
            info.branch = v.trim().to_string();
        }
    }
    if info.title.is_empty() {
        return Err(String::from("Blender did not report its version"));
    }
    let lower = info.title.to_lowercase();
    info.cycle = if lower.contains("candidate") {
        "candidate"
    } else if lower.contains("alpha") {
        "alpha"
    } else if lower.contains("beta") {
        "beta"
    } else if lower.contains("lts") {
        "lts"
    } else {
        "stable"
    }
    .to_string();
    Ok(info)
}
