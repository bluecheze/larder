use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use larder_core::Invite;
use larder_host::devices::Device;
use larder_host::tls::{load_or_create, AcmeSettings};
use larder_host::{start, Host, HostConfig, RunningServer, Tls};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::config::{AppConfig, HostSettings};
use crate::network::lan_addresses;

const INVITE_LIFETIME: Duration = Duration::from_secs(15 * 60);

struct Running {
    host: Host,
    server: RunningServer,
    settings: HostSettings,
    fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HostStatus {
    pub running: bool,
    pub error: Option<String>,
    pub port: u16,
    pub addresses: Vec<String>,
    pub domain: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceView {
    pub id: String,
    pub name: String,
    pub paired_at: u64,
    pub last_seen: u64,
}

/// The host side of the app. Lives for the whole session and can be
/// started and stopped as the role changes.
pub struct HostRuntime {
    data_dir: PathBuf,
    running: Mutex<Option<Running>>,
    last_error: Mutex<Option<String>>,
}

impl HostRuntime {
    pub fn new(data_dir: PathBuf) -> Self {
        HostRuntime { data_dir, running: Mutex::new(None), last_error: Mutex::new(None) }
    }

    /// Starts serving if this computer is set up as a host. Stops whatever
    /// was running first, so calling it after a settings change is fine.
    pub async fn apply(&self, config: Option<&AppConfig>) {
        self.stop().await;
        let Some((config, settings)) = config.and_then(|c| c.host.as_ref().map(|s| (c, s))) else {
            return;
        };
        let result = self.launch(&config.device_name, settings.clone()).await;
        *self.last_error.lock().await = result.err().map(|e| e.to_string());
    }

    async fn launch(&self, name: &str, settings: HostSettings) -> io::Result<()> {
        let host = Host::open(HostConfig {
            name: name.to_string(),
            library: settings.library.clone(),
            data_dir: self.data_dir.clone(),
        })?;
        let (tls, fingerprint) = match (&settings.domain, &settings.email) {
            (Some(domain), Some(email)) => (
                Tls::Acme(AcmeSettings {
                    domain: domain.clone(),
                    email: email.clone(),
                    cache_dir: self.data_dir.join("acme"),
                    staging: false,
                }),
                None,
            ),
            _ => {
                let identity = load_or_create(&self.data_dir.join("tls"))?;
                let fingerprint = identity.fingerprint.clone();
                (Tls::SelfSigned(identity), Some(fingerprint))
            }
        };
        let addr = SocketAddr::from(([0, 0, 0, 0], settings.port));
        let server = start(host.router(), addr, tls).await?;
        *self.running.lock().await = Some(Running { host, server, settings, fingerprint });
        Ok(())
    }

    pub async fn stop(&self) {
        if let Some(running) = self.running.lock().await.take() {
            running.server.stop();
        }
    }

    pub async fn status(&self) -> HostStatus {
        let running = self.running.lock().await;
        let error = self.last_error.lock().await.clone();
        HostStatus {
            running: running.is_some(),
            error,
            port: running.as_ref().map_or(0, |r| r.settings.port),
            addresses: lan_addresses(),
            domain: running.as_ref().and_then(|r| r.settings.domain.clone()),
        }
    }

    /// `address` is whatever the user picked in the UI: a LAN IP, a public
    /// IP or a hostname. The port gets added here unless it's a domain,
    /// those are reached on 443 through the router.
    pub async fn create_invite(&self, address: &str) -> io::Result<String> {
        let running = self.running.lock().await;
        let running = running.as_ref().ok_or_else(|| io::Error::other("the host isn't running"))?;
        let address = address.trim();
        if address.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "pick an address first"));
        }
        let is_domain = running.settings.domain.as_deref() == Some(address);
        let address = if is_domain || address.contains(':') {
            address.to_string()
        } else {
            format!("{address}:{}", running.settings.port)
        };
        let token = running.host.create_invite_token(INVITE_LIFETIME);
        Ok(Invite { address, token, fingerprint: running.fingerprint.clone() }.to_string())
    }

    pub async fn devices(&self) -> Vec<DeviceView> {
        let running = self.running.lock().await;
        running.as_ref().map_or_else(Vec::new, |r| r.host.devices().into_iter().map(view).collect())
    }

    pub async fn revoke(&self, id: &str) -> io::Result<bool> {
        let running = self.running.lock().await;
        let running = running.as_ref().ok_or_else(|| io::Error::other("the host isn't running"))?;
        running.host.revoke(id)
    }
}

fn view(device: Device) -> DeviceView {
    DeviceView { id: device.id, name: device.name, paired_at: device.paired_at, last_seen: device.last_seen }
}
