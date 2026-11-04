use serde::{Deserialize, Serialize};

/// Bumped whenever the HTTP API changes in a way old clients can't handle.
pub const API_VERSION: u32 = 1;

/// Hash the client expects the host to have before an upload or delete.
/// Empty means "I think this file doesn't exist on the host yet".
pub const HEADER_BASE: &str = "x-larder-base";

/// blake3 of the body being uploaded, checked by the host before the file
/// is moved into place.
pub const HEADER_HASH: &str = "x-larder-hash";

/// Modified time to stamp on the file, in unix millis.
pub const HEADER_MODIFIED: &str = "x-larder-modified";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    pub name: String,
    pub api_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairRequest {
    pub token: String,
    pub device_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairResponse {
    pub device_id: String,
    /// Long-lived secret for this device. The host only keeps its hash.
    pub device_token: String,
    pub host_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shelf {
    pub name: String,
    pub files: u64,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewShelf {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
}
