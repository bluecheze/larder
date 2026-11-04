use std::io;
use std::path::{Path, PathBuf};

use larder_client::Connection;
use larder_core::fsutil::save_json;
use serde::{Deserialize, Serialize};

/// 5273 spells LARD on a phone keypad, and nothing common sits on it.
pub const DEFAULT_PORT: u16 = 5273;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Host,
    Client,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostSettings {
    pub library: PathBuf,
    pub port: u16,
    pub domain: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientSettings {
    pub connection: Connection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub role: Role,
    pub device_name: String,
    pub host: Option<HostSettings>,
    pub client: Option<ClientSettings>,
}

impl AppConfig {
    /// What the UI gets to see. The device token stays on the Rust side,
    /// the webview has no reason to ever hold it.
    pub fn for_ui(&self) -> AppConfig {
        let mut copy = self.clone();
        if let Some(client) = copy.client.as_mut() {
            client.connection.token.clear();
        }
        copy
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Defaults {
    pub device_name: String,
    pub library: PathBuf,
    pub port: u16,
}

/// Where the app keeps its own settings. No config file at all means
/// setup hasn't run yet, which is how first launch is detected.
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(dir: &Path) -> Self {
        ConfigStore { path: dir.join("config.json") }
    }

    /// A config we can't read (an older format, a hand edit gone wrong)
    /// sends you back through setup. Nothing in the library is touched by
    /// that, so it's the friendlier failure.
    pub fn load(&self) -> io::Result<Option<AppConfig>> {
        match std::fs::read(&self.path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes).ok()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub fn save(&self, config: &AppConfig) -> io::Result<()> {
        validate(config)?;
        save_json(&self.path, config)
    }

    /// Only forgets the role and settings. The library and any synced
    /// files stay on disk, switching roles should never delete things.
    pub fn reset(&self) -> io::Result<()> {
        match std::fs::remove_file(&self.path) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
            _ => Ok(()),
        }
    }
}

fn validate(config: &AppConfig) -> io::Result<()> {
    let ok = match config.role {
        Role::Host => config.host.is_some(),
        Role::Client => config.client.is_some(),
    };
    if ok && !config.device_name.trim().is_empty() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "setup isn't finished"))
    }
}

pub fn defaults(home: &Path) -> Defaults {
    let name = gethostname::gethostname().to_string_lossy().trim_end_matches(".local").to_string();
    Defaults {
        device_name: if name.is_empty() { "My computer".into() } else { name },
        library: home.join("Larder"),
        port: DEFAULT_PORT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host_config() -> AppConfig {
        AppConfig {
            role: Role::Host,
            device_name: "studio".into(),
            host: Some(HostSettings { library: "/lib".into(), port: DEFAULT_PORT, domain: None, email: None }),
            client: None,
        }
    }

    #[test]
    fn first_launch_has_no_config() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(ConfigStore::new(dir.path()).load().unwrap(), None);
    }

    #[test]
    fn save_load_reset() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::new(dir.path());
        store.save(&host_config()).unwrap();
        assert_eq!(store.load().unwrap(), Some(host_config()));
        store.reset().unwrap();
        assert_eq!(store.load().unwrap(), None);
        store.reset().unwrap();
    }

    #[test]
    fn unreadable_config_means_setup_again() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("config.json"), r#"{"role":"client","client":{"invite":"old"}}"#).unwrap();
        assert_eq!(ConfigStore::new(dir.path()).load().unwrap(), None);
    }

    #[test]
    fn half_finished_setup_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::new(dir.path());
        let broken = AppConfig { host: None, ..host_config() };
        assert!(store.save(&broken).is_err());
        let nameless = AppConfig { device_name: "  ".into(), ..host_config() };
        assert!(store.save(&nameless).is_err());
    }

    #[test]
    fn ui_never_sees_the_device_token() {
        let config = AppConfig {
            role: Role::Client,
            device_name: "laptop".into(),
            host: None,
            client: Some(ClientSettings {
                connection: Connection {
                    address: "pc:5273".into(),
                    fingerprint: None,
                    host_name: "pc".into(),
                    device_id: "d1".into(),
                    token: "secret".into(),
                },
            }),
        };
        assert_eq!(config.for_ui().client.unwrap().connection.token, "");
        assert_eq!(config.client.unwrap().connection.token, "secret");
    }

    #[test]
    fn role_serializes_lowercase_for_the_ui() {
        let json = serde_json::to_string(&host_config()).unwrap();
        assert!(json.contains(r#""role":"host""#));
    }
}
