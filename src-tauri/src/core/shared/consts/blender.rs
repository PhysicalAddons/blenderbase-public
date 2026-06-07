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
pub const RECENT_FILES_TXT: &str = "recent-files.txt";
pub const CONFIG: &str = "config";
pub const DATE_FORMAT: &str = "%d.%m.%Y %H:%M";
pub const BLENDER_FOUNDATION: &str = "Blender Foundation";
pub const BLENDER: &str = "Blender";
// pub const CONTENTS: &str = "Contents";
pub const MAC_OS: &str = "MacOS";
pub const BLENDER_EXE: &str = "blender.exe";
pub const BLENDER_LAUNCHER_EXE: &str = "blender-launcher.exe";
pub const DS_STORE: &str = ".DS_Store";
pub const ZIP: &str = ".zip";
pub const ACTIVE_DOWNLOADABLE_BLENDER_FILTER: &str = "active_downloadable_blender_filter.json";
pub const RELEASE_BLENDER: &str = "release_blenders.json";
pub const DAILY_BLENDERS: &str = "daily_blenders.json";
pub const PATCH_BLENDERS: &str = "patch_blenders.json";
pub const DOWNLOAD_DATA: &str = "download_data.json";
// pub const CONTENTS: &str = "Contents";
// Release Blender versions
//https://www.blender.org/about/website/
pub const RELEASE_BLENDER_URL_EU: &str = "https://ftp.nluug.nl/pub/graphics/blender/release/"; //This is a mirror for EU. This is the official link: "https://download.blender.org/release/"
pub const RELEASE_BLENDER_URL_US: &str = "https://mirror.clarkson.edu/blender/";
pub const RELEASE_BLENDER_URL_CN: &str = "https://mirrors.aliyun.com/blender/";
// Daily Blender versions
pub const DAILY_BLENDERS_URL: &str = "https://builder.blender.org/download/daily/";
// Patch Blender versions
pub const PATCH_BLENDERS_URL: &str = "https://builder.blender.org/download/patch/";
#[cfg(target_os = "windows")]
pub const PLATFORM_SELECTOR: &str = "li.t-row.build.is-windows:not([style*='display:none;'])";
#[cfg(target_os = "macos")]
pub const PLATFORM_SELECTOR: &str =
    ".builds-list-container.platform-darwin li:not([style*='display:none;'])";
pub const SUBVERSION_REGEX: &str = r#"\d+\.\d+"#;
pub const A_SELECTOR: &str = "a.b-version, a.b-variant, a.b-reference";
pub const DIV_SELECTOR: &str = "div.b-date, div.b-arch";
// pub const ISO_FORMAT: &str = "%d-%b-%Y %H:%M";
pub const DOWNLOAD_BLENDER_ORG_RELEASE: &str = "https://ftp.nluug.nl/pub/graphics/blender/release/"; //"https://download.blender.org/release/";
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
pub const RELEASE: &str = "release";
pub const RELEASE_SENTANCE_CASE: &str = "Release";
pub const DAILY: &str = "daily";
pub const DAILY_SENTANCE_CASE: &str = "Daily";
pub const PATCH: &str = "community";
pub const PATCH_SENTANCE_CASE: &str = "Community";
pub const ACTIVE_ADDON_FILTER: &str = "active_addon_filter.json";
pub const ALL: &str = "all";
pub const ALL_UPPERCASE: &str = "All";
pub const OFFICIAL: &str = "official";
pub const OFFICIAL_UPPERCASE: &str = "Official";
pub const COMMUNITY: &str = "community";
pub const COMMUNITY_UPPERCASE: &str = "Community";
pub const INIT_PY: &str = "__init__.py";

// #[cfg(target_os = "windows")]
// // pub const PATH_TO_DATABASE_DIRECTORY: &str = "AppData\\Local\\com.physicaladdons.blenderbase\\db\\installed_blender_versions";

// #[cfg(target_os = "macos")]
// // pub const PATH_TO_DATABASE_DIRECTORY: &str = "Library/Application Support/com.physicaladdons.blenderbase/db/installed_blender_versions";

// // pub const CONTENTS: &str = "Contents";

// pub const MACOS: &str = "MacOS";

// pub const BLENDER_EXE: &str = "blender.exe";
// pub const BLENDER_LAUNCHER_EXE: &str = "blender-launcher.exe";

// pub const DS_STORE: &str = ".DS_Store";
// pub const ZIP: &str = ".zip";
// pub const BLENDER: &str = "Blender";

// #[cfg(target_os = "windows")]
// // pub const PATH_TO_DATABASE_DIRECTORY: &str = "AppData\\Local\\com.physicaladdons.blenderbase\\db\\downloadable_blender_versions";

// #[cfg(target_os = "macos")]
// // pub const PATH_TO_DATABASE_DIRECTORY: &str = "Library/Application Support/com.physicaladdons.blenderbase/db/downloadable_blender_versions";

// pub const ACTIVE_DOWNLOADABLE_BLENDER_FILTER: &str = "active_downloadable_blender_filter.json";
// pub const RELEASE_BLENDER: &str = "release_blenders.json";
// pub const DAILY_BLENDERS: &str = "daily_blenders.json";
// pub const PATCH_BLENDERS: &str = "patch_blenders.json";
// pub const DOWNLOAD_DATA: &str = "download_data.json";
// // pub const CONTENTS: &str = "Contents";

// // Release Blender versions
// //https://www.blender.org/about/website/
// pub const RELEASE_BLENDER_URL_EU: &str = "https://ftp.nluug.nl/pub/graphics/blender/release/"; //This is a mirror for EU. This is the official link: "https://download.blender.org/release/"
// pub const RELEASE_BLENDER_URL_US: &str = "https://mirror.clarkson.edu/blender/";
// pub const RELEASE_BLENDER_URL_CN: &str = "https://mirrors.aliyun.com/blender/";
// // Daily Blender versions
// pub const DAILY_BLENDERS_URL: &str = "https://builder.blender.org/download/daily/";
// // Patch Blender versions
// pub const PATCH_BLENDERS_URL: &str = "https://builder.blender.org/download/patch/";

// #[cfg(target_os = "windows")]
// pub const PLATFORM_SELECTOR: &str = "li.t-row.build.is-windows:not([style*='display:none;'])";

// #[cfg(target_os = "macos")]
// pub const PLATFORM_SELECTOR: &str = ".builds-list-container.platform-darwin li:not([style*='display:none;'])";

// pub const B3D_LINK_REGEX: &str = r#"Blender(\d+\.\d+)[^\d]"#;
// pub const SUBVERSION_REGEX: &str = r#"\d+\.\d+"#;
// // pub const BLENDER: &str = "Blender";

// pub const DOWNLOAD_BLENDER_ORG_RELEASE: &str = "https://ftp.nluug.nl/pub/graphics/blender/release/"; //"https://download.blender.org/release/";
// pub const BUILDER_BLENDER_ORG_DOWNLOAD_DAILY_FORMAT_JSON_V2: &str =
//     "https://builder.blender.org/download/daily/?format=json&v=2";
// pub const BUILDER_BLENDER_ORG_DOWNLOAD_PATCH_FORMAT_JSON_V2: &str =
//     "https://builder.blender.org/download/patch/?format=json&v=2";
// pub const B3D_LINK_REGEX: &str = r#"Blender(\d+\.\d+)[^\d]"#;
// pub const BLENDER: &str = "Blender";
// pub const LTS: &str = "lts";
// pub const STABLE: &str = "stable";

// pub const BLENDER_DOWNLOAD_LINK_REGEX: &str = r#"href="([^"]*blender[^"]*)""#;
// // // pub const PUBLISH_TIMESTAMP_REGEX: &str = r#"<a href="[^"]+">([^<]+)</a>\s+(\d+-[A-Za-z]+-\d+\s+\d+:\d+)\s+(\d+)"#;
// pub const PUBLISH_TIMESTAMP_REGEX: &str = r#"<tr>\s*<td class="link"><a href="([^"]+)".*?>.*?</a></td>\s*<td class="size">([^<]+)</td>\s*<td class="date">(\d{4}-[A-Za-z]{3}-\d{2}\s+\d{2}:\d{2})</td>\s*</tr>"#;
// pub const BLENDER_VERSION_REGEX: &str = r#"\d+\.\d+[a-z]*\d*(?:\.\d+)*"#;

// // pub const A_SELECTOR: &str = "a.b-version, a.b-variant, a.b-reference";
// // pub const DIV_SELECTOR: &str = "div.b-date, div.b-arch";

// #[cfg(target_os = "windows")]
// pub const FILE_REGEX_RELEASE: &str = r"(?i)blender-[\d.]+-windows-x64\.zip$";

// #[cfg(target_os = "windows")]
// pub const FILE_REGEX: &str = r"blender-.+win.+64.+zip$";

// #[cfg(target_os = "macos")]
// pub const FILE_REGEX: &str = r"blender-.+(macOS|macos|darwin).+(dmg|tar\.gz|\.zip)$";

// #[cfg(target_os = "linux")]
// pub const FILE_REGEX: &str = r"blender-.+linux.+(tar\.xz|tar\.bz2)$";

// pub const LTS_VERSION_ARR: [&str; 6] = ["2.83", "2.93", "3.3", "3.6", "4.2", "4.5"];

// // // pub const ISO_FORMAT: &str = "%d-%b-%Y %H:%M";
// pub const ISO_FORMAT: &str = "%Y-%b-%d %H:%M";

// pub const WINDOWS: &str = "windows";
// pub const X64: &str = "x64";
// pub const X32: &str = "x32";
// pub const MACOS: &str = "macos";
// pub const ARM64: &str = "arm64";
// pub const APPLE_SILICON: &str = "Apple Silicon";
// pub const INTEL: &str = "Intel";
// pub const PUB_GRAPHICS_BLENDER_RELEASE: &str = "/pub/graphics/blender/release/";

// pub const RELEASE: &str = "release";
// pub const RELEASE_SENTANCE_CASE: &str = "Release";

// pub const DAILY: &str = "daily";
// pub const DAILY_SENTANCE_CASE: &str = "Daily";

// pub const PATCH: &str = "community";
// pub const PATCH_SENTANCE_CASE: &str = "Community";

// #[cfg(target_os = "windows")]
// // pub const PATH_TO_DATABASE_DIRECTORY: &str = "AppData\\Local\\com.physicaladdons.blenderbase\\db\\installed_blender_versions";
// #[cfg(target_os = "macos")]
// // pub const PATH_TO_DATABASE_DIRECTORY: &str = "Library/Application Support/com.physicaladdons.blenderbase/db/installed_blender_versions";

// // pub const CONTENTS: &str = "Contents";

// pub const MACOS: &str = "MacOS";

// pub const BLENDER_EXE: &str = "blender.exe";
// pub const BLENDER_LAUNCHER_EXE: &str = "blender-launcher.exe";

// pub const DS_STORE: &str = ".DS_Store";
// pub const ZIP: &str = ".zip";

// #[cfg(target_os = "windows")]
// // pub const PATH_TO_DATABASE_DIRECTORY: &str = "AppData\\Local\\com.physicaladdons.blenderbase\\db\\downloadable_blender_versions";
// #[cfg(target_os = "macos")]
// // pub const PATH_TO_DATABASE_DIRECTORY: &str = "Library/Application Support/com.physicaladdons.blenderbase/db/downloadable_blender_versions";

// pub const ACTIVE_DOWNLOADABLE_BLENDER_FILTER: &str = "active_downloadable_blender_filter.json";
// pub const RELEASE_BLENDER: &str = "release_blenders.json";
// pub const DAILY_BLENDERS: &str = "daily_blenders.json";
// pub const PATCH_BLENDERS: &str = "patch_blenders.json";
// pub const DOWNLOAD_DATA: &str = "download_data.json";
// // pub const CONTENTS: &str = "Contents";

// // Release Blender versions
// //https://www.blender.org/about/website/
// pub const RELEASE_BLENDER_URL_EU: &str = "https://ftp.nluug.nl/pub/graphics/blender/release/"; //This is a mirror for EU. This is the official link: "https://download.blender.org/release/"
// pub const RELEASE_BLENDER_URL_US: &str = "https://mirror.clarkson.edu/blender/";
// pub const RELEASE_BLENDER_URL_CN: &str = "https://mirrors.aliyun.com/blender/";
// // Daily Blender versions
// pub const DAILY_BLENDERS_URL: &str = "https://builder.blender.org/download/daily/";
// // Patch Blender versions
// pub const PATCH_BLENDERS_URL: &str = "https://builder.blender.org/download/patch/";

// #[cfg(target_os = "windows")]
// pub const PLATFORM_SELECTOR: &str = "li.t-row.build.is-windows:not([style*='display:none;'])";

// #[cfg(target_os = "macos")]
// pub const PLATFORM_SELECTOR: &str =
//     ".builds-list-container.platform-darwin li:not([style*='display:none;'])";

// pub const SUBVERSION_REGEX: &str = r#"\d+\.\d+"#;

// pub const A_SELECTOR: &str = "a.b-version, a.b-variant, a.b-reference";
// pub const DIV_SELECTOR: &str = "div.b-date, div.b-arch";

// // pub const ISO_FORMAT: &str = "%d-%b-%Y %H:%M";
