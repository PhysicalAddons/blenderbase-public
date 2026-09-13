pub const FORWARD_SLASH_DELIMETER: &str = "/";
pub const PROJECTS_BLENDER_ORG_BLENDER_BLENDER_COMMIT: &str =
    "https://projects.blender.org/blender/blender/commit";
pub const PROJECTS_BLENDER_ORG_BLENDER_BLENDER_PULLS: &str =
    "https://projects.blender.org/blender/blender/pulls";
pub const PR: &str = "PR";
#[cfg(target_os = "windows")]
pub const BLENDER_USER_FILE_DIRECTORY: &str = "AppData\\Roaming\\Blender Foundation\\Blender";
#[cfg(target_os = "macos")]
pub const BLENDER_USER_FILE_DIRECTORY: &str = "Library/Application Support/Blender";
pub const BLENDER_FOUNDATION: &str = "Blender Foundation";
pub const BLENDER: &str = "Blender";
// pub const CONTENTS: &str = "Contents";
pub const BLENDER_EXE: &str = "blender.exe";
pub const BLENDER_LAUNCHER_EXE: &str = "blender-launcher.exe";
// pub const CONTENTS: &str = "Contents";
// Release Blender versions
//https://www.blender.org/about/website/
pub const RELEASE_BLENDER_URL_EU: &str = "https://ftp.nluug.nl/pub/graphics/blender/release/"; //This is a mirror for EU. This is the official link: "https://download.blender.org/release/"
// Daily Blender versions
// Patch Blender versions
#[cfg(target_os = "windows")]
pub const PLATFORM_SELECTOR: &str = "li.t-row.build.is-windows:not([style*='display:none;'])";
#[cfg(target_os = "macos")]
pub const PLATFORM_SELECTOR: &str =
    ".builds-list-container.platform-darwin li:not([style*='display:none;'])";
pub const A_SELECTOR: &str = "a.b-version, a.b-variant, a.b-reference";
pub const DIV_SELECTOR: &str = "div.b-date, div.b-arch";
// pub const ISO_FORMAT: &str = "%d-%b-%Y %H:%M";
pub const DOWNLOAD_BLENDER_ORG_RELEASE: &str = "https://ftp.nluug.nl/pub/graphics/blender/release/"; //"https://download.blender.org/release/";
/// Official Blender Foundation release host. Checksums are always fetched from
/// here, even when the archive itself came from a mirror, so a tampered mirror
/// cannot also supply a matching checksum.
pub const BLENDER_ORG_RELEASE_CHECKSUM_BASE: &str = "https://download.blender.org/release/";
pub const BUILDER_BLENDER_ORG_DOWNLOAD_DAILY_FORMAT_JSON_V2: &str =
    "https://builder.blender.org/download/daily/?format=json&v=2";
pub const BUILDER_BLENDER_ORG_DOWNLOAD_PATCH_FORMAT_JSON_V2: &str =
    "https://builder.blender.org/download/patch/?format=json&v=2";
pub const B3D_LINK_REGEX: &str = r#"Blender(\d+\.\d+)[^\d]"#;
pub const LTS: &str = "lts";
pub const STABLE: &str = "stable";
pub const BLENDER_DOWNLOAD_LINK_REGEX: &str = r#"href="([^"]*blender[^"]*)""#;
// // pub const PUBLISH_TIMESTAMP_REGEX: &str = r#"<a href="[^"]+">([^<]+)</a>\s+(\d+-[A-Za-z]+-\d+\s+\d+:\d+)\s+(\d+)"#;
pub const PUBLISH_TIMESTAMP_REGEX: &str = r#"<tr>\s*<td class="link"><a href="([^"]+)".*?>.*?</a></td>\s*<td class="size">([^<]+)</td>\s*<td class="date">(\d{4}-[A-Za-z]{3}-\d{2}\s+\d{2}:\d{2})</td>\s*</tr>"#;
pub const BLENDER_VERSION_REGEX: &str = r#"\d+\.\d+[a-z]*\d*(?:\.\d+)*"#;
// pub const A_SELECTOR: &str = "a.b-version, a.b-variant, a.b-reference";
// pub const DIV_SELECTOR: &str = "div.b-date, div.b-arch";
#[cfg(target_os = "windows")]
pub const FILE_REGEX_RELEASE: &str = r"(?i)blender-[\d.]+-windows-x64\.zip$";
#[cfg(target_os = "windows")]
pub const FILE_REGEX: &str = r"blender-.+win.+64.+zip$";
#[cfg(target_os = "macos")]
pub const FILE_REGEX: &str = r"blender-.+(macOS|macos|darwin).+(dmg|tar\.gz|\.zip)$";
#[cfg(target_os = "linux")]
pub const FILE_REGEX: &str = r"blender-.+linux.+(tar\.xz|tar\.bz2)$";
pub const LTS_VERSION_ARR: [&str; 6] = ["2.83", "2.93", "3.3", "3.6", "4.2", "4.5"];
// // pub const ISO_FORMAT: &str = "%d-%b-%Y %H:%M";
pub const ISO_FORMAT: &str = "%Y-%b-%d %H:%M";
pub const WINDOWS: &str = "windows";
pub const X64: &str = "x64";
pub const X32: &str = "x32";
pub const MACOS: &str = "macos";
pub const ARM64: &str = "arm64";
pub const APPLE_SILICON: &str = "Apple Silicon";
pub const INTEL: &str = "Intel";
pub const PUB_GRAPHICS_BLENDER_RELEASE: &str = "/pub/graphics/blender/release/";
pub const ALL: &str = "all";
pub const OFFICIAL: &str = "official";
pub const COMMUNITY: &str = "community";
