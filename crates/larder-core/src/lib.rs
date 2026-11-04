pub mod conflict;
pub mod paths;
pub mod plan;
pub mod scan;

pub use paths::RelPath;
pub use plan::{Action, Base};
pub use scan::{FileEntry, Manifest};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not a valid shelf path: {0:?}")]
    InvalidPath(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
