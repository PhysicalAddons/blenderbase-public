use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Layout version this build writes, and the newest it can read.
pub const SETUP_MANIFEST_SCHEMA: u32 = 1;
/// A blob reference is this prefix followed by 64 lowercase hex digits.
pub const SETUP_BLOB_PREFIX: &str = "sha256:";
/// Format tag of the preferences blob: JSON grouped by preference struct.
pub const SETUP_PREFERENCES_FORMAT: &str = "prefs-json-1";

pub const SETUP_ADDON_REASON_SYMLINK: &str = "symlink";
pub const SETUP_ADDON_REASON_SYSTEM_REPOSITORY: &str = "system-repository";
pub const SETUP_ADDON_REASON_FILES_NOT_INCLUDED: &str = "files-not-included";

/// A Blender setup with every machine-specific path left out, so it can be rebuilt on
/// another computer. Things are identified by natural keys (version string, extension id,
/// module name); local database ids never appear here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetupManifest {
    pub schema: u32,
    #[serde(default)]
    pub meta: SetupMeta,
    #[serde(default)]
    pub blender: Vec<SetupBlenderVersion>,
    /// One section per Blender series (`4.5`, `5.2`), because Blender keeps user files per series.
    #[serde(default)]
    pub series: BTreeMap<String, SetupSeries>,
}

/// Where and when the manifest was written. Not part of [`SetupManifest::content_hash`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SetupMeta {
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub app_version: String,
    #[serde(default)]
    pub platform: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetupBlenderVersion {
    pub version: String,
    pub series: String,
    /// lts / stable / candidate / beta / alpha.
    pub channel: String,
    /// Daily and patch builds expire upstream, so they are restored by branch, not by version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, rename = "default")]
    pub is_default: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SetupSeries {
    /// The Blender version that read this series' configuration.
    #[serde(default)]
    pub captured_with: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferences: Option<SetupBlob>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<SetupBlob>,
    /// Present only when the user changed keymaps; holds Blender's own keyconfig export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keymap: Option<SetupBlob>,
    /// Remote extension repositories. Access tokens are never part of a setup.
    #[serde(default)]
    pub repositories: Vec<SetupRepository>,
    #[serde(default)]
    pub addons: Vec<SetupAddon>,
}

/// A file of the setup, stored once under its SHA-256.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetupBlob {
    pub blob: String,
    pub size: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetupRepository {
    pub module: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub needs_token: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SetupAddonSource {
    /// Bundled with Blender; only the enabled state travels.
    Core,
    /// An extension of a remote repository, restored by id.
    Repo,
    /// Installed from a file. Restorable when `file` is present.
    File,
    /// Cannot be restored automatically; `reason` says why.
    Manual,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetupAddon {
    pub source: SetupAddonSource,
    pub module: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    /// `extension` or `addon`; absent for core addons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Module of the repository the extension belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// The extension id inside its repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Hash over the addon's files, independent of how they are archived.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<SetupBlob>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl SetupManifest {
    pub fn new(meta: SetupMeta) -> Self {
        Self {
            schema: SETUP_MANIFEST_SCHEMA,
            meta,
            blender: Vec::new(),
            series: BTreeMap::new(),
        }
    }

    /// SHA-256 over everything except `meta`, so two captures of an unchanged setup agree.
    // Change detection is not wired up yet; until then only the tests call this.
    #[allow(dead_code)]
    pub fn content_hash(&self) -> Result<String, String> {
        #[derive(Serialize)]
        struct Content<'a> {
            schema: u32,
            blender: &'a Vec<SetupBlenderVersion>,
            series: &'a BTreeMap<String, SetupSeries>,
        }
        let bytes = serde_json::to_vec(&Content {
            schema: self.schema,
            blender: &self.blender,
            series: &self.series,
        })
        .map_err(|e| format!("Could not hash the setup: {}", e))?;
        Ok(format!("{:x}", Sha256::digest(&bytes)))
    }

    /// Every blob the manifest refers to, without duplicates.
    pub fn blob_references(&self) -> Vec<&SetupBlob> {
        let mut found: Vec<&SetupBlob> = Vec::new();
        for section in self.series.values() {
            let files = [&section.preferences, &section.theme, &section.keymap]
                .into_iter()
                .flatten()
                .chain(section.addons.iter().filter_map(|a| a.file.as_ref()));
            for blob in files {
                if !found.iter().any(|b| b.blob == blob.blob) {
                    found.push(blob);
                }
            }
        }
        found
    }

    /// A manifest comes from a file someone else may have written: nothing in it is used for
    /// a path or handed to Blender before it passed this check.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema == 0 {
            return Err(String::from("The setup has no schema version"));
        }
        if self.schema > SETUP_MANIFEST_SCHEMA {
            return Err(format!(
                "The setup was written by a newer Blenderbase (schema {}, this build reads up to {}). Update Blenderbase to open it.",
                self.schema, SETUP_MANIFEST_SCHEMA
            ));
        }
        for version in &self.blender {
            if !is_series(&version.series) {
                return Err(format!("'{}' is not a Blender series", version.series));
            }
        }
        for (series, section) in &self.series {
            if !is_series(series) {
                return Err(format!("'{}' is not a Blender series", series));
            }
            for blob in [&section.preferences, &section.theme, &section.keymap]
                .into_iter()
                .flatten()
            {
                validate_blob(blob)?;
            }
            for repository in &section.repositories {
                if !is_identifier(&repository.module) {
                    return Err(format!(
                        "'{}' is not a repository identifier",
                        repository.module
                    ));
                }
            }
            for addon in &section.addons {
                if addon.module.trim().is_empty() {
                    return Err(String::from("The setup lists an addon without a module name"));
                }
                if let Some(file) = &addon.file {
                    validate_blob(file)?;
                    match &file.name {
                        Some(name) if is_plain_file_name(name) => {}
                        _ => {
                            return Err(format!(
                                "The file of addon '{}' has no usable name",
                                addon.module
                            ))
                        }
                    }
                }
                if addon.source == SetupAddonSource::Repo {
                    let named = addon.repository.as_deref().map(is_identifier).unwrap_or(false)
                        && addon.package.as_deref().map(is_identifier).unwrap_or(false);
                    if !named {
                        return Err(format!(
                            "Extension '{}' does not name its repository and id",
                            addon.module
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

/// The hex digest of a well-formed blob reference.
pub fn blob_digest(reference: &str) -> Option<&str> {
    let digest = reference.strip_prefix(SETUP_BLOB_PREFIX)?;
    let well_formed = digest.len() == 64
        && digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    well_formed.then_some(digest)
}

pub fn blob_reference(digest: &str) -> String {
    format!("{}{}", SETUP_BLOB_PREFIX, digest.to_ascii_lowercase())
}

fn validate_blob(blob: &SetupBlob) -> Result<(), String> {
    match blob_digest(&blob.blob) {
        Some(_) => Ok(()),
        None => Err(format!("'{}' is not a blob reference", blob.blob)),
    }
}

/// `4.5`, `5.2`: the name of Blender's per-series user folder, so digits and one dot only.
pub fn is_series(value: &str) -> bool {
    let mut parts = value.split('.');
    let numeric = |p: Option<&str>| {
        p.map(|s| !s.is_empty() && s.len() <= 3 && s.bytes().all(|b| b.is_ascii_digit()))
            .unwrap_or(false)
    };
    numeric(parts.next()) && numeric(parts.next()) && parts.next().is_none()
}

/// Repository modules and extension ids are Python identifiers.
pub fn is_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !value.starts_with(|c: char| c.is_ascii_digit())
}

/// A name that stays inside the folder it is written to: no separators, no `..`, no drive.
pub fn is_plain_file_name(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && value.len() <= 255
        && value != "."
        && value != ".."
        && !value.contains(['/', '\\', ':', '\0'])
        && !value.chars().any(|c| c.is_control())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(fill: char) -> String {
        blob_reference(&fill.to_string().repeat(64))
    }

    fn sample() -> SetupManifest {
        let mut manifest = SetupManifest::new(SetupMeta {
            created: String::from("2026-09-17T19:00:00Z"),
            app_version: String::from("1.2.8"),
            platform: String::from("windows-x86_64"),
        });
        manifest.blender.push(SetupBlenderVersion {
            version: String::from("5.2.1"),
            series: String::from("5.2"),
            channel: String::from("lts"),
            branch: None,
            is_default: true,
            custom_name: None,
        });
        manifest.series.insert(
            String::from("5.2"),
            SetupSeries {
                captured_with: String::from("5.2.1"),
                preferences: Some(SetupBlob {
                    blob: digest('a'),
                    size: 10,
                    name: None,
                    format: Some(String::from(SETUP_PREFERENCES_FORMAT)),
                }),
                theme: Some(SetupBlob {
                    blob: digest('b'),
                    size: 20,
                    name: Some(String::from("Graphite")),
                    format: None,
                }),
                keymap: None,
                repositories: vec![SetupRepository {
                    module: String::from("blender_org"),
                    name: String::from("extensions.blender.org"),
                    url: String::from("https://extensions.blender.org/api/v1/extensions/"),
                    needs_token: false,
                }],
                addons: vec![
                    SetupAddon {
                        source: SetupAddonSource::Repo,
                        module: String::from("bl_ext.blender_org.example_tools"),
                        name: String::from("Example Tools"),
                        version: Some(String::from("2.1.0")),
                        enabled: true,
                        kind: Some(String::from("extension")),
                        repository: Some(String::from("blender_org")),
                        package: Some(String::from("example_tools")),
                        content_hash: None,
                        file: None,
                        reason: None,
                    },
                    SetupAddon {
                        source: SetupAddonSource::File,
                        module: String::from("example_paid_addon"),
                        name: String::from("Example Paid Addon"),
                        version: Some(String::from("1.4.2")),
                        enabled: true,
                        kind: Some(String::from("addon")),
                        repository: None,
                        package: None,
                        content_hash: Some("c".repeat(64)),
                        file: Some(SetupBlob {
                            blob: digest('b'),
                            size: 20,
                            name: Some(String::from("example_paid_addon.zip")),
                            format: None,
                        }),
                        reason: None,
                    },
                ],
            },
        );
        manifest
    }

    #[test]
    fn manifest_survives_a_round_trip() {
        let manifest = sample();
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        assert!(json.contains("\"default\": true"));
        assert!(json.contains("\"source\": \"repo\""));
        let back: SetupManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, manifest);
        back.validate().unwrap();
    }

    #[test]
    fn content_hash_ignores_meta_and_follows_content() {
        let first = sample();
        let mut later = sample();
        later.meta.created = String::from("2027-01-01T00:00:00Z");
        assert_eq!(first.content_hash().unwrap(), later.content_hash().unwrap());
        later.series.get_mut("5.2").unwrap().addons[0].enabled = false;
        assert_ne!(first.content_hash().unwrap(), later.content_hash().unwrap());
    }

    #[test]
    fn blob_references_are_listed_once() {
        let manifest = sample();
        let blobs = manifest.blob_references();
        assert_eq!(blobs.len(), 2, "the theme and the addon share one blob");
    }

    #[test]
    fn unknown_fields_are_ignored_but_a_newer_schema_is_refused() {
        let json = r#"{"schema": 1, "future": {"x": 1}, "series": {"5.2": {"captured_with": "5.2.1", "later": true}}}"#;
        let manifest: SetupManifest = serde_json::from_str(json).unwrap();
        manifest.validate().unwrap();

        let newer: SetupManifest = serde_json::from_str(r#"{"schema": 2}"#).unwrap();
        let refusal = newer.validate().unwrap_err();
        assert!(refusal.contains("newer Blenderbase"), "{}", refusal);
    }

    #[test]
    fn hostile_names_are_refused() {
        let mut escaping_series = sample();
        let section = escaping_series.series.remove("5.2").unwrap();
        escaping_series.series.insert(String::from("../5.2"), section);
        assert!(escaping_series.validate().is_err());

        let mut escaping_file = sample();
        let file = escaping_file.series.get_mut("5.2").unwrap().addons[1]
            .file
            .as_mut()
            .unwrap();
        file.name = Some(String::from("..\\..\\startup.py"));
        assert!(escaping_file.validate().is_err());

        let mut bad_blob = sample();
        bad_blob
            .series
            .get_mut("5.2")
            .unwrap()
            .theme
            .as_mut()
            .unwrap()
            .blob = String::from("sha256:../../etc/passwd");
        assert!(bad_blob.validate().is_err());

        let mut bad_package = sample();
        bad_package.series.get_mut("5.2").unwrap().addons[0].package =
            Some(String::from("x, os.system('calc')"));
        assert!(bad_package.validate().is_err());
    }

    #[test]
    fn name_checks() {
        assert!(is_series("4.5") && is_series("10.12"));
        assert!(!is_series("4") && !is_series("4.5.1") && !is_series("4.x") && !is_series(""));
        assert!(is_identifier("blender_org") && !is_identifier("1abc") && !is_identifier("a-b"));
        assert!(is_plain_file_name("addon.zip") && is_plain_file_name("physical_atmosphere².zip"));
        assert!(!is_plain_file_name("a/b.zip") && !is_plain_file_name("C:x") && !is_plain_file_name(" x"));
        assert_eq!(blob_digest(&digest('a')), Some("a".repeat(64).as_str()));
        assert_eq!(blob_digest("sha256:ABC"), None);
    }
}
