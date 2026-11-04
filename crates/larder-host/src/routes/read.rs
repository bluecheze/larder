use std::io::SeekFrom;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use larder_core::RelPath;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use crate::auth::Authed;
use crate::error::ApiError;
use crate::range::{parse_range, RangeRequest};
use crate::state::AppState;

/// Streams a file, or part of one. The client drive asks for ranges a lot,
/// a video player seeking around is basically nothing but range requests.
pub async fn get_file(
    State(state): State<AppState>,
    _auth: Authed,
    Path((shelf, raw)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let path = RelPath::parse(&raw)?;
    let full = path.to_fs(&state.shelves.dir(&shelf)?);
    let mut file = tokio::fs::File::open(&full).await?;
    let size = file.metadata().await?.len();
    let range = headers.get(header::RANGE).and_then(|v| v.to_str().ok());

    let (status, start, len) = match parse_range(range, size) {
        RangeRequest::Full => (StatusCode::OK, 0, size),
        RangeRequest::Partial(r) => (StatusCode::PARTIAL_CONTENT, r.start, r.end - r.start + 1),
        RangeRequest::Unsatisfiable => {
            let mut res = StatusCode::RANGE_NOT_SATISFIABLE.into_response();
            res.headers_mut().insert(header::CONTENT_RANGE, value(format!("bytes */{size}")));
            return Ok(res);
        }
    };
    file.seek(SeekFrom::Start(start)).await?;
    let body = Body::from_stream(ReaderStream::new(file.take(len)));

    let mut res = (status, body).into_response();
    let out = res.headers_mut();
    out.insert(header::CONTENT_LENGTH, value(len.to_string()));
    out.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    out.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
    if status == StatusCode::PARTIAL_CONTENT {
        out.insert(header::CONTENT_RANGE, value(format!("bytes {start}-{}/{size}", start + len - 1)));
    }
    Ok(res)
}

fn value(text: String) -> HeaderValue {
    HeaderValue::from_str(&text).expect("ascii header value")
}
