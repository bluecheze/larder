use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;

use crate::devices::Device;
use crate::error::ApiError;
use crate::state::AppState;

/// Put this in a handler's arguments and it only runs for paired devices.
pub struct Authed(pub Device);

impl FromRequestParts<AppState> for Authed {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let secret = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| ApiError::Unauthorized("this device isn't paired".into()))?;
        state
            .devices
            .authenticate(secret)
            .map(Authed)
            .ok_or_else(|| ApiError::Unauthorized("this device isn't paired, or was removed".into()))
    }
}
