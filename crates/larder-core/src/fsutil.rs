use std::fs;
use std::io;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Writes to a temp file next to the target and renames it over. If the app
/// dies halfway through, you keep the old file instead of half a JSON blob.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)
}

pub fn save_json<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    write_atomic(path, &bytes)
}

/// Missing file means "first run", so that gives you the default instead
/// of an error. A file that exists but won't parse is still an error, we
/// don't want to silently wipe someone's device list.
pub fn load_json<T: DeserializeOwned + Default>(path: &Path) -> io::Result<T> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(io::Error::other),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(T::default()),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn save_then_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/state.json");
        let mut data = BTreeMap::new();
        data.insert("a".to_string(), 1);
        save_json(&path, &data).unwrap();
        let back: BTreeMap<String, i32> = load_json(&path).unwrap();
        assert_eq!(back, data);
    }

    #[test]
    fn missing_file_is_default_but_garbage_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let missing: Vec<u8> = load_json(&dir.path().join("nope.json")).unwrap();
        assert!(missing.is_empty());
        let bad = dir.path().join("bad.json");
        fs::write(&bad, "{not json").unwrap();
        assert!(load_json::<Vec<u8>>(&bad).is_err());
    }
}
