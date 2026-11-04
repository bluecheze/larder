use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use larder_client::sync::{sync_shelf, ChangeKind, ShelfState, SyncJob};
use larder_client::{pair, HostClient};
use larder_core::Invite;
use larder_host::tls::load_or_create;
use larder_host::{start, Host, HostConfig, RunningServer, Tls};

struct Machine {
    name: &'static str,
    client: HostClient,
    dir: tempfile::TempDir,
    state: ShelfState,
}

impl Machine {
    fn path(&self, rel: &str) -> PathBuf {
        self.dir.path().join(rel)
    }

    fn write(&self, rel: &str, text: &str) {
        let path = self.path(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn read(&self, rel: &str) -> Option<String> {
        fs::read_to_string(self.path(rel)).ok()
    }

    async fn sync(&mut self) -> Vec<ChangeKind> {
        self.sync_ignoring(&[]).await
    }

    async fn sync_ignoring(&mut self, ignore: &[String]) -> Vec<ChangeKind> {
        let job = SyncJob { client: &self.client, shelf: "docs", root: self.dir.path(), device: self.name, ignore };
        sync_shelf(&job, &mut self.state).await.unwrap().into_iter().map(|c| c.kind).collect()
    }
}

struct World {
    _host_dir: tempfile::TempDir,
    _server: RunningServer,
    library: PathBuf,
    a: Machine,
    b: Machine,
}

async fn world() -> World {
    let host_dir = tempfile::tempdir().unwrap();
    let library = host_dir.path().join("library");
    let host = Host::open(HostConfig { name: "host".into(), library: library.clone(), data_dir: host_dir.path().join("data") })
        .unwrap();
    host.create_shelf("docs").unwrap();
    let identity = load_or_create(&host_dir.path().join("data/tls")).unwrap();
    let fingerprint = identity.fingerprint.clone();
    let server = start(host.router(), "127.0.0.1:0".parse().unwrap(), Tls::SelfSigned(identity)).await.unwrap();
    let mut machines = Vec::new();
    for name in ["laptop", "desktop"] {
        let invite = Invite {
            address: server.addr.to_string(),
            token: host.create_invite_token(Duration::from_secs(60)),
            fingerprint: Some(fingerprint.clone()),
        };
        let connection = pair(&invite, name).await.unwrap();
        machines.push(Machine {
            name,
            client: HostClient::for_connection(&connection).unwrap(),
            dir: tempfile::tempdir().unwrap(),
            state: ShelfState::default(),
        });
    }
    let b = machines.pop().unwrap();
    let a = machines.pop().unwrap();
    World { _host_dir: host_dir, _server: server, library, a, b }
}

fn files_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    names.sort();
    names
}

#[tokio::test]
async fn new_files_travel_both_ways() {
    let mut w = world().await;
    w.a.write("notes/plan.md", "from a");
    assert_eq!(w.a.sync().await, [ChangeKind::Uploaded]);
    assert_eq!(fs::read_to_string(w.library.join("docs/notes/plan.md")).unwrap(), "from a");

    assert_eq!(w.b.sync().await, [ChangeKind::Downloaded]);
    assert_eq!(w.b.read("notes/plan.md").as_deref(), Some("from a"));

    w.b.write("reply.txt", "from b");
    w.b.sync().await;
    w.a.sync().await;
    assert_eq!(w.a.read("reply.txt").as_deref(), Some("from b"));
    assert!(w.a.sync().await.is_empty(), "a second pass should have nothing to do");
}

#[tokio::test]
async fn edits_and_deletes_propagate() {
    let mut w = world().await;
    w.a.write("f.txt", "v1");
    w.a.sync().await;
    w.b.sync().await;

    w.b.write("f.txt", "v2");
    assert_eq!(w.b.sync().await, [ChangeKind::Uploaded]);
    assert_eq!(w.a.sync().await, [ChangeKind::Downloaded]);
    assert_eq!(w.a.read("f.txt").as_deref(), Some("v2"));

    fs::remove_file(w.a.path("f.txt")).unwrap();
    assert_eq!(w.a.sync().await, [ChangeKind::Deleted]);
    assert_eq!(w.b.sync().await, [ChangeKind::Removed]);
    assert_eq!(w.b.read("f.txt"), None);
}

#[tokio::test]
async fn offline_edits_on_both_sides_keep_both() {
    let mut w = world().await;
    w.a.write("shared.txt", "original");
    w.a.sync().await;
    w.b.sync().await;

    w.a.write("shared.txt", "a's version");
    w.b.write("shared.txt", "b's version");
    w.a.sync().await;
    assert_eq!(w.b.sync().await, [ChangeKind::Conflict]);
    w.b.sync().await;
    w.a.sync().await;

    for machine in [&w.a, &w.b] {
        assert_eq!(machine.read("shared.txt").as_deref(), Some("a's version"));
        let names = files_in(machine.dir.path());
        let copy = names.iter().find(|n| n.contains("(conflict, desktop,")).expect("conflict copy exists");
        assert_eq!(machine.read(copy).as_deref(), Some("b's version"));
    }
}

#[tokio::test]
async fn deleting_the_last_file_cleans_up_folders() {
    let mut w = world().await;
    w.a.write("deep/er/x.txt", "x");
    w.a.sync().await;
    w.b.sync().await;
    fs::remove_file(w.a.path("deep/er/x.txt")).unwrap();
    w.a.sync().await;
    w.b.sync().await;
    assert!(!w.b.path("deep").exists());
}

#[tokio::test]
async fn ignoring_synced_files_never_deletes_them() {
    let mut w = world().await;
    w.a.write("art.psd", "layers");
    w.a.write("cache/blob.bin", "junk");
    w.a.sync().await;
    let ignore = vec!["*.psd".to_string(), "cache".to_string()];
    assert!(w.a.sync_ignoring(&ignore).await.is_empty());
    fs::remove_file(w.a.path("art.psd")).unwrap();
    assert!(w.a.sync_ignoring(&ignore).await.is_empty());
    assert!(w.library.join("docs/art.psd").exists());
    assert!(w.library.join("docs/cache/blob.bin").exists());
}

#[tokio::test]
async fn losing_the_whole_folder_redownloads_instead_of_wiping_the_host() {
    let mut w = world().await;
    w.a.write("one.txt", "1");
    w.a.write("sub/two.txt", "2");
    w.a.sync().await;
    fs::remove_dir_all(w.a.dir.path()).unwrap();
    let changes = w.a.sync().await;
    assert!(changes.iter().all(|c| *c == ChangeKind::Downloaded), "{changes:?}");
    assert_eq!(w.a.read("sub/two.txt").as_deref(), Some("2"));
    assert!(w.library.join("docs/one.txt").exists());
}
