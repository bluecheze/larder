use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;

use crate::devices::{Device, Devices};
use crate::index::Index;
use crate::routes::router;
use crate::shelves::{ShelfError, Shelves};
use crate::state::AppState;

pub struct HostConfig {
    pub name: String,
    pub library: PathBuf,
    pub data_dir: PathBuf,
}

/// Everything the desktop app needs from the host side. Device admin goes
/// through here directly, not over HTTP, so nobody on the network can
/// revoke your laptop.
#[derive(Clone)]
pub struct Host {
    state: AppState,
}

impl Host {
    pub fn open(config: HostConfig) -> io::Result<Self> {
        let shelves = Shelves::new(config.library.clone())?;
        let state = AppState {
            name: config.name.into(),
            devices: Arc::new(Devices::open(config.data_dir.join("devices.json"))?),
            shelves: Arc::new(shelves),
            index: Arc::new(Index::new(config.data_dir.join("index"))),
            tmp_dir: config.library.join(".larder").join("tmp"),
        };
        Ok(Host { state })
    }

    pub fn router(&self) -> Router {
        router(self.state.clone())
    }

    pub fn create_invite_token(&self, valid_for: Duration) -> String {
        self.state.devices.create_invite(valid_for)
    }

    pub fn devices(&self) -> Vec<Device> {
        self.state.devices.list()
    }

    pub fn revoke(&self, device_id: &str) -> io::Result<bool> {
        self.state.devices.revoke(device_id)
    }

    pub fn shelf_names(&self) -> io::Result<Vec<String>> {
        self.state.shelves.names()
    }

    pub fn create_shelf(&self, name: &str) -> Result<(), ShelfError> {
        self.state.shelves.create(name).map(|_| ())
    }
}
