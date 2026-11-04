mod apply;
mod date;
mod state;
mod watch;

use std::path::Path;

use larder_core::plan::plan;
use larder_core::scan::scan;
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
    tokio::fs::create_dir_all(job.root).await?;
    let root = job.root.to_path_buf();
    let previous = state.scan.clone();
    let local = tokio::task::spawn_blocking(move || scan(&root, &previous))
        .await
        .map_err(std::io::Error::other)??;
    let remote = job.client.manifest(job.shelf).await?;
    let mut changes = Vec::new();
    for action in plan(&local, &remote, &state.base) {
        if let Some(change) = apply::apply(job, &local, &remote, state, action).await? {
            changes.push(change);
        }
    }
    state.scan = local;
    Ok(changes)
}
