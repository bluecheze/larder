mod apply;
mod date;
mod state;
mod watch;

use std::path::Path;

use larder_core::plan::plan;
use larder_core::scan::{matches_pattern, scan_with};
use larder_core::RelPath;
use serde::Serialize;

pub use state::{ShelfState, StateFile};
pub use watch::{watch, Watch};

use crate::api::HostClient;
use crate::error::ClientError;

/// One shelf being synced into one local folder.
pub struct SyncJob<'a> {
    pub client: &'a HostClient,
    pub shelf: &'a str,
    pub root: &'a Path,
    /// Goes into conflict copy names, so you can tell whose edit it was.
    pub device: &'a str,
    /// Extra names to skip, from the user's settings.
    pub ignore: &'a [String],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    Uploaded,
    Downloaded,
    /// Deleted here, so we deleted it on the host.
    Deleted,
    /// Deleted on the host, so we removed our copy.
    Removed,
    Conflict,
}

#[derive(Debug, Clone, Serialize)]
pub struct Change {
    pub shelf: String,
    pub path: String,
    pub kind: ChangeKind,
}

/// One full pass: scan the folder, ask the host what it has, plan, apply.
/// `state` is updated as things succeed, so save it even when this errors
/// halfway, it still describes what really happened.
pub async fn sync_shelf(job: &SyncJob<'_>, state: &mut ShelfState) -> Result<Vec<Change>, ClientError> {
    // The whole folder vanishing (deleted, renamed, drive unplugged) is not
    // the same as deleting every file in it one by one. Treat it as a fresh
    // start and pull everything down again, never as "delete it all on the
    // host". This is the classic way sync apps wipe people's servers.
    if !job.root.exists() {
        *state = ShelfState::default();
    }
    tokio::fs::create_dir_all(job.root).await?;
    let root = job.root.to_path_buf();
    let previous = state.scan.clone();
    let ignore = job.ignore.to_vec();
    let local = tokio::task::spawn_blocking(move || scan_with(&root, &previous, &ignore))
        .await
        .map_err(std::io::Error::other)??;
    // Ignored paths drop out of every side before planning. Otherwise adding
    // "*.psd" to the ignore list would look exactly like deleting all your
    // psd files, and the host would get told to delete them too.
    let mut remote = job.client.manifest(job.shelf).await?;
    remote.retain(|path, _| !ignored(path, job.ignore));
    let mut base = state.base.clone();
    base.retain(|path, _| !ignored(path, job.ignore));
    let mut changes = Vec::new();
    for action in plan(&local, &remote, &base) {
        if let Some(change) = apply::apply(job, &local, &remote, state, action).await? {
            changes.push(change);
        }
    }
    state.scan = local;
    Ok(changes)
}

fn ignored(path: &RelPath, patterns: &[String]) -> bool {
    path.as_str().split('/').any(|part| patterns.iter().any(|p| matches_pattern(p, part)))
}
