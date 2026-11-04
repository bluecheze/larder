use std::io;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::devices::PairError;
use crate::shelves::ShelfError;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    /// Carries the hash the host has right now, if there is a file at all.
    Conflict(Option<String>),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error, current) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg, None),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg, None),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg, None),
            ApiError::Conflict(current) => {
                (StatusCode::CONFLICT, "the file changed on the host since you last synced".into(), current)
            }
            ApiError::Internal(msg) => {
                // The real reason goes to the host's log. Clients just get a
                // generic message, no need to tell them about our disks.
                tracing::error!("{msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong on the host".into(), None)
            }
        };
        let body = larder_core::protocol::ApiError { error, current };
        (status, Json(body)).into_response()
    }
}

impl From<io::Error> for ApiError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => ApiError::NotFound("no such file".into()),
            _ => ApiError::Internal(err.to_string()),
        }
    }
}

impl From<ShelfError> for ApiError {
    fn from(err: ShelfError) -> Self {
        match err {
            ShelfError::BadName(_) => ApiError::BadRequest(err.to_string()),
            ShelfError::Missing(_) => ApiError::NotFound(err.to_string()),
            ShelfError::Io(io) => io.into(),
        }
    }
}

impl From<larder_core::Error> for ApiError {
    fn from(err: larder_core::Error) -> Self {
        match err {
            larder_core::Error::InvalidPath(_) => ApiError::BadRequest(err.to_string()),
            larder_core::Error::Io(io) => io.into(),
        }
    }
}

impl From<PairError> for ApiError {
    fn from(err: PairError) -> Self {
        match err {
            PairError::BadInvite => ApiError::Unauthorized(err.to_string()),
            PairError::Io(io) => io.into(),
        }
    }
}
