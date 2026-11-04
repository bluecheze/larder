use serde::{Deserialize, Serialize};

/// Everything a client remembers about its host after pairing. The token
/// is the device's password, so this gets stored like one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub address: String,
    pub fingerprint: Option<String>,
    pub host_name: String,
    pub device_id: String,
    pub token: String,
}
