use std::fmt;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Error;

/// A file path relative to a shelf root, always written with `/`.
///
/// Every path that comes off the network goes through `RelPath::parse`, so
/// the host never ends up writing outside a shelf because a client sent
/// `../../etc/passwd`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RelPath(String);

impl RelPath {
    pub fn parse(raw: &str) -> Result<Self, Error> {
        let bad = || Error::InvalidPath(raw.to_string());
        if raw.is_empty() || raw.starts_with('/') || raw.contains('\\') || raw.contains('\0') {
            return Err(bad());
        }
        for part in raw.split('/') {
            if !valid_component(part) {
                return Err(bad());
            }
        }
        Ok(RelPath(raw.to_string()))
    }

    /// Turns a path found on disk back into a RelPath. Returns None for
    /// anything that wouldn't survive the trip to another OS.
    pub fn from_fs(root: &Path, full: &Path) -> Option<Self> {
        let rel = full.strip_prefix(root).ok()?;
        let mut parts = Vec::new();
        for component in rel.components() {
            match component {
                Component::Normal(part) => parts.push(part.to_str()?),
                _ => return None,
            }
        }
        RelPath::parse(&parts.join("/")).ok()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_fs(&self, root: &Path) -> PathBuf {
        let mut out = root.to_path_buf();
        out.extend(self.0.split('/'));
        out
    }

    pub fn file_name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }
}

/// Shelf names end up as folder names on the host, so they follow the same
/// rules as a single path segment.
pub fn valid_shelf_name(name: &str) -> bool {
    !name.contains('/') && !name.contains('\\') && valid_component(name) && !name.starts_with('.')
}

fn valid_component(part: &str) -> bool {
    if part.is_empty() || part == "." || part == ".." {
        return false;
    }
    // Windows quietly drops trailing dots and spaces, so "notes." and "notes"
    // would turn into the same file there. Refuse them everywhere instead.
    if part.ends_with('.') || part.ends_with(' ') {
        return false;
    }
    // ':' is a drive letter or an NTFS alternate stream on Windows. The rest
    // are just illegal in Windows filenames.
    !part.chars().any(|c| matches!(c, ':' | '<' | '>' | '"' | '|' | '?' | '*') || c.is_control())
}

impl fmt::Display for RelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for RelPath {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        RelPath::parse(&value)
    }
}

impl From<RelPath> for String {
    fn from(value: RelPath) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_normal_paths() {
        for ok in ["a.txt", "docs/report.pdf", "Photos/2026/IMG_0001.HEIC", "with space/x y.md"] {
            assert!(RelPath::parse(ok).is_ok(), "{ok}");
        }
    }

    #[test]
    fn rejects_escapes_and_windows_trouble() {
        for bad in [
            "", "/abs", "../up", "a/../b", "a/./b", "a//b", "a\\b", "C:/x", "trail.", "trail ",
            "what?", "a\0b", "pipe|name",
        ] {
            assert!(RelPath::parse(bad).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn round_trips_through_the_filesystem() {
        let root = Path::new("/tmp/shelf");
        let rel = RelPath::parse("a/b/c.txt").unwrap();
        let full = rel.to_fs(root);
        assert_eq!(RelPath::from_fs(root, &full), Some(rel));
    }

    #[test]
    fn shelf_names_are_single_segments() {
        assert!(valid_shelf_name("Photos"));
        assert!(!valid_shelf_name("a/b"));
        assert!(!valid_shelf_name(".larder"));
        assert!(!valid_shelf_name(""));
    }

    #[test]
    fn serde_runs_validation() {
        let parsed: Result<RelPath, _> = serde_json::from_str("\"../nope\"");
        assert!(parsed.is_err());
    }
}
