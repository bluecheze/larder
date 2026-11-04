mod pairing;
mod read;
mod shelves;
mod write;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let files = get(read::get_file).put(write::put_file).delete(write::delete_file);
    Router::new()
        .route("/v1/info", get(pairing::info))
        .route("/v1/pair", post(pairing::pair))
        .route("/v1/shelves", get(shelves::list).post(shelves::create))
        .route("/v1/shelves/{shelf}/manifest", get(shelves::manifest))
        .route("/v1/shelves/{shelf}/move", post(write::move_file))
        .route("/v1/shelves/{shelf}/files/{*path}", files)
        // Uploads are streamed to disk, so the default 2 MB cap only gets
        // in the way. Big files are the whole point of a drive.
        .layer(DefaultBodyLimit::disable())
        .with_state(state)
}
