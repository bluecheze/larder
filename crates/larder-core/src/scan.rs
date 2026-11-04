use std::collections::BTreeMap;
use std::fs::File;
use std::io;
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::paths::RelPath;

/// Suffix for downloads that haven't finished yet. They get renamed into
/// place once the hash checks out.
pub const PARTIAL_SUFFIX: &str = ".larder-part";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub size: u64,
    /// Milliseconds since the unix epoch.
    pub modified: i64,
    pub hash: String,
}

pub type Manifest = BTreeMap<RelPath, FileEntry>;

/// Walks a shelf and returns every file in it with its hash.
///
/// `previous` is the last scan. If a file's size and mtime haven't moved we
/// reuse the old hash, otherwise a big photo library would be rehashed every
/// few seconds.
pub fn scan(root: &Path, previous: &Manifest) -> io::Result<Manifest> {
    let mut found = Manifest::new();
    let walker = WalkDir::new(root).follow_links(false).into_iter();
    for item in walker.filter_entry(|e| !is_ignored(e.file_name().to_str().unwrap_or(""))) {
        let item = match item {
            Ok(item) => item,
            // A file can vanish between listing the folder and reading it.
            // That's normal while someone is working, so skip it and let the
            // next scan pick things up.
            Err(err) if err.io_error().map(|e| e.kind()) == Some(io::ErrorKind::NotFound) => continue,
            Err(err) => return Err(err.into()),
        };
        // Symlinks are skipped on purpose. Following them can pull in files
        // from outside the shelf, and "syncing" a link to a folder elsewhere
        // is how you end up deleting things you never meant to share.
        if !item.file_type().is_file() {
            continue;
        }
        let Some(rel) = RelPath::from_fs(root, item.path()) else {
            continue;
        };
        let meta = match item.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        let size = meta.len();
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let hash = match previous.get(&rel) {
            Some(old) if old.size == size && old.modified == modified => old.hash.clone(),
            _ => match hash_file(item.path()) {
                Ok(hash) => hash,
                Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
                Err(err) => return Err(err),
            },
        };
        found.insert(rel, FileEntry { size, modified, hash });
    }
    Ok(found)
}

pub fn hash_file(path: &Path) -> io::Result<String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update_reader(File::open(path)?)?;
    Ok(hasher.finalize().to_hex().to_string())
}

/// OS junk and our own temp files. None of it should ever be synced.
pub fn is_ignored(name: &str) -> bool {
    matches!(name, ".DS_Store" | "Thumbs.db" | "desktop.ini" | ".larder")
        || name.ends_with(PARTIAL_SUFFIX)
        || name.starts_with("._")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn finds_nested_files_and_skips_junk() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a/b")).unwrap();
        fs::write(dir.path().join("a/b/note.txt"), "hi").unwrap();
        fs::write(dir.path().join("top.md"), "x").unwrap();
        fs::write(dir.path().join(".DS_Store"), "junk").unwrap();
        fs::write(dir.path().join("big.bin.larder-part"), "half").unwrap();

        let manifest = scan(dir.path(), &Manifest::new()).unwrap();
        let names: Vec<_> = manifest.keys().map(|k| k.as_str()).collect();
        assert_eq!(names, ["a/b/note.txt", "top.md"]);
        assert_eq!(manifest[&RelPath::parse("top.md").unwrap()].size, 1);
    }

    #[test]
    fn hash_changes_with_content() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("f");
        fs::write(&file, "one").unwrap();
        let first = hash_file(&file).unwrap();
        fs::write(&file, "two").unwrap();
        assert_ne!(first, hash_file(&file).unwrap());
    }

    #[test]
    fn reuses_hash_when_size_and_mtime_match() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("f"), "data").unwrap();
        let mut first = scan(dir.path(), &Manifest::new()).unwrap();
        let key = RelPath::parse("f").unwrap();
        first.get_mut(&key).unwrap().hash = "cached".into();
        let second = scan(dir.path(), &first).unwrap();
        assert_eq!(second[&key].hash, "cached");
    }
}
