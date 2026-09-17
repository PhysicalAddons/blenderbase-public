use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

use super::bundle::collect_files;

/// What a setup can overwrite inside a series folder, and therefore what a backup holds.
const BACKED_UP_DIRECTORIES: [&str; 2] = ["config", "scripts/presets"];
/// Backups kept per series; older ones are removed when a new one is written.
const BACKUPS_KEPT: usize = 3;
const BACKUP_EXTENSION: &str = "zip";

/// Zips the parts of a series folder a setup may change into `backups_directory` and returns
/// the archive. Nothing is applied before this succeeded, so every apply can be undone.
pub fn backup_series(series_directory: &Path, backups_directory: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(backups_directory)
        .map_err(|e| format!("Could not create {}: {}", backups_directory.display(), e))?;
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S%.3f").to_string();
    let archive_path = backups_directory.join(format!("{}.{}", stamp, BACKUP_EXTENSION));
    let file = std::fs::File::create(&archive_path)
        .map_err(|e| format!("Could not create {}: {}", archive_path.display(), e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(1));
    for directory in BACKED_UP_DIRECTORIES {
        let source = series_directory.join(directory);
        if !source.is_dir() {
            continue;
        }
        let mut files: Vec<(String, PathBuf)> = Vec::new();
        collect_files(&source, directory, &mut files)?;
        files.sort_by(|a, b| a.0.cmp(&b.0));
        for (relative, path) in files {
            zip.start_file(relative, options.large_file(true))
                .map_err(|e| format!("Could not back up {}: {}", path.display(), e))?;
            let mut source_file =
                std::fs::File::open(&path).map_err(|e| format!("Could not back up {}: {}", path.display(), e))?;
            std::io::copy(&mut source_file, &mut zip)
                .map_err(|e| format!("Could not back up {}: {}", path.display(), e))?;
        }
    }
    zip.finish()
        .map_err(|e| format!("Could not write {}: {}", archive_path.display(), e))?;
    prune_backups(backups_directory);
    Ok(archive_path)
}

/// Backups of one series, newest first. The file names are timestamps, so the name orders them.
pub fn list_backups(backups_directory: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(backups_directory)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.is_file()
                        && p.extension()
                            .map(|e| e.to_string_lossy().eq_ignore_ascii_case(BACKUP_EXTENSION))
                            .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default();
    found.sort();
    found.reverse();
    found
}

fn prune_backups(backups_directory: &Path) {
    for old in list_backups(backups_directory).into_iter().skip(BACKUPS_KEPT) {
        let _ = std::fs::remove_file(old);
    }
}

/// Puts a backup back: the backed-up folders are emptied and refilled from the archive, so
/// presets a setup added disappear again as well. Entries outside those folders are ignored.
pub fn restore_backup(archive_path: &Path, series_directory: &Path) -> Result<usize, String> {
    let file = std::fs::File::open(archive_path)
        .map_err(|e| format!("Could not open {}: {}", archive_path.display(), e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Could not read {}: {}", archive_path.display(), e))?;
    // Everything is read before anything is removed: a damaged backup must not cost the
    // configuration that is there now.
    let mut entries: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("Could not read {}: {}", archive_path.display(), e))?;
        if entry.is_dir() {
            continue;
        }
        let Some(relative) = entry.enclosed_name() else {
            continue;
        };
        let inside = BACKED_UP_DIRECTORIES
            .iter()
            .any(|d| relative.starts_with(Path::new(d)));
        if !inside {
            continue;
        }
        let mut content = Vec::new();
        entry
            .read_to_end(&mut content)
            .map_err(|e| format!("Could not read {}: {}", archive_path.display(), e))?;
        entries.push((relative, content));
    }
    for directory in BACKED_UP_DIRECTORIES {
        let target = series_directory.join(directory);
        if target.is_dir() {
            std::fs::remove_dir_all(&target).map_err(|e| format!("Could not clear {}: {}", target.display(), e))?;
        }
    }
    for (relative, content) in &entries {
        let target = series_directory.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Could not create {}: {}", parent.display(), e))?;
        }
        let mut output =
            std::fs::File::create(&target).map_err(|e| format!("Could not write {}: {}", target.display(), e))?;
        output
            .write_all(content)
            .map_err(|e| format!("Could not write {}: {}", target.display(), e))?;
    }
    Ok(entries.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("blenderbase-backup-{}-{}", label, uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_backup_restores_what_was_there_and_removes_what_was_added() {
        let dir = temp_dir("restore");
        let series = dir.join("5.2");
        std::fs::create_dir_all(series.join("config")).unwrap();
        std::fs::create_dir_all(series.join("scripts").join("presets").join("keyconfig")).unwrap();
        std::fs::create_dir_all(series.join("scripts").join("addons").join("kept")).unwrap();
        std::fs::write(series.join("config").join("userpref.blend"), "before").unwrap();
        std::fs::write(series.join("scripts").join("presets").join("keyconfig").join("mine.py"), "mine").unwrap();
        std::fs::write(series.join("scripts").join("addons").join("kept").join("__init__.py"), "addon").unwrap();

        let backup = backup_series(&series, &dir.join("backups")).unwrap();

        std::fs::write(series.join("config").join("userpref.blend"), "after").unwrap();
        std::fs::write(series.join("scripts").join("presets").join("keyconfig").join("added.py"), "added").unwrap();
        let restored = restore_backup(&backup, &series).unwrap();

        assert_eq!(restored, 2);
        assert_eq!(std::fs::read_to_string(series.join("config").join("userpref.blend")).unwrap(), "before");
        assert!(series.join("scripts").join("presets").join("keyconfig").join("mine.py").is_file());
        assert!(!series.join("scripts").join("presets").join("keyconfig").join("added.py").exists());
        assert!(
            series.join("scripts").join("addons").join("kept").join("__init__.py").is_file(),
            "addons are not part of a backup and are left alone"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_the_newest_backups_are_kept() {
        let dir = temp_dir("prune");
        let series = dir.join("5.2");
        std::fs::create_dir_all(series.join("config")).unwrap();
        std::fs::write(series.join("config").join("userpref.blend"), "x").unwrap();
        let mut written = Vec::new();
        for _ in 0..BACKUPS_KEPT + 2 {
            written.push(backup_series(&series, &dir.join("backups")).unwrap());
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let kept = list_backups(&dir.join("backups"));
        assert_eq!(kept.len(), BACKUPS_KEPT);
        assert_eq!(kept[0], *written.last().unwrap(), "newest first");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_backup_cannot_write_outside_the_folders_it_covers() {
        let dir = temp_dir("escape");
        let series = dir.join("5.2");
        std::fs::create_dir_all(series.join("config")).unwrap();
        let archive_path = dir.join("hostile.zip");
        {
            let mut zip = zip::ZipWriter::new(std::fs::File::create(&archive_path).unwrap());
            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("config/userpref.blend", options).unwrap();
            zip.write_all(b"ok").unwrap();
            zip.start_file("scripts/startup/run_me.py", options).unwrap();
            zip.write_all(b"import os").unwrap();
            zip.start_file("../outside.txt", options).unwrap();
            zip.write_all(b"escape").unwrap();
            zip.finish().unwrap();
        }
        assert_eq!(restore_backup(&archive_path, &series).unwrap(), 1);
        assert!(!series.join("scripts").join("startup").exists());
        assert!(!dir.join("outside.txt").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
