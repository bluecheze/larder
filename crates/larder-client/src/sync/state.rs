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
    pub fn new(dir: &Path, shelf: &str) -> Self {
        StateFile { path: dir.join(format!("{shelf}.json")) }
    }

    pub fn load(&self) -> io::Result<ShelfState> {
        load_json(&self.path)
    }

    pub fn save(&self, state: &ShelfState) -> io::Result<()> {
        save_json(&self.path, state)
    }
}
