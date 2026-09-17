use std::{
    collections::HashSet,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use super::manifest::{blob_digest, blob_reference, SetupManifest};

/// File extension of a setup bundle: a zip holding `manifest.json` and `blobs/<sha256>`.
pub const SETUP_BUNDLE_EXTENSION: &str = "bbsetup";
const MANIFEST_ENTRY: &str = "manifest.json";
const BLOB_DIRECTORY: &str = "blobs/";
/// A manifest is a few kilobytes; anything near this size is not one.
const MANIFEST_SIZE_LIMIT: u64 = 16 * 1024 * 1024;

/// Folders and files of an installed addon that are rebuilt on the target machine.
const SKIPPED_DIRECTORIES: [&str; 3] = ["__pycache__", ".git", ".vs"];
const SKIPPED_EXTENSIONS: [&str; 2] = ["pyc", "pyo"];
/// Formats that are compressed already: deflating them again costs time and saves nothing.
const STORED_EXTENSIONS: [&str; 16] = [
    "png", "jpg", "jpeg", "webp", "gif", "exr", "mp4", "mov", "mp3", "ogg", "zip", "7z", "gz", "xz", "zst", "whl",
];

/// Writes a bundle next to its final name and moves it into place on `finish`, so a failed
/// export never leaves a half-written file behind.
pub struct SetupBundleWriter {
    zip: zip::ZipWriter<std::fs::File>,
    partial_path: PathBuf,
    final_path: PathBuf,
    stored: HashSet<String>,
}

impl SetupBundleWriter {
    pub fn create(file_path: &Path) -> Result<Self, String> {
        let partial_path = file_path.with_extension(format!("{}.part", SETUP_BUNDLE_EXTENSION));
        let file = std::fs::File::create(&partial_path)
            .map_err(|e| format!("Could not create {}: {}", partial_path.display(), e))?;
        Ok(Self {
            zip: zip::ZipWriter::new(file),
            partial_path,
            final_path: file_path.to_path_buf(),
            stored: HashSet::new(),
        })
    }

    /// Stores a file under its SHA-256 and returns the blob reference and size. A file the
    /// bundle already holds is not stored twice.
    pub fn add_blob_from_file(&mut self, source: &Path) -> Result<(String, u64), String> {
        let digest = sha256_of_path(source)?;
        let size = std::fs::metadata(source)
            .map_err(|e| format!("Could not read {}: {}", source.display(), e))?
            .len();
        if self.stored.insert(digest.clone()) {
            let is_archive = is_stored_extension(source);
            let mut file = std::fs::File::open(source)
                .map_err(|e| format!("Could not read {}: {}", source.display(), e))?;
            self.zip
                .start_file(format!("{}{}", BLOB_DIRECTORY, digest), entry_options(is_archive, size))
                .map_err(|e| format!("Could not write the setup file: {}", e))?;
            std::io::copy(&mut file, &mut self.zip)
                .map_err(|e| format!("Could not write the setup file: {}", e))?;
        }
        Ok((blob_reference(&digest), size))
    }

    pub fn add_blob_from_bytes(&mut self, bytes: &[u8]) -> Result<(String, u64), String> {
        let digest = format!("{:x}", Sha256::digest(bytes));
        let size = bytes.len() as u64;
        if self.stored.insert(digest.clone()) {
            self.zip
                .start_file(format!("{}{}", BLOB_DIRECTORY, digest), entry_options(false, size))
                .map_err(|e| format!("Could not write the setup file: {}", e))?;
            self.zip
                .write_all(bytes)
                .map_err(|e| format!("Could not write the setup file: {}", e))?;
        }
        Ok((blob_reference(&digest), size))
    }

    /// Writes the manifest, closes the archive and moves it to its final name.
    pub fn finish(mut self, manifest: &SetupManifest) -> Result<u64, String> {
        for blob in manifest.blob_references() {
            let held = blob_digest(&blob.blob)
                .map(|d| self.stored.contains(d))
                .unwrap_or(false);
            if !held {
                return Err(format!("The setup refers to {} but does not hold it", blob.blob));
            }
        }
        let json = serde_json::to_vec_pretty(manifest)
            .map_err(|e| format!("Could not write the setup manifest: {}", e))?;
        self.zip
            .start_file(MANIFEST_ENTRY, entry_options(false, json.len() as u64))
            .map_err(|e| format!("Could not write the setup file: {}", e))?;
        self.zip
            .write_all(&json)
            .map_err(|e| format!("Could not write the setup file: {}", e))?;
        self.zip
            .finish()
            .map_err(|e| format!("Could not write the setup file: {}", e))?;
        if self.final_path.exists() {
            std::fs::remove_file(&self.final_path)
                .map_err(|e| format!("Could not replace {}: {}", self.final_path.display(), e))?;
        }
        std::fs::rename(&self.partial_path, &self.final_path)
            .map_err(|e| format!("Could not write {}: {}", self.final_path.display(), e))?;
        std::fs::metadata(&self.final_path)
            .map(|m| m.len())
            .map_err(|e| format!("Could not read {}: {}", self.final_path.display(), e))
    }

    /// Removes the partial file of an export that did not finish.
    pub fn discard(self) {
        let partial_path = self.partial_path.clone();
        drop(self.zip);
        let _ = std::fs::remove_file(partial_path);
    }
}

/// Reads and validates the manifest of a bundle, and checks that every blob it names is there.
pub fn read_bundle_manifest(file_path: &Path) -> Result<SetupManifest, String> {
    let mut archive = open_bundle(file_path)?;
    let manifest: SetupManifest = {
        let entry = archive
            .by_name(MANIFEST_ENTRY)
            .map_err(|_| String::from("This is not a Blenderbase setup file: it has no manifest"))?;
        if entry.size() > MANIFEST_SIZE_LIMIT {
            return Err(String::from("This is not a Blenderbase setup file: the manifest is too large"));
        }
        let mut json = Vec::new();
        entry
            .take(MANIFEST_SIZE_LIMIT)
            .read_to_end(&mut json)
            .map_err(|e| format!("Could not read the setup manifest: {}", e))?;
        serde_json::from_slice(&json).map_err(|e| format!("Could not read the setup manifest: {}", e))?
    };
    manifest.validate()?;
    for blob in manifest.blob_references() {
        let digest = blob_digest(&blob.blob).unwrap_or_default();
        let entry = archive
            .by_name(&format!("{}{}", BLOB_DIRECTORY, digest))
            .map_err(|_| format!("The setup file is incomplete: {} is missing", blob.blob))?;
        if entry.size() != blob.size {
            return Err(format!("The setup file is damaged: {} has the wrong size", blob.blob));
        }
    }
    Ok(manifest)
}

/// Copies one blob out of a bundle. The content is hashed on the way and the copy is removed
/// again when it does not match its name, so nothing unverified is left for Blender to load.
// Applying a setup is the next step to land; until then only the tests call this.
#[allow(dead_code)]
pub fn extract_bundle_blob(file_path: &Path, reference: &str, destination: &Path) -> Result<(), String> {
    let digest = blob_digest(reference).ok_or_else(|| format!("'{}' is not a blob reference", reference))?;
    let mut archive = open_bundle(file_path)?;
    let mut entry = archive
        .by_name(&format!("{}{}", BLOB_DIRECTORY, digest))
        .map_err(|_| format!("The setup file is incomplete: {} is missing", reference))?;
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Could not create {}: {}", parent.display(), e))?;
    }
    let mut output = std::fs::File::create(destination)
        .map_err(|e| format!("Could not create {}: {}", destination.display(), e))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 16];
    loop {
        let read = entry
            .read(&mut buffer)
            .map_err(|e| format!("Could not read the setup file: {}", e))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        output
            .write_all(&buffer[..read])
            .map_err(|e| format!("Could not write {}: {}", destination.display(), e))?;
    }
    drop(output);
    if format!("{:x}", hasher.finalize()) != digest {
        let _ = std::fs::remove_file(destination);
        return Err(format!("The setup file is damaged: {} does not match its hash", reference));
    }
    Ok(())
}

/// An installed addon turned back into an installable file.
pub struct PackedAddon {
    pub file_path: PathBuf,
    pub content: AddonContent,
}

/// What an installed addon consists of, without archiving it.
#[derive(Debug, Clone, PartialEq)]
pub struct AddonContent {
    /// Hash over relative paths and file contents; the same files give the same value no
    /// matter how or when they were archived.
    pub content_hash: String,
    /// Bytes of all files together.
    pub size: u64,
}

/// Hashes an installed addon in place. This is all a setup needs when the addon's files do
/// not travel with it, and it is many times faster than packing them.
pub fn addon_content(main_python_file: &Path) -> Result<AddonContent, String> {
    if !is_addon_package(main_python_file) {
        let size = std::fs::metadata(main_python_file)
            .map_err(|e| format!("Could not read {}: {}", main_python_file.display(), e))?
            .len();
        let digest = sha256_of_path(main_python_file)?;
        return Ok(AddonContent {
            content_hash: content_hash_of(&[(file_name_of(main_python_file)?, digest)]),
            size,
        });
    }
    let directory = main_python_file
        .parent()
        .ok_or_else(|| format!("{} has no folder", main_python_file.display()))?;
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    collect_addon_files(directory, "", &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut size = 0u64;
    let mut hashes: Vec<(String, String)> = Vec::with_capacity(files.len());
    for (relative, path) in files {
        size += std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        hashes.push((relative, sha256_of_path(&path)?));
    }
    Ok(AddonContent {
        content_hash: content_hash_of(&hashes),
        size,
    })
}

/// Packs an addon for the bundle. A single-file addon is used as is. A package folder becomes
/// a zip: an extension with its files at the root, a legacy addon inside a folder of its name,
/// which is the layout Blender's installers expect for each.
pub fn pack_addon(main_python_file: &Path, is_extension: bool, work_directory: &Path) -> Result<PackedAddon, String> {
    if !is_addon_package(main_python_file) {
        return Ok(PackedAddon {
            file_path: main_python_file.to_path_buf(),
            content: addon_content(main_python_file)?,
        });
    }
    let directory = main_python_file
        .parent()
        .ok_or_else(|| format!("{} has no folder", main_python_file.display()))?;
    let folder_name = file_name_of(directory)?;
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    collect_addon_files(directory, "", &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));

    std::fs::create_dir_all(work_directory)
        .map_err(|e| format!("Could not create {}: {}", work_directory.display(), e))?;
    let zip_path = work_directory.join(format!("{}.zip", folder_name));
    let zip_file = std::fs::File::create(&zip_path)
        .map_err(|e| format!("Could not create {}: {}", zip_path.display(), e))?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let mut hashes: Vec<(String, String)> = Vec::with_capacity(files.len());
    let mut total_size = 0u64;
    for (relative, path) in &files {
        let entry_name = if is_extension {
            relative.clone()
        } else {
            format!("{}/{}", folder_name, relative)
        };
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        total_size += size;
        zip.start_file(entry_name, entry_options(is_stored_extension(path), size))
            .map_err(|e| format!("Could not pack {}: {}", path.display(), e))?;
        let mut source = std::fs::File::open(path).map_err(|e| format!("Could not read {}: {}", path.display(), e))?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 1 << 16];
        loop {
            let read = source
                .read(&mut buffer)
                .map_err(|e| format!("Could not read {}: {}", path.display(), e))?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
            zip.write_all(&buffer[..read])
                .map_err(|e| format!("Could not pack {}: {}", path.display(), e))?;
        }
        hashes.push((relative.clone(), format!("{:x}", hasher.finalize())));
    }
    zip.finish().map_err(|e| format!("Could not pack {}: {}", directory.display(), e))?;
    Ok(PackedAddon {
        file_path: zip_path,
        content: AddonContent {
            content_hash: content_hash_of(&hashes),
            size: total_size,
        },
    })
}

/// A package is a folder with an `__init__.py`; anything else is a single-file addon.
fn is_addon_package(main_python_file: &Path) -> bool {
    main_python_file
        .file_name()
        .map(|n| n.to_string_lossy().eq_ignore_ascii_case("__init__.py"))
        .unwrap_or(false)
}

fn is_stored_extension(path: &Path) -> bool {
    path.extension()
        .map(|e| STORED_EXTENSIONS.iter().any(|s| e.to_string_lossy().eq_ignore_ascii_case(s)))
        .unwrap_or(false)
}

fn collect_addon_files(directory: &Path, prefix: &str, found: &mut Vec<(String, PathBuf)>) -> Result<(), String> {
    let entries = std::fs::read_dir(directory).map_err(|e| format!("Could not read {}: {}", directory.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Could not read {}: {}", directory.display(), e))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        // Links are never followed: what they point at is not part of the addon.
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Could not read {}: {}", path.display(), e))?;
        if file_type.is_symlink() {
            continue;
        }
        let relative = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", prefix, name)
        };
        if file_type.is_dir() {
            if SKIPPED_DIRECTORIES.iter().any(|s| s.eq_ignore_ascii_case(&name)) {
                continue;
            }
            collect_addon_files(&path, &relative, found)?;
        } else if file_type.is_file() {
            let skipped = path
                .extension()
                .map(|e| SKIPPED_EXTENSIONS.iter().any(|s| e.to_string_lossy().eq_ignore_ascii_case(s)))
                .unwrap_or(false);
            if !skipped {
                found.push((relative, path));
            }
        }
    }
    Ok(())
}

fn content_hash_of(files: &[(String, String)]) -> String {
    let mut hasher = Sha256::new();
    for (relative, digest) in files {
        hasher.update(relative.as_bytes());
        hasher.update([0u8]);
        hasher.update(digest.as_bytes());
        hasher.update([b'\n']);
    }
    format!("{:x}", hasher.finalize())
}

/// A fixed timestamp keeps the archive of unchanged files byte-identical between exports.
fn entry_options(stored: bool, size: u64) -> zip::write::SimpleFileOptions {
    let method = if stored {
        zip::CompressionMethod::Stored
    } else {
        zip::CompressionMethod::Deflated
    };
    // Addon folders run to hundreds of megabytes: the fastest deflate level keeps an export
    // to seconds and still halves the Python and text that make up most of the rest.
    zip::write::SimpleFileOptions::default()
        .compression_method(method)
        .compression_level(if stored { None } else { Some(1) })
        .last_modified_time(zip::DateTime::default())
        .large_file(size >= u32::MAX as u64)
}

fn open_bundle(file_path: &Path) -> Result<zip::ZipArchive<std::fs::File>, String> {
    let file = std::fs::File::open(file_path).map_err(|e| format!("Could not open {}: {}", file_path.display(), e))?;
    zip::ZipArchive::new(file).map_err(|_| String::from("This is not a Blenderbase setup file"))
}

fn file_name_of(path: &Path) -> Result<String, String> {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or_else(|| format!("{} has no name", path.display()))
}

fn sha256_of_path(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|e| format!("Could not read {}: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 16];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| format!("Could not read {}: {}", path.display(), e))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::setup::manifest::{SetupBlob, SetupMeta, SetupSeries};

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("blenderbase-setup-{}-{}", label, uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_bundle_round_trips_and_verifies_its_blobs() {
        let dir = temp_dir("bundle");
        let theme = dir.join("theme.xml");
        std::fs::write(&theme, "<bpy><Theme/></bpy>").unwrap();
        let bundle_path = dir.join("my.bbsetup");

        let mut writer = SetupBundleWriter::create(&bundle_path).unwrap();
        let (theme_ref, theme_size) = writer.add_blob_from_file(&theme).unwrap();
        let (again_ref, _) = writer.add_blob_from_file(&theme).unwrap();
        assert_eq!(theme_ref, again_ref, "the same content is one blob");
        let (prefs_ref, prefs_size) = writer.add_blob_from_bytes(b"{\"view\": {}}").unwrap();

        let mut manifest = SetupManifest::new(SetupMeta::default());
        manifest.series.insert(
            String::from("5.2"),
            SetupSeries {
                captured_with: String::from("5.2.1"),
                preferences: Some(SetupBlob { blob: prefs_ref, size: prefs_size, name: None, format: None }),
                theme: Some(SetupBlob { blob: theme_ref.clone(), size: theme_size, name: Some(String::from("Default")), format: None }),
                ..SetupSeries::default()
            },
        );
        assert!(!bundle_path.exists(), "nothing is visible under the final name before finish");
        writer.finish(&manifest).unwrap();
        assert!(bundle_path.exists());
        assert!(!dir.join("my.bbsetup.part").exists());

        let read = read_bundle_manifest(&bundle_path).unwrap();
        assert_eq!(read, manifest);
        let restored = dir.join("out").join("theme.xml");
        extract_bundle_blob(&bundle_path, &theme_ref, &restored).unwrap();
        assert_eq!(std::fs::read_to_string(&restored).unwrap(), "<bpy><Theme/></bpy>");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_manifest_that_names_a_missing_blob_is_refused() {
        let dir = temp_dir("missing");
        let bundle_path = dir.join("broken.bbsetup");
        let writer = SetupBundleWriter::create(&bundle_path).unwrap();
        let mut manifest = SetupManifest::new(SetupMeta::default());
        manifest.series.insert(
            String::from("5.2"),
            SetupSeries {
                theme: Some(SetupBlob { blob: blob_reference(&"a".repeat(64)), size: 1, name: None, format: None }),
                ..SetupSeries::default()
            },
        );
        assert!(writer.finish(&manifest).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_blob_that_does_not_match_its_name_is_not_left_behind() {
        let dir = temp_dir("tampered");
        let bundle_path = dir.join("tampered.bbsetup");
        let claimed = "b".repeat(64);
        {
            let file = std::fs::File::create(&bundle_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            zip.start_file(format!("{}{}", BLOB_DIRECTORY, claimed), entry_options(false, 4)).unwrap();
            zip.write_all(b"evil").unwrap();
            zip.finish().unwrap();
        }
        let destination = dir.join("addon.zip");
        let refusal = extract_bundle_blob(&bundle_path, &blob_reference(&claimed), &destination).unwrap_err();
        assert!(refusal.contains("does not match"), "{}", refusal);
        assert!(!destination.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn packing_an_addon_folder_is_repeatable_and_skips_caches() {
        let dir = temp_dir("pack");
        let addon = dir.join("my_addon");
        std::fs::create_dir_all(addon.join("__pycache__")).unwrap();
        std::fs::create_dir_all(addon.join("ui")).unwrap();
        std::fs::write(addon.join("__init__.py"), "bl_info = {}").unwrap();
        std::fs::write(addon.join("ui").join("panel.py"), "pass").unwrap();
        std::fs::write(addon.join("__pycache__").join("x.pyc"), "cache").unwrap();

        let legacy = pack_addon(&addon.join("__init__.py"), false, &dir.join("w1")).unwrap();
        let again = pack_addon(&addon.join("__init__.py"), false, &dir.join("w2")).unwrap();
        assert_eq!(legacy.content, again.content);
        assert_eq!(addon_content(&addon.join("__init__.py")).unwrap(), legacy.content, "hashing in place agrees with packing");
        assert_eq!(legacy.content.size, ("bl_info = {}".len() + "pass".len()) as u64);
        assert_eq!(std::fs::read(&legacy.file_path).unwrap(), std::fs::read(&again.file_path).unwrap());

        let names = |path: &Path| -> Vec<String> {
            let mut archive = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
            (0..archive.len()).map(|i| archive.by_index(i).unwrap().name().to_string()).collect()
        };
        assert_eq!(names(&legacy.file_path), vec!["my_addon/__init__.py", "my_addon/ui/panel.py"]);
        let extension = pack_addon(&addon.join("__init__.py"), true, &dir.join("w3")).unwrap();
        assert_eq!(names(&extension.file_path), vec!["__init__.py", "ui/panel.py"]);
        assert_eq!(extension.content, legacy.content, "the layout does not change the content hash");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
