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

pub const DEFAULT_CHECK_SECS: u64 = 30;

fn default_check() -> u64 {
    DEFAULT_CHECK_SECS
}

fn yes() -> bool {
    true
}

fn system() -> String {
    "system".into()
}

fn cobalt() -> String {
    "cobalt".into()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientSettings {
    pub connection: Connection,
    /// None means the default, ~/Larder.
    #[serde(default)]
    pub folder: Option<PathBuf>,
    /// Shelves kept on this computer and synced both ways.
    #[serde(default)]
    pub synced: Vec<String>,
    /// How often to ask the host for changes made on other computers.
    /// Local changes don't wait for this, the folder watcher catches those.
    #[serde(default = "default_check")]
    pub check_every_secs: u64,
}

/// Things people tweak. Every field has a default so configs written by
/// older versions still load.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prefs {
    #[serde(default = "system")]
    pub theme: String,
    #[serde(default = "cobalt")]
    pub accent: String,
    #[serde(default = "yes")]
    pub show_in_sidebar: bool,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub start_at_login: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs { theme: system(), accent: cobalt(), show_in_sidebar: true, ignore: vec![], start_at_login: false }
    }
}

pub const THEMES: [&str; 3] = ["system", "light", "dark"];
pub const ACCENTS: [&str; 4] = ["cobalt", "forest", "rust", "graphite"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub role: Role,
    pub device_name: String,
    pub host: Option<HostSettings>,
    pub client: Option<ClientSettings>,
    #[serde(default)]
    pub prefs: Prefs,
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

    /// Applies an edit that came back from the Settings page. The UI only
    /// ever saw a redacted copy, so the connection (with its token) and the
    /// role always come from what's on disk. Changing role goes through
    /// setup, not through here.
    pub fn with_edit(&self, edit: AppConfig) -> AppConfig {
        let client = match (&self.client, edit.client) {
            (Some(stored), Some(edited)) => Some(ClientSettings { connection: stored.connection.clone(), ..edited }),
            (stored, _) => stored.clone(),
        };
        let host = if self.host.is_some() { edit.host.or_else(|| self.host.clone()) } else { None };
        AppConfig { role: self.role.clone(), device_name: edit.device_name, host, client, prefs: edit.prefs }
    }

    pub fn larder_folder(&self, home: &Path) -> Option<PathBuf> {
        let client = self.client.as_ref()?;
        Some(client.folder.clone().unwrap_or_else(|| home.join("Larder")))
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
    let invalid = |msg: &str| Err(io::Error::new(io::ErrorKind::InvalidInput, msg.to_string()));
    if !ok || config.device_name.trim().is_empty() {
        return invalid("setup isn't finished");
    }
    if !THEMES.contains(&config.prefs.theme.as_str()) || !ACCENTS.contains(&config.prefs.accent.as_str()) {
        return invalid("unknown theme or accent");
    }
    if config.host.as_ref().is_some_and(|h| h.port < 1024) {
        return invalid("use a port between 1024 and 65535");
    }
    if let Some(client) = &config.client {
        if !(10..=3600).contains(&client.check_every_secs) {
            return invalid("check for changes somewhere between every 10 seconds and every hour");
        }
    }
    Ok(())
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
            prefs: Prefs::default(),
        }
    }

    fn client_config(token: &str) -> AppConfig {
        AppConfig {
            role: Role::Client,
            device_name: "laptop".into(),
            host: None,
            client: Some(ClientSettings {
                connection: Connection {
                    address: "pc:5273".into(),
                    fingerprint: None,
                    host_name: "pc".into(),
                    device_id: "d1".into(),
                    token: token.into(),
                },
                folder: None,
                synced: vec![],
                check_every_secs: DEFAULT_CHECK_SECS,
            }),
            prefs: Prefs::default(),
        }
    }

    #[test]
    fn settings_edits_keep_the_secret_token() {
        let stored = client_config("secret");
        let mut edit = stored.for_ui();
        edit.device_name = "renamed".into();
        edit.client.as_mut().unwrap().synced = vec!["Photos".into()];
        edit.prefs.theme = "dark".into();
        let merged = stored.with_edit(edit);
        assert_eq!(merged.client.as_ref().unwrap().connection.token, "secret");
        assert_eq!(merged.client.as_ref().unwrap().synced, ["Photos"]);
        assert_eq!(merged.device_name, "renamed");
        assert_eq!(merged.prefs.theme, "dark");
    }

    #[test]
    fn edits_cant_flip_the_role() {
        let stored = host_config();
        let mut edit = client_config("x");
        edit.device_name = "studio".into();
        let merged = stored.with_edit(edit);
        assert_eq!(merged.role, Role::Host);
        assert!(merged.client.is_none());
        assert!(merged.host.is_some());
    }

    #[test]
    fn old_configs_get_default_prefs() {
        let dir = tempfile::tempdir().unwrap();
        let json = r#"{"role":"host","device_name":"pc","host":{"library":"/l","port":5273,"domain":null,"email":null},"client":null}"#;
        std::fs::write(dir.path().join("config.json"), json).unwrap();
        let loaded = ConfigStore::new(dir.path()).load().unwrap().unwrap();
        assert_eq!(loaded.prefs, Prefs::default());
    }

    #[test]
    fn silly_values_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::new(dir.path());
        let mut config = client_config("t");
        config.client.as_mut().unwrap().check_every_secs = 1;
        assert!(store.save(&config).is_err());
        let mut config = host_config();
        config.prefs.theme = "neon".into();
        assert!(store.save(&config).is_err());
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
        let config = client_config("secret");
        assert_eq!(config.for_ui().client.unwrap().connection.token, "");
        assert_eq!(config.client.unwrap().connection.token, "secret");
    }

    #[test]
    fn role_serializes_lowercase_for_the_ui() {
        let json = serde_json::to_string(&host_config()).unwrap();
        assert!(json.contains(r#""role":"host""#));
    }
}
