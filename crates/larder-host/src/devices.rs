use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use larder_core::fsutil::{load_json, save_json};
use serde::{Deserialize, Serialize};

use crate::secret::{hash_secret, new_secret, now_secs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub paired_at: u64,
    pub last_seen: u64,
    #[serde(rename = "token_hash")]
    pub(crate) token_hash: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PairError {
    #[error("that invite is wrong or has expired")]
    BadInvite,
    #[error("couldn't save the device list: {0}")]
    Io(#[from] io::Error),
}

/// Paired devices live on disk. Open invites only live in memory on purpose:
/// restarting the host is an easy way to kill any invite you handed out.
pub struct Devices {
    path: PathBuf,
    inner: Mutex<Inner>,
}

struct Inner {
    devices: Vec<Device>,
    invites: HashMap<String, Instant>,
}

impl Devices {
    pub fn open(path: PathBuf) -> io::Result<Self> {
        let devices = load_json(&path)?;
        Ok(Devices { path, inner: Mutex::new(Inner { devices, invites: HashMap::new() }) })
    }

    /// Returns the one-time token that goes into an invite.
    pub fn create_invite(&self, valid_for: Duration) -> String {
        let token = new_secret(12);
        let mut inner = self.inner.lock().unwrap();
        inner.invites.retain(|_, expires| *expires > Instant::now());
        inner.invites.insert(hash_secret(&token), Instant::now() + valid_for);
        token
    }

    pub fn pair(&self, token: &str, name: &str) -> Result<(Device, String), PairError> {
        let mut inner = self.inner.lock().unwrap();
        // remove() makes the token single use, even if pairing fails below.
        let expires = inner.invites.remove(&hash_secret(token)).ok_or(PairError::BadInvite)?;
        if expires < Instant::now() {
            return Err(PairError::BadInvite);
        }
        let secret = new_secret(32);
        let now = now_secs();
        let device = Device {
            id: new_secret(8),
            name: clean_name(name),
            paired_at: now,
            last_seen: now,
            token_hash: hash_secret(&secret),
        };
        inner.devices.push(device.clone());
        save_json(&self.path, &inner.devices)?;
        Ok((device, secret))
    }

    /// last_seen is only kept in memory here and written out the next time
    /// the list is saved. Writing a file on every request isn't worth it.
    pub fn authenticate(&self, secret: &str) -> Option<Device> {
        let hash = hash_secret(secret);
        let mut inner = self.inner.lock().unwrap();
        let device = inner.devices.iter_mut().find(|d| d.token_hash == hash)?;
        device.last_seen = now_secs();
        Some(device.clone())
    }

    pub fn list(&self) -> Vec<Device> {
        self.inner.lock().unwrap().devices.clone()
    }

    pub fn revoke(&self, id: &str) -> io::Result<bool> {
        let mut inner = self.inner.lock().unwrap();
        let before = inner.devices.len();
        inner.devices.retain(|d| d.id != id);
        if inner.devices.len() == before {
            return Ok(false);
        }
        save_json(&self.path, &inner.devices)?;
        Ok(true)
    }
}

fn clean_name(name: &str) -> String {
    let trimmed: String = name.trim().chars().filter(|c| !c.is_control()).take(48).collect();
    if trimmed.is_empty() { "Unnamed device".into() } else { trimmed }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (tempfile::TempDir, Devices) {
        let dir = tempfile::tempdir().unwrap();
        let devices = Devices::open(dir.path().join("devices.json")).unwrap();
        (dir, devices)
    }

    #[test]
    fn invite_pairs_once() {
        let (_dir, devices) = store();
        let token = devices.create_invite(Duration::from_secs(60));
        let (device, secret) = devices.pair(&token, "  laptop ").unwrap();
        assert_eq!(device.name, "laptop");
        assert!(devices.authenticate(&secret).is_some());
        assert!(matches!(devices.pair(&token, "again"), Err(PairError::BadInvite)));
    }

    #[test]
    fn expired_invites_fail() {
        let (_dir, devices) = store();
        let token = devices.create_invite(Duration::ZERO);
        std::thread::sleep(Duration::from_millis(5));
        assert!(matches!(devices.pair(&token, "x"), Err(PairError::BadInvite)));
    }

    #[test]
    fn devices_survive_a_restart_and_can_be_revoked() {
        let (dir, devices) = store();
        let token = devices.create_invite(Duration::from_secs(60));
        let (device, secret) = devices.pair(&token, "pc").unwrap();
        drop(devices);
        let reopened = Devices::open(dir.path().join("devices.json")).unwrap();
        assert!(reopened.authenticate(&secret).is_some());
        assert!(reopened.revoke(&device.id).unwrap());
        assert!(reopened.authenticate(&secret).is_none());
        assert!(!reopened.revoke(&device.id).unwrap());
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let (_dir, devices) = store();
        assert!(devices.authenticate("nope").is_none());
    }
}
