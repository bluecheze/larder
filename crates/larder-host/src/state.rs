use std::path::PathBuf;
use std::sync::Arc;

use crate::devices::Devices;
use crate::index::Index;
use crate::shelves::Shelves;

#[derive(Clone)]
pub struct AppState {
    pub name: Arc<str>,
    pub devices: Arc<Devices>,
    pub shelves: Arc<Shelves>,
    pub index: Arc<Index>,
    /// Lives inside the library on purpose. Uploads get renamed from here
    /// into place, and a rename only works on the same disk. People love
    /// putting their library on an external drive.
    pub tmp_dir: PathBuf,
}
