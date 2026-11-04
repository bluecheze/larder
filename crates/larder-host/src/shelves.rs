use std::fs;
use std::io;
use std::path::PathBuf;

use larder_core::paths::valid_shelf_name;

/// Shelves are just the top-level folders in the library. No extra
/// database, so the host's owner can still poke around in Finder or
/// Explorer and everything makes sense.
pub struct Shelves {
    root: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum ShelfError {
    #[error("{0:?} isn't a usable shelf name")]
    BadName(String),
    #[error("there's no shelf called {0:?}")]
    Missing(String),
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl Shelves {
    pub fn new(root: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&root)?;
        Ok(Shelves { root })
    }

    pub fn names(&self) -> io::Result<Vec<String>> {
        let mut names: Vec<String> = fs::read_dir(&self.root)?
            .filter_map(Result::ok)
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|name| valid_shelf_name(name))
            .collect();
        names.sort_by_key(|n| n.to_lowercase());
        Ok(names)
    }

    pub fn create(&self, name: &str) -> Result<PathBuf, ShelfError> {
        let name = name.trim();
        if !valid_shelf_name(name) {
            return Err(ShelfError::BadName(name.into()));
        }
        let dir = self.root.join(name);
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    pub fn dir(&self, name: &str) -> Result<PathBuf, ShelfError> {
        if !valid_shelf_name(name) {
            return Err(ShelfError::BadName(name.into()));
        }
        let dir = self.root.join(name);
        if !dir.is_dir() {
            return Err(ShelfError::Missing(name.into()));
        }
        Ok(dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_list_and_find() {
        let dir = tempfile::tempdir().unwrap();
        let shelves = Shelves::new(dir.path().to_path_buf()).unwrap();
        shelves.create("photos").unwrap();
        shelves.create(" Documents ").unwrap();
        fs::write(dir.path().join("stray.txt"), "not a shelf").unwrap();
        fs::create_dir(dir.path().join(".hidden")).unwrap();
        assert_eq!(shelves.names().unwrap(), ["Documents", "photos"]);
        assert!(shelves.dir("photos").is_ok());
        assert!(matches!(shelves.dir("music"), Err(ShelfError::Missing(_))));
        assert!(matches!(shelves.create("../evil"), Err(ShelfError::BadName(_))));
    }
}
