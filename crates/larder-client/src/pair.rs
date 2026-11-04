use larder_core::Invite;

use crate::api::HostClient;
use crate::connection::Connection;
use crate::error::ClientError;

/// Redeems an invite. Checks the host is reachable and speaks our version
/// before spending the one-time token, so a typo in the address doesn't
/// burn the invite.
pub async fn pair(invite: &Invite, device_name: &str) -> Result<Connection, ClientError> {
    let fingerprint = invite.fingerprint.as_deref();
    let client = HostClient::new(&invite.address, fingerprint, None)?;
    client.info().await?;
    let paired = client.pair(&invite.token, device_name).await.map_err(|err| match err {
        ClientError::Unpaired => ClientError::BadInvite,
        other => other,
    })?;
    Ok(Connection {
        address: invite.address.clone(),
        fingerprint: invite.fingerprint.clone(),
        host_name: paired.host_name,
        device_id: paired.device_id,
        token: paired.device_token,
    })
}

impl HostClient {
    pub fn for_connection(connection: &Connection) -> Result<Self, ClientError> {
        HostClient::new(&connection.address, connection.fingerprint.as_deref(), Some(connection.token.clone()))
    }
}
