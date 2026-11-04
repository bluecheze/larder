use axum::extract::{Path, State};
use axum::Json;
use larder_core::protocol::{NewShelf, Shelf};
use larder_core::Manifest;

use crate::auth::Authed;
use crate::error::ApiError;
use crate::state::AppState;

/// File counts come from the cached index, not a fresh scan. Listing
/// shelves should be instant even with a big library.
pub async fn list(State(state): State<AppState>, _auth: Authed) -> Result<Json<Vec<Shelf>>, ApiError> {
    let mut out = Vec::new();
    for name in state.shelves.names()? {
        let guard = state.index.lock(&name).await?;
        let manifest = guard.as_ref();
        out.push(Shelf {
            files: manifest.map_or(0, |m| m.len() as u64),
            bytes: manifest.map_or(0, |m| m.values().map(|e| e.size).sum()),
            name,
        });
    }
    Ok(Json(out))
}

pub async fn create(
    State(state): State<AppState>,
    _auth: Authed,
    Json(req): Json<NewShelf>,
) -> Result<Json<Shelf>, ApiError> {
    state.shelves.create(&req.name)?;
    Ok(Json(Shelf { name: req.name.trim().to_string(), files: 0, bytes: 0 }))
}

pub async fn manifest(
    State(state): State<AppState>,
    _auth: Authed,
    Path(shelf): Path<String>,
) -> Result<Json<Manifest>, ApiError> {
    let root = state.shelves.dir(&shelf)?;
    Ok(Json(state.index.refresh(&shelf, root).await?))
}
