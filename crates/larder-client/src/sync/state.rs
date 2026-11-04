use std::io;
use std::path::{Path, PathBuf};

use larder_core::fsutil::{load_json, save_json};
use larder_core::{Base, Manifest};
use serde::{Deserialize, Serialize};

/// What a client remembers about one synced shelf between runs.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShelfState {
    /// Hash of every file as of the last time we and the host agreed on it.
    pub base: Base,
    /// The last local scan. Only used so unchanged files skip rehashing.
    pub scan: Manifest,
}

pub struct StateFile {
    path: PathBuf,
}

impl StateFile {
    /// The local folder is part of the file name. Point a shelf at a new
    /// folder and it starts from scratch, instead of reading the old
    /// folder's history and deciding everything in the new, empty one got
    /// deleted.
    pub fn new(dir: &Path, shelf: &str, root: &Path) -> Self {
        let tag = blake3::hash(root.to_string_lossy().as_bytes()).to_hex();
        StateFile { path: dir.join(format!("{shelf}-{}.json", &tag[..12])) }
    }

    pub fn load(&self) -> io::Result<ShelfState> {
        load_json(&self.path)
    }

    pub fn save(&self, state: &ShelfState) -> io::Result<()> {
        save_json(&self.path, state)
    }
}
