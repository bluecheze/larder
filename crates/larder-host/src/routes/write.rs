use std::path::Path as FsPath;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use filetime::FileTime;
use larder_core::protocol::{MoveRequest, HEADER_BASE, HEADER_HASH, HEADER_MODIFIED};
use larder_core::{FileEntry, RelPath};

use crate::auth::Authed;
use crate::error::ApiError;
use crate::index::current_entry;
use crate::state::AppState;
use crate::upload::{stage, Staged};

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

/// "" and a missing header both mean the client thinks the file is new.
fn expected_base(headers: &HeaderMap) -> Option<String> {
    header(headers, HEADER_BASE).filter(|v| !v.is_empty()).map(str::to_string)
}

pub async fn put_file(
    State(state): State<AppState>,
    Authed(device): Authed,
    Path((shelf, raw)): Path<(String, String)>,
    headers: HeaderMap,
    body: Body,
) -> Result<Json<FileEntry>, ApiError> {
    let path = RelPath::parse(&raw)?;
    let root = state.shelves.dir(&shelf)?;
    // The body goes to disk before we take the shelf lock. A slow upload
    // over someone's home internet shouldn't stall every other client.
    let staged = stage(body, &state.tmp_dir).await?;
    let hash = staged.hash.clone();
    match commit(&state, &shelf, &root, &path, &headers, staged).await {
        Ok(entry) => Ok(Json(entry)),
        Err((err, leftover)) => {
            if let Some(staged) = leftover {
                staged.discard().await;
            }
            tracing::debug!(device = %device.name, %path, %hash, "upload refused");
            Err(err)
        }
    }
}

async fn commit(
    state: &AppState,
    shelf: &str,
    root: &FsPath,
    path: &RelPath,
    headers: &HeaderMap,
    staged: Staged,
) -> Result<FileEntry, (ApiError, Option<Staged>)> {
    if header(headers, HEADER_HASH).is_some_and(|h| h != staged.hash) {
        return Err((ApiError::BadRequest("the upload got corrupted on the way".into()), Some(staged)));
    }
    let mut guard = match state.index.lock(shelf).await {
        Ok(guard) => guard,
        Err(err) => return Err((err.into(), Some(staged))),
    };
    let cache = guard.get_or_insert_with(Default::default);
    let current = match current_entry(cache, root, path) {
        Ok(current) => current,
        Err(err) => return Err((err.into(), Some(staged))),
    };
    if current.as_ref().map(|e| &e.hash) != expected_base(headers).as_ref() {
        return Err((ApiError::Conflict(current.map(|e| e.hash)), Some(staged)));
    }
    let target = path.to_fs(root);
    let placed = async {
        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::rename(&staged.path, &target).await
    };
    if let Err(err) = placed.await {
        return Err((err.into(), Some(staged)));
    }
    if let Some(millis) = header(headers, HEADER_MODIFIED).and_then(|v| v.parse::<i64>().ok()) {
        let stamp = FileTime::from_unix_time(millis.div_euclid(1000), (millis.rem_euclid(1000) * 1_000_000) as u32);
        // Not fatal. Worst case the file shows today's date on the host.
        let _ = filetime::set_file_mtime(&target, stamp);
    }
    let entry = current_entry(cache, root, path)
        .ok()
        .flatten()
        .map(|e| FileEntry { hash: staged.hash.clone(), ..e })
        .ok_or_else(|| (ApiError::Internal("upload vanished after rename".into()), None))?;
    cache.insert(path.clone(), entry.clone());
    // The whole shelf index gets rewritten on every upload. Fine for a few
    // thousand files. If libraries get huge this should move to sqlite.
    state.index.save(shelf, cache).map_err(|e| (e.into(), None))?;
    Ok(entry)
}

pub async fn delete_file(
    State(state): State<AppState>,
    _auth: Authed,
    Path((shelf, raw)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let path = RelPath::parse(&raw)?;
    let root = state.shelves.dir(&shelf)?;
    let mut guard = state.index.lock(&shelf).await?;
    let cache = guard.get_or_insert_with(Default::default);
    let current = current_entry(cache, &root, &path)?.ok_or_else(|| ApiError::NotFound("no such file".into()))?;
    if Some(&current.hash) != expected_base(&headers).as_ref() {
        return Err(ApiError::Conflict(Some(current.hash)));
    }
    let target = path.to_fs(&root);
    tokio::fs::remove_file(&target).await?;
    prune_empty_dirs(&root, &target).await;
    cache.remove(&path);
    state.index.save(&shelf, cache)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn move_file(
    State(state): State<AppState>,
    _auth: Authed,
    Path(shelf): Path<String>,
    Json(req): Json<MoveRequest>,
) -> Result<Json<FileEntry>, ApiError> {
    let from = RelPath::parse(&req.from)?;
    let to = RelPath::parse(&req.to)?;
    let root = state.shelves.dir(&shelf)?;
    let mut guard = state.index.lock(&shelf).await?;
    let cache = guard.get_or_insert_with(Default::default);
    let entry = current_entry(cache, &root, &from)?.ok_or_else(|| ApiError::NotFound("no such file".into()))?;
    if let Some(existing) = current_entry(cache, &root, &to)? {
        return Err(ApiError::Conflict(Some(existing.hash)));
    }
    let target = to.to_fs(&root);
    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::rename(from.to_fs(&root), &target).await?;
    prune_empty_dirs(&root, &from.to_fs(&root)).await;
    cache.remove(&from);
    cache.insert(to, entry.clone());
    state.index.save(&shelf, cache)?;
    Ok(Json(entry))
}

/// Deleting the last file in a folder takes the folder with it, the same
/// way it would look on a client. Stops at the shelf root.
async fn prune_empty_dirs(root: &FsPath, removed: &FsPath) {
    let mut dir = removed.parent();
    while let Some(current) = dir {
        if current == root || tokio::fs::remove_dir(current).await.is_err() {
            break;
        }
        dir = current.parent();
    }
}
