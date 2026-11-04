use std::io;
use std::path::{Path, PathBuf};

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
    pub invite: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub role: Role,
    pub device_name: String,
    pub host: Option<HostSettings>,
    pub client: Option<ClientSettings>,
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

    pub fn load(&self) -> io::Result<Option<AppConfig>> {
        match std::fs::read(&self.path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map(Some).map_err(io::Error::other),
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
    fn half_finished_setup_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::new(dir.path());
        let broken = AppConfig { host: None, ..host_config() };
        assert!(store.save(&broken).is_err());
        let nameless = AppConfig { device_name: "  ".into(), ..host_config() };
        assert!(store.save(&nameless).is_err());
    }

    #[test]
    fn role_serializes_lowercase_for_the_ui() {
        let json = serde_json::to_string(&host_config()).unwrap();
        assert!(json.contains(r#""role":"host""#));
    }
}
