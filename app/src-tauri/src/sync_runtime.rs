use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use larder_client::sync::{sync_shelf, watch, Change, ChangeKind, StateFile, SyncJob, Watch};
use larder_client::HostClient;
use serde::Serialize;
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;

use crate::config::AppConfig;

/// Waits this long after the last local change before syncing. Saving a
/// file in most editors is several writes in a row, no point syncing each.
const SETTLE: Duration = Duration::from_secs(2);
const KEEP_ACTIVITY: usize = 200;

#[derive(Debug, Clone, Serialize)]
pub struct ShelfSync {
    pub state: &'static str,
    pub last_sync: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Activity {
    pub at: u64,
    pub shelf: String,
    pub path: String,
    pub kind: ChangeKind,
}

type Status = Arc<StdMutex<BTreeMap<String, ShelfSync>>>;
type Log = Arc<StdMutex<VecDeque<Activity>>>;

struct Loop {
    handle: JoinHandle<()>,
    poke: Arc<Notify>,
}

/// Keeps every "keep on this computer" shelf in sync, one loop per shelf.
pub struct SyncRuntime {
    state_dir: PathBuf,
    loops: Mutex<BTreeMap<String, Loop>>,
    status: Status,
    log: Log,
}

impl SyncRuntime {
    pub fn new(state_dir: PathBuf) -> Self {
        SyncRuntime { state_dir, loops: Mutex::new(BTreeMap::new()), status: Status::default(), log: Log::default() }
    }

    /// Tears down every loop and starts the ones the config asks for. Cheap
    /// enough to call on every settings change.
    pub async fn apply(&self, config: Option<&AppConfig>, home: &Path) {
        let mut loops = self.loops.lock().await;
        for (_, running) in std::mem::take(&mut *loops) {
            running.handle.abort();
        }
        self.status.lock().unwrap().clear();
        let Some(config) = config else { return };
        let (Some(client), Some(folder)) = (config.client.as_ref(), config.larder_folder(home)) else { return };
        let Ok(http) = HostClient::for_connection(&client.connection) else { return };
        for shelf in &client.synced {
            let poke = Arc::new(Notify::new());
            let job = ShelfLoop {
                client: http.clone(),
                shelf: shelf.clone(),
                root: folder.join(shelf),
                device: config.device_name.clone(),
                ignore: config.prefs.ignore.clone(),
                every: Duration::from_secs(client.check_every_secs),
                state_dir: self.state_dir.clone(),
                status: self.status.clone(),
                log: self.log.clone(),
                poke: poke.clone(),
            };
            loops.insert(shelf.clone(), Loop { handle: tokio::spawn(job.run()), poke });
        }
    }

    pub async fn sync_now(&self, shelf: &str) {
        if let Some(running) = self.loops.lock().await.get(shelf) {
            running.poke.notify_one();
        }
    }

    pub fn status(&self) -> BTreeMap<String, ShelfSync> {
        self.status.lock().unwrap().clone()
    }

    pub fn activity(&self) -> Vec<Activity> {
        self.log.lock().unwrap().iter().rev().cloned().collect()
    }
}

struct ShelfLoop {
    client: HostClient,
    shelf: String,
    root: PathBuf,
    device: String,
    ignore: Vec<String>,
    every: Duration,
    state_dir: PathBuf,
    status: Status,
    log: Log,
    poke: Arc<Notify>,
}

impl ShelfLoop {
    async fn run(self) {
        let file = StateFile::new(&self.state_dir, &self.shelf, &self.root);
        let mut state = file.load().unwrap_or_default();
        let _ = std::fs::create_dir_all(&self.root);
        // No watcher (some network drives can't be watched) just means we
        // fall back to the timer for local changes too.
        let mut watcher = watch(&self.root).ok();
        loop {
            self.set("syncing", None);
            let job = SyncJob {
                client: &self.client,
                shelf: &self.shelf,
                root: &self.root,
                device: &self.device,
                ignore: &self.ignore,
            };
            let result = sync_shelf(&job, &mut state).await;
            let _ = file.save(&state);
            match result {
                Ok(changes) => {
                    self.record(changes);
                    self.set("idle", None);
                }
                Err(err) => self.set("error", Some(err.to_string())),
            }
            tokio::select! {
                _ = tokio::time::sleep(self.every) => {}
                _ = self.poke.notified() => {}
                _ = next_change(&mut watcher) => {
                    tokio::time::sleep(SETTLE).await;
                    drain(&mut watcher);
                }
            }
        }
    }

    fn set(&self, state: &'static str, error: Option<String>) {
        let mut all = self.status.lock().unwrap();
        let last_sync = match state {
            "idle" => Some(now()),
            _ => all.get(&self.shelf).and_then(|s| s.last_sync),
        };
        all.insert(self.shelf.clone(), ShelfSync { state, last_sync, error });
    }

    fn record(&self, changes: Vec<Change>) {
        let mut log = self.log.lock().unwrap();
        for change in changes {
            log.push_back(Activity { at: now(), shelf: change.shelf, path: change.path, kind: change.kind });
            if log.len() > KEEP_ACTIVITY {
                log.pop_front();
            }
        }
    }
}

async fn next_change(watcher: &mut Option<Watch>) {
    match watcher {
        Some(w) => {
            w.changes.recv().await;
        }
        None => std::future::pending().await,
    }
}

fn drain(watcher: &mut Option<Watch>) {
    if let Some(w) = watcher {
        while w.changes.try_recv().is_ok() {}
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}
