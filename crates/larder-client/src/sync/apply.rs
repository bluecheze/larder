use std::io;
use std::path::Path;

use filetime::FileTime;
use larder_core::conflict::conflict_name;
use larder_core::scan::hash_file;
use larder_core::{Action, Manifest, RelPath};

use super::date::today;
use super::state::ShelfState;
use super::{Change, ChangeKind, SyncJob};
use crate::error::ClientError;

/// Carries out one planned action and keeps `state.base` honest: it only
/// moves forward once the change has actually happened on both sides.
/// Races with the host (someone else got there first) are skipped, the
/// next pass sees the new situation and plans again.
pub(super) async fn apply(
    job: &SyncJob<'_>,
    local: &Manifest,
    remote: &Manifest,
    state: &mut ShelfState,
    action: Action,
) -> Result<Option<Change>, ClientError> {
    match action {
        Action::Upload { path, base } => upload(job, local, state, path, base).await,
        Action::Download { path, hash } => {
            let modified = remote.get(&path).map_or(0, |e| e.modified);
            download(job, local, state, path, hash, modified).await
        }
        Action::DeleteLocal { path } => delete_local(job, state, path).await,
        Action::DeleteRemote { path, base } => delete_remote(job, state, path, base).await,
        Action::Conflict { path, remote_hash } => {
            let modified = remote.get(&path).map_or(0, |e| e.modified);
            conflict(job, state, path, remote_hash, modified).await
        }
        Action::Settle { path, hash } => {
            match hash {
                Some(hash) => state.base.insert(path, hash),
                None => state.base.remove(&path),
            };
            Ok(None)
        }
    }
}

fn done(job: &SyncJob<'_>, path: &RelPath, kind: ChangeKind) -> Option<Change> {
    Some(Change { shelf: job.shelf.to_string(), path: path.to_string(), kind })
}

async fn upload(
    job: &SyncJob<'_>,
    local: &Manifest,
    state: &mut ShelfState,
    path: RelPath,
    base: Option<String>,
) -> Result<Option<Change>, ClientError> {
    let modified = local.get(&path).map_or(0, |e| e.modified);
    match job.client.upload(job.shelf, &path, &path.to_fs(job.root), base.as_deref(), modified).await {
        Ok(entry) => {
            state.base.insert(path.clone(), entry.hash);
            Ok(done(job, &path, ChangeKind::Uploaded))
        }
        Err(ClientError::Conflict(_)) => Ok(None),
        Err(ClientError::Io(err)) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

async fn download(
    job: &SyncJob<'_>,
    local: &Manifest,
    state: &mut ShelfState,
    path: RelPath,
    hash: String,
    modified: i64,
) -> Result<Option<Change>, ClientError> {
    let target = path.to_fs(job.root);
    // If someone edited the file since we scanned, leave it alone. The next
    // pass will see a local change and plan a conflict instead of us
    // quietly overwriting their work. There's still a tiny window between
    // this check and the rename, which is as good as it gets without
    // OS-level file locking.
    if !unchanged_since_scan(&target, local.get(&path).map(|e| e.hash.as_str()))? {
        return Ok(None);
    }
    match job.client.download(job.shelf, &path, &hash, &target).await {
        Ok(()) => {
            stamp(&target, modified);
            state.base.insert(path.clone(), hash);
            Ok(done(job, &path, ChangeKind::Downloaded))
        }
        Err(ClientError::NotFound) => Ok(None),
        Err(err) => Err(err),
    }
}

async fn delete_local(job: &SyncJob<'_>, state: &mut ShelfState, path: RelPath) -> Result<Option<Change>, ClientError> {
    let target = path.to_fs(job.root);
    let expected = state.base.get(&path).cloned();
    if !unchanged_since_scan(&target, expected.as_deref())? {
        return Ok(None);
    }
    match tokio::fs::remove_file(&target).await {
        Ok(()) => prune_empty_dirs(job.root, &target).await,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err.into()),
    }
    state.base.remove(&path);
    Ok(done(job, &path, ChangeKind::Removed))
}

async fn delete_remote(
    job: &SyncJob<'_>,
    state: &mut ShelfState,
    path: RelPath,
    base: String,
) -> Result<Option<Change>, ClientError> {
    match job.client.delete(job.shelf, &path, &base).await {
        Ok(()) | Err(ClientError::NotFound) => {
            state.base.remove(&path);
            Ok(done(job, &path, ChangeKind::Deleted))
        }
        Err(ClientError::Conflict(_)) => Ok(None),
        Err(err) => Err(err),
    }
}

async fn conflict(
    job: &SyncJob<'_>,
    state: &mut ShelfState,
    path: RelPath,
    remote_hash: String,
    modified: i64,
) -> Result<Option<Change>, ClientError> {
    let target = path.to_fs(job.root);
    let copy = conflict_name(&path, job.device, &today()).to_fs(job.root);
    tokio::fs::rename(&target, &copy).await?;
    // The copy is a brand new local file now, so the next pass uploads it
    // and every other computer gets both versions.
    job.client.download(job.shelf, &path, &remote_hash, &target).await?;
    stamp(&target, modified);
    state.base.insert(path.clone(), remote_hash);
    Ok(done(job, &path, ChangeKind::Conflict))
}

fn unchanged_since_scan(target: &Path, scanned: Option<&str>) -> io::Result<bool> {
    match (target.exists(), scanned) {
        (false, _) => Ok(true),
        (true, None) => Ok(false),
        (true, Some(hash)) => Ok(hash_file(target)? == hash),
    }
}

fn stamp(target: &Path, modified: i64) {
    if modified > 0 {
        let time = FileTime::from_unix_time(modified.div_euclid(1000), (modified.rem_euclid(1000) * 1_000_000) as u32);
        let _ = filetime::set_file_mtime(target, time);
    }
}

async fn prune_empty_dirs(root: &Path, removed: &Path) {
    let mut dir = removed.parent();
    while let Some(current) = dir {
        if current == root || tokio::fs::remove_dir(current).await.is_err() {
            break;
        }
        dir = current.parent();
    }
}
