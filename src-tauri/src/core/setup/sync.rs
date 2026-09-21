use std::path::{Path, PathBuf};

use serde::Serialize;

use super::{bundle::read_bundle_manifest, manifest::SetupMeta};
use crate::database::SetupSync;

/// The one file a sync folder holds. Every computer saves to it and applies from it.
pub const SETUP_SYNC_FILE_NAME: &str = "Blender setup.bbsetup";

/// What the sync folder holds right now, next to what this computer last synced.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SetupSyncStatus {
    /// Empty when no folder is set.
    pub folder_path: String,
    pub last_synced_hash: String,
    pub last_synced_at: String,
    /// The setup file in the folder, when there is one that can be read.
    pub file: Option<SetupSyncFile>,
    /// A readable file whose content differs from what this computer last saved or applied.
    pub is_newer: bool,
    /// Why the file could not be read, when it exists but is not a usable setup.
    pub file_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetupSyncFile {
    pub file_path: String,
    pub content_hash: String,
    pub meta: SetupMeta,
    /// RFC 3339, from the file's modification time.
    pub modified_at: String,
    pub file_size: u64,
    pub blender_versions: usize,
    pub series: usize,
}

pub fn sync_file_path(folder_path: &str) -> PathBuf {
    Path::new(folder_path).join(SETUP_SYNC_FILE_NAME)
}

/// Reads the folder's setup file (manifest only, a few kilobytes) and compares it to the
/// last synced hash. Blocking file work; call from the blocking pool.
pub fn read_sync_status(sync: &SetupSync) -> SetupSyncStatus {
    let mut status = SetupSyncStatus {
        folder_path: sync.folder_path.clone(),
        last_synced_hash: sync.last_synced_hash.clone(),
        last_synced_at: sync.last_synced_at.clone(),
        ..SetupSyncStatus::default()
    };
    if sync.folder_path.trim().is_empty() {
        return status;
    }
    let path = sync_file_path(&sync.folder_path);
    if !path.is_file() {
        return status;
    }
    let metadata = match std::fs::metadata(&path) {
        Ok(v) => v,
        Err(e) => {
            status.file_error = Some(format!("Could not read {}: {}", path.display(), e));
            return status;
        }
    };
    let manifest = match read_bundle_manifest(&path) {
        Ok(v) => v,
        Err(e) => {
            status.file_error = Some(e);
            return status;
        }
    };
    let content_hash = manifest.content_hash().unwrap_or_default();
    let modified_at = metadata
        .modified()
        .ok()
        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .unwrap_or_default();
    status.is_newer = !content_hash.is_empty() && content_hash != sync.last_synced_hash;
    status.file = Some(SetupSyncFile {
        file_path: path.to_string_lossy().to_string(),
        content_hash,
        meta: manifest.meta.clone(),
        modified_at,
        file_size: metadata.len(),
        blender_versions: manifest.blender.len(),
        series: manifest.series.len(),
    });
    status
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::setup::{bundle::SetupBundleWriter, manifest::SetupManifest};

    #[test]
    fn a_folder_file_counts_as_newer_until_it_is_synced() {
        let dir = std::env::temp_dir().join(format!("blenderbase-sync-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut sync = SetupSync {
            folder_path: dir.to_string_lossy().to_string(),
            ..SetupSync::default()
        };
        let empty = read_sync_status(&sync);
        assert!(empty.file.is_none() && !empty.is_newer && empty.file_error.is_none());

        let manifest = SetupManifest::new(SetupMeta {
            device: String::from("Studio-PC"),
            ..SetupMeta::default()
        });
        let writer = SetupBundleWriter::create(&sync_file_path(&sync.folder_path)).unwrap();
        writer.finish(&manifest).unwrap();

        let fresh = read_sync_status(&sync);
        let file = fresh.file.as_ref().expect("the file is read");
        assert!(fresh.is_newer, "nothing was synced yet");
        assert_eq!(file.meta.device, "Studio-PC");
        assert_eq!(file.content_hash, manifest.content_hash().unwrap());

        sync.last_synced_hash = file.content_hash.clone();
        assert!(!read_sync_status(&sync).is_newer, "the same content is not news");

        std::fs::write(sync_file_path(&sync.folder_path), b"not a setup").unwrap();
        let broken = read_sync_status(&sync);
        assert!(broken.file.is_none() && broken.file_error.is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
