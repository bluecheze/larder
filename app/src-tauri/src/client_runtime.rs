use larder_client::{pair, ClientError, HostClient};
use larder_core::protocol::Shelf;
use larder_core::Invite;
use serde::Serialize;
use tokio::sync::Mutex;

use crate::config::{AppConfig, ClientSettings, Role};

#[derive(Debug, Clone, Serialize)]
pub struct ClientStatus {
    pub connected: bool,
    pub host_name: Option<String>,
    pub address: Option<String>,
    pub error: Option<String>,
    pub shelves: Vec<Shelf>,
}

/// The client side of the app: who we're paired with and a ready-made
/// HTTP client for talking to them.
pub struct ClientRuntime {
    current: Mutex<Option<(HostClient, ClientSettings)>>,
}

impl ClientRuntime {
    pub fn new() -> Self {
        ClientRuntime { current: Mutex::new(None) }
    }

    pub async fn apply(&self, config: Option<&AppConfig>) {
        let next = config
            .and_then(|c| c.client.clone())
            .and_then(|settings| HostClient::for_connection(&settings.connection).ok().map(|c| (c, settings)));
        *self.current.lock().await = next;
    }

    /// Pairs using an invite and returns the finished config. Nothing is
    /// saved if pairing fails, so a bad invite just means trying again.
    pub async fn join(invite: &str, device_name: &str) -> Result<AppConfig, ClientError> {
        let invite: Invite = invite
            .parse()
            .map_err(|e: larder_core::invite::InviteError| ClientError::Refused(e.to_string()))?;
        let connection = pair(&invite, device_name.trim()).await?;
        Ok(AppConfig {
            role: Role::Client,
            device_name: device_name.trim().to_string(),
            host: None,
            client: Some(ClientSettings { connection }),
        })
    }

    pub async fn status(&self) -> ClientStatus {
        let current = self.current.lock().await;
        let Some((client, settings)) = current.as_ref() else {
            return ClientStatus { connected: false, host_name: None, address: None, error: None, shelves: vec![] };
        };
        let (shelves, error) = match client.shelves().await {
            Ok(shelves) => (shelves, None),
            Err(err) => (vec![], Some(err.to_string())),
        };
        ClientStatus {
            connected: error.is_none(),
            host_name: Some(settings.connection.host_name.clone()),
            address: Some(settings.connection.address.clone()),
            error,
            shelves,
        }
    }
}
