//! Where Blender ends up when it was installed without Blenderbase: the
//! Blender installer, Steam and the package managers each have their own
//! folder. The sweep lists the ones present on this machine so they can be
//! registered as installation locations on first launch or from Settings.
//!
//! An installation location is always the folder *above* the version folders
//! (the version scan lists its subfolders and keeps the ones that hold a
//! Blender executable), so a candidate is either such a container
//! (`Program Files\Blender Foundation`, `/Applications`) or one version
//! folder whose parent gets registered (`steamapps\common\Blender`).

use std::path::{Path, PathBuf};

use crate::core::{blender_executable_for_entry, default_installation_directory, BLENDER_FOUNDATION};

/// A folder that holds Blender versions, found by the sweep.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DetectedBlenderLocation {
    /// The folder to register: its subfolders (or app bundles) are the versions.
    pub directory_path: String,
    /// Where the folder comes from, for the status line: "Program Files", "Steam", ...
    pub label: String,
    /// The subfolders of `directory_path` that hold a Blender executable.
    pub version_dirs: Vec<String>,
}

/// How a candidate folder relates to the versions inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateKind {
    /// The folder holds version folders: `Program Files\Blender Foundation`, `/Applications`.
    Container,
    /// The folder is one Blender version itself: `steamapps\common\Blender`,
    /// `/snap/blender/current`. Its parent is what gets registered.
    VersionDir,
}

struct Candidate {
    path: PathBuf,
    label: &'static str,
    kind: CandidateKind,
}

impl Candidate {
    fn container(path: PathBuf, label: &'static str) -> Self {
        Self { path, label, kind: CandidateKind::Container }
    }
    fn version_dir(path: PathBuf, label: &'static str) -> Self {
        Self { path, label, kind: CandidateKind::VersionDir }
    }
}

/// The folders where Blender is installed on this machine, with the versions
/// found in each. Only folders that hold at least one Blender executable are
/// returned, and none is required to exist beforehand.
pub fn detect_blender_installations() -> Vec<DetectedBlenderLocation> {
    detect_from(candidates())
}

/// The places to look, per platform. Existence is checked by the caller.
fn candidates() -> Vec<Candidate> {
    let mut list: Vec<Candidate> = Vec::new();
    #[cfg(target_os = "windows")]
    {
        // The Blender installer (MSI) puts a `Blender <series>` folder here.
        if let Some(program_files) = std::env::var_os("ProgramFiles") {
            list.push(Candidate::container(
                PathBuf::from(program_files).join(BLENDER_FOUNDATION),
                "Program Files",
            ));
        }
        // Steam keeps one `Blender` folder per library, next to the other games.
        for library in steam_library_folders() {
            list.push(Candidate::version_dir(
                library.join("steamapps").join("common").join("Blender"),
                "Steam",
            ));
        }
        // Per-user installs land next to the other per-user programs.
        if let Some(local_app_data) = dirs::data_local_dir() {
            list.push(Candidate::container(local_app_data.join("Programs"), "AppData Programs"));
        }
        // Not covered: the Microsoft Store install under `%ProgramFiles%\WindowsApps`.
        // That folder denies listing to normal users and its executable cannot be
        // started by path (the Store version only runs through its app alias), so
        // there is nothing Blenderbase could register.
    }
    #[cfg(target_os = "macos")]
    {
        // `Blender.app`, or a renamed `Blender 4.2.app`, dragged into either
        // Applications folder. The per-entry resolution understands bundles.
        list.push(Candidate::container(PathBuf::from("/Applications"), "Applications"));
        if let Some(home) = dirs::home_dir() {
            list.push(Candidate::container(home.join("Applications"), "your Applications folder"));
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // Unpacked release archives, one folder per version.
        list.push(Candidate::container(PathBuf::from("/opt"), "/opt"));
        // A distribution package that keeps the whole release layout in one folder.
        list.push(Candidate::version_dir(PathBuf::from("/usr/lib/blender"), "/usr/lib"));
        // snap: `current` links to the active revision. The revision folders next
        // to it hold the same build and show up as further entries after the scan.
        // The snap is classic-confined, so its binary starts by path.
        list.push(Candidate::version_dir(PathBuf::from("/snap/blender/current"), "snap"));
        // flatpak: the release archive is unpacked under `files/blender`, for the
        // system-wide and the per-user installation. The binary is started by
        // path rather than through `flatpak run`, so it depends on the system
        // providing what the Flatpak runtime would; this is best effort.
        list.push(Candidate::version_dir(
            PathBuf::from("/var/lib/flatpak/app/org.blender.Blender/current/active/files/blender"),
            "Flatpak",
        ));
        if let Some(data) = dirs::data_dir() {
            list.push(Candidate::version_dir(
                data.join("flatpak/app/org.blender.Blender/current/active/files/blender"),
                "Flatpak",
            ));
        }
    }
    // Blenderbase's own folder: versions from an earlier install whose database
    // is gone would otherwise only reappear after the first download prompt.
    list.push(Candidate::container(default_installation_directory(), "the Blenderbase folder"));
    list
}

/// The candidates that exist and hold a Blender version, each folder once.
fn detect_from(candidates: Vec<Candidate>) -> Vec<DetectedBlenderLocation> {
    let mut detected: Vec<DetectedBlenderLocation> = Vec::new();
    for candidate in candidates {
        let location_dir: PathBuf = match candidate.kind {
            CandidateKind::Container => candidate.path,
            CandidateKind::VersionDir => {
                if !blender_executable_for_entry(&candidate.path).exists() {
                    continue;
                }
                match candidate.path.parent() {
                    Some(parent) => parent.to_path_buf(),
                    None => continue,
                }
            }
        };
        let version_dirs = blender_version_dirs_in(&location_dir);
        if version_dirs.is_empty() {
            continue;
        }
        let directory_path = location_dir.to_string_lossy().to_string();
        if detected.iter().any(|d| same_location_path(&d.directory_path, &directory_path)) {
            continue;
        }
        detected.push(DetectedBlenderLocation {
            directory_path,
            label: candidate.label.to_string(),
            version_dirs: version_dirs.iter().map(|p| p.to_string_lossy().to_string()).collect(),
        });
    }
    detected
}

/// The subfolders of `dir` that hold a Blender executable, sorted by path.
/// Uses the same per-entry resolution as the version scan, so this is exactly
/// what registering `dir` as a location would list.
pub fn blender_version_dirs_in(dir: &Path) -> Vec<PathBuf> {
    let entries = match std::fs::read_dir(dir) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let mut found: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && blender_executable_for_entry(path).exists())
        .collect();
    found.sort();
    found
}

/// Whether two location paths name the same folder. Trailing separators are
/// ignored; on Windows so are letter case and the slash direction, since the
/// registry, environment variables and the folder picker spell paths
/// differently.
pub fn same_location_path(a: &str, b: &str) -> bool {
    normalize_location_path(a) == normalize_location_path(b)
}

fn normalize_location_path(path: &str) -> String {
    let trimmed = path.trim().trim_end_matches(['/', '\\']);
    #[cfg(windows)]
    {
        trimmed.replace('/', "\\").to_lowercase()
    }
    #[cfg(not(windows))]
    {
        trimmed.to_string()
    }
}

/// Spells a Windows path the way the folder picker does: backslashes and an
/// upper-case drive letter. The registry stores Steam's folder as
/// `c:/program files (x86)/steam`.
pub fn native_windows_path(path: &str) -> String {
    let mut spelled = path.trim().replace('/', "\\");
    let bytes = spelled.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_lowercase() {
        spelled[..1].make_ascii_uppercase();
    }
    spelled
}

/// The `"path"` entries of Steam's `libraryfolders.vdf`, in file order. The
/// file is a key/value tree with one quoted key and value per line:
///
/// ```text
/// "libraryfolders"
/// {
///     "0"
///     {
///         "path"      "C:\\Program Files (x86)\\Steam"
///         ...
/// ```
pub fn parse_steam_library_folders(vdf: &str) -> Vec<PathBuf> {
    vdf.lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split('"').collect();
            if fields.len() < 5 || fields[1] != "path" {
                return None;
            }
            let value = fields[3].replace("\\\\", "\\");
            if value.is_empty() {
                None
            } else {
                Some(PathBuf::from(value))
            }
        })
        .collect()
}

/// Every Steam library folder listed in `libraryfolders.vdf`, plus Steam's
/// own folder when the file does not list it. Empty when Steam is not
/// installed. The file's spelling is preferred: the registry has Steam's
/// folder in lower case (`c:/program files (x86)/steam`), the file as on disk.
#[cfg(target_os = "windows")]
fn steam_library_folders() -> Vec<PathBuf> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    let steam_path: String = match RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Valve\\Steam")
        .and_then(|key| key.get_value::<String, _>("SteamPath"))
    {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let steam_dir = PathBuf::from(native_windows_path(&steam_path));
    let mut libraries: Vec<PathBuf> = Vec::new();
    let vdf_path = steam_dir.join("steamapps").join("libraryfolders.vdf");
    if let Ok(vdf) = std::fs::read_to_string(vdf_path) {
        libraries = parse_steam_library_folders(&vdf)
            .iter()
            .map(|library| PathBuf::from(native_windows_path(&library.to_string_lossy())))
            .collect();
    }
    let steam_dir_listed = libraries
        .iter()
        .any(|l| same_location_path(&l.to_string_lossy(), &steam_dir.to_string_lossy()));
    if !steam_dir_listed {
        libraries.push(steam_dir);
    }
    libraries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blender_executable_in;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("blenderbase-sweep-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A version folder `name` inside `location`, with the executable this
    /// platform registers (`blender-launcher.exe`, `Blender.app/...`, `blender`).
    fn make_version_dir(location: &Path, name: &str) -> PathBuf {
        let dir = location.join(name);
        let executable = blender_executable_in(&dir);
        std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
        std::fs::write(&executable, b"").unwrap();
        dir
    }

    fn paths(detected: &DetectedBlenderLocation) -> Vec<PathBuf> {
        detected.version_dirs.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn container_lists_only_subfolders_with_an_executable() {
        let root = temp_dir();
        let location = root.join("Blender Foundation");
        let a = make_version_dir(&location, "Blender 4.2");
        let b = make_version_dir(&location, "Blender 4.5");
        std::fs::create_dir_all(location.join("Blender 3.6")).unwrap(); // uninstalled, empty
        std::fs::write(location.join("notes.txt"), b"").unwrap();

        let found = detect_from(vec![Candidate::container(location.clone(), "Program Files")]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].directory_path, location.to_string_lossy());
        assert_eq!(found[0].label, "Program Files");
        assert_eq!(paths(&found[0]), vec![a, b]);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn version_dir_candidate_registers_its_parent() {
        let root = temp_dir();
        let common = root.join("steamapps").join("common");
        let blender = make_version_dir(&common, "Blender");
        std::fs::create_dir_all(common.join("Some Game")).unwrap();

        let found = detect_from(vec![Candidate::version_dir(blender.clone(), "Steam")]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].directory_path, common.to_string_lossy());
        assert_eq!(paths(&found[0]), vec![blender]);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn absent_and_empty_candidates_are_skipped() {
        let root = temp_dir();
        let empty = root.join("empty");
        std::fs::create_dir_all(&empty).unwrap();
        let candidates = vec![
            Candidate::container(root.join("missing"), "missing"),
            Candidate::container(empty, "empty"),
            Candidate::version_dir(root.join("not-a-version"), "no executable"),
        ];
        assert!(detect_from(candidates).is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn overlapping_candidates_are_reported_once() {
        let root = temp_dir();
        let common = root.join("common");
        let blender = make_version_dir(&common, "Blender");
        let with_trailing_separator =
            PathBuf::from(format!("{}{}", common.to_string_lossy(), std::path::MAIN_SEPARATOR));

        let found = detect_from(vec![
            Candidate::version_dir(blender, "Steam"),
            Candidate::container(with_trailing_separator, "again"),
        ]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].label, "Steam");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn steam_library_folders_are_read_from_vdf() {
        let vdf = r#""libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"label"		""
		"apps"
		{
			"228980"		"183291"
		}
	}
	"1"
	{
		"path"		"D:\\SteamLibrary"
		"label"		"games"
	}
}
"#;
        assert_eq!(
            parse_steam_library_folders(vdf),
            vec![PathBuf::from("C:\\Program Files (x86)\\Steam"), PathBuf::from("D:\\SteamLibrary")]
        );
        assert!(parse_steam_library_folders("").is_empty());
    }

    #[test]
    fn windows_paths_get_the_picker_spelling() {
        assert_eq!(native_windows_path("c:/program files (x86)/steam"), "C:\\program files (x86)\\steam");
        assert_eq!(native_windows_path("D:\\SteamLibrary"), "D:\\SteamLibrary");
        assert_eq!(native_windows_path("relative/dir"), "relative\\dir");
    }

    /// Prints what the sweep finds on this machine; for checking the platform
    /// candidates (and the Steam registry read) by hand:
    /// `cargo test --lib prints_detected_installations -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn prints_detected_installations() {
        for found in detect_blender_installations() {
            println!("{} ({}):", found.directory_path, found.label);
            for dir in &found.version_dirs {
                println!("    {}", dir);
            }
        }
    }

    #[test]
    fn same_location_path_ignores_trailing_separators() {
        assert!(same_location_path("/opt", "/opt/"));
        assert!(same_location_path(
            "C:\\Program Files\\Blender Foundation\\",
            "C:\\Program Files\\Blender Foundation"
        ));
        assert!(!same_location_path("/opt", "/opt/blender"));
        #[cfg(windows)]
        {
            assert!(same_location_path(
                "c:/program files/blender foundation",
                "C:\\Program Files\\Blender Foundation"
            ));
        }
    }
}
