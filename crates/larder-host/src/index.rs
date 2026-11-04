use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;

use larder_core::fsutil::{load_json, save_json};
use larder_core::scan::{hash_file, scan};
use larder_core::{FileEntry, Manifest, RelPath};
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

/// Hash cache for every shelf, saved under the host's data folder.
///
/// Each shelf has one async lock around its manifest. Anything that changes
/// files on a shelf holds it, so two uploads to the same shelf can't race
/// each other's conflict checks. Different shelves don't block each other.
pub struct Index {
    dir: PathBuf,
    shelves: Mutex<HashMap<String, Arc<AsyncMutex<Option<Manifest>>>>>,
}

pub type ShelfGuard = OwnedMutexGuard<Option<Manifest>>;

impl Index {
    pub fn new(dir: PathBuf) -> Self {
        Index { dir, shelves: Mutex::new(HashMap::new()) }
    }

    pub async fn lock(&self, shelf: &str) -> io::Result<ShelfGuard> {
        let slot = self.shelves.lock().unwrap().entry(shelf.to_string()).or_default().clone();
        let mut guard = slot.lock_owned().await;
        if guard.is_none() {
            *guard = Some(load_json(&self.file(shelf))?);
        }
        Ok(guard)
    }

    /// Rescans the shelf and stores the result. Unchanged files keep their
    /// cached hash, so this is mostly a directory walk.
    pub async fn refresh(&self, shelf: &str, root: PathBuf) -> io::Result<Manifest> {
        let mut guard = self.lock(shelf).await?;
        let previous = guard.take().unwrap_or_default();
        let fresh = tokio::task::spawn_blocking(move || scan(&root, &previous))
            .await
            .map_err(io::Error::other)??;
        self.save(shelf, &fresh)?;
        *guard = Some(fresh.clone());
        Ok(fresh)
    }

    pub fn save(&self, shelf: &str, manifest: &Manifest) -> io::Result<()> {
        save_json(&self.file(shelf), manifest)
    }

    fn file(&self, shelf: &str) -> PathBuf {
        self.dir.join(format!("{shelf}.json"))
    }
}

/// What's on disk right now for one path. Uses the cache when size and
/// mtime still match, otherwise hashes the file.
pub fn current_entry(cache: &Manifest, root: &Path, path: &RelPath) -> io::Result<Option<FileEntry>> {
    let full = path.to_fs(root);
    let meta = match std::fs::metadata(&full) {
        Ok(meta) if meta.is_file() => meta,
        Ok(_) => return Ok(None),
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err),
    };
    let size = meta.len();
    let modified = meta
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let hash = match cache.get(path) {
        Some(old) if old.size == size && old.modified == modified => old.hash.clone(),
        _ => hash_file(&full)?,
    };
    Ok(Some(FileEntry { size, modified, hash }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn refresh_scans_and_persists() {
        let data = tempfile::tempdir().unwrap();
        let shelf = tempfile::tempdir().unwrap();
        std::fs::write(shelf.path().join("a.txt"), "hello").unwrap();
        let index = Index::new(data.path().to_path_buf());
        let manifest = index.refresh("docs", shelf.path().to_path_buf()).await.unwrap();
        assert_eq!(manifest.len(), 1);

        let reopened = Index::new(data.path().to_path_buf());
        let guard = reopened.lock("docs").await.unwrap();
        assert_eq!(guard.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn current_entry_sees_missing_and_present_files() {
        let shelf = tempfile::tempdir().unwrap();
        let path = RelPath::parse("x/y.txt").unwrap();
        assert!(current_entry(&Manifest::new(), shelf.path(), &path).unwrap().is_none());
        std::fs::create_dir_all(shelf.path().join("x")).unwrap();
        std::fs::write(shelf.path().join("x/y.txt"), "data").unwrap();
        let entry = current_entry(&Manifest::new(), shelf.path(), &path).unwrap().unwrap();
        assert_eq!(entry.size, 4);
    }
}
