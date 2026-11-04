use axum::extract::State;
use axum::Json;
use larder_core::protocol::{HostInfo, PairRequest, PairResponse, API_VERSION};

use crate::error::ApiError;
use crate::state::AppState;

/// The only route that works without pairing. Clients hit it first to
/// check they've got the right machine and a compatible version.
pub async fn info(State(state): State<AppState>) -> Json<HostInfo> {
    Json(HostInfo { name: state.name.to_string(), api_version: API_VERSION })
}

pub async fn pair(
    State(state): State<AppState>,
    Json(req): Json<PairRequest>,
) -> Result<Json<PairResponse>, ApiError> {
    let devices = state.devices.clone();
    // pair() writes the device list to disk, keep that off the async threads.
    let (device, secret) = tokio::task::spawn_blocking(move || devices.pair(&req.token, &req.device_name))
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))??;
    tracing::info!(device = %device.name, "paired a new device");
    Ok(Json(PairResponse { device_id: device.id, device_token: secret, host_name: state.name.to_string() }))
}
