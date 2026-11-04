use std::time::Duration;

use larder_client::{pair, ClientError, HostClient};
use larder_core::{Invite, RelPath};
use larder_host::tls::load_or_create;
use larder_host::{start, Host, HostConfig, RunningServer, Tls};

struct Live {
    _dir: tempfile::TempDir,
    host: Host,
    server: RunningServer,
    fingerprint: String,
}

async fn live_host() -> Live {
    let dir = tempfile::tempdir().unwrap();
    let host = Host::open(HostConfig {
        name: "the host".into(),
        library: dir.path().join("library"),
        data_dir: dir.path().join("data"),
    })
    .unwrap();
    host.create_shelf("docs").unwrap();
    let identity = load_or_create(&dir.path().join("data/tls")).unwrap();
    let fingerprint = identity.fingerprint.clone();
    let server = start(host.router(), "127.0.0.1:0".parse().unwrap(), Tls::SelfSigned(identity)).await.unwrap();
    Live { _dir: dir, host, server, fingerprint }
}

impl Live {
    fn invite(&self) -> Invite {
        Invite {
            address: self.server.addr.to_string(),
            token: self.host.create_invite_token(Duration::from_secs(60)),
            fingerprint: Some(self.fingerprint.clone()),
        }
    }

    async fn client(&self) -> HostClient {
        let connection = pair(&self.invite(), "laptop").await.unwrap();
        assert_eq!(connection.host_name, "the host");
        HostClient::for_connection(&connection).unwrap()
    }
}

fn p(s: &str) -> RelPath {
    RelPath::parse(s).unwrap()
}

#[tokio::test]
async fn pairs_and_moves_files_both_ways() {
    let live = live_host().await;
    let client = live.client().await;
    let shelves = client.shelves().await.unwrap();
    assert_eq!(shelves[0].name, "docs");

    let local = tempfile::tempdir().unwrap();
    let source = local.path().join("hello world #1.txt");
    std::fs::write(&source, "hello from the client").unwrap();
    let path = p("notes/hello world #1.txt");
    let entry = client.upload("docs", &path, &source, None, 1_700_000_000_000).await.unwrap();

    let manifest = client.manifest("docs").await.unwrap();
    assert_eq!(manifest[&path].hash, entry.hash);
    assert_eq!(manifest[&path].modified, 1_700_000_000_000);

    let dest = local.path().join("copy/out.txt");
    client.download("docs", &path, &entry.hash, &dest).await.unwrap();
    assert_eq!(std::fs::read_to_string(&dest).unwrap(), "hello from the client");
    assert_eq!(client.read_range("docs", &path, 15, 6).await.unwrap(), b"client");
}

#[tokio::test]
async fn stale_upload_is_a_conflict() {
    let live = live_host().await;
    let client = live.client().await;
    let local = tempfile::tempdir().unwrap();
    let file = local.path().join("f");
    std::fs::write(&file, "one").unwrap();
    let first = client.upload("docs", &p("f"), &file, None, 0).await.unwrap();
    std::fs::write(&file, "two").unwrap();
    match client.upload("docs", &p("f"), &file, None, 0).await {
        Err(ClientError::Conflict(Some(current))) => assert_eq!(current, first.hash),
        other => panic!("expected a conflict, got {other:?}"),
    }
}

#[tokio::test]
async fn wrong_fingerprint_is_refused() {
    let live = live_host().await;
    let mut invite = live.invite();
    invite.fingerprint = Some("00".repeat(32));
    assert!(matches!(pair(&invite, "laptop").await, Err(ClientError::WrongHost)));
}

#[tokio::test]
async fn invites_only_work_once() {
    let live = live_host().await;
    let invite = live.invite();
    pair(&invite, "first").await.unwrap();
    assert!(matches!(pair(&invite, "second").await, Err(ClientError::BadInvite)));
}

#[tokio::test]
async fn revoked_devices_are_locked_out() {
    let live = live_host().await;
    let connection = pair(&live.invite(), "laptop").await.unwrap();
    let client = HostClient::for_connection(&connection).unwrap();
    assert!(client.shelves().await.is_ok());
    live.host.revoke(&connection.device_id).unwrap();
    assert!(matches!(client.shelves().await, Err(ClientError::Unpaired)));
}

#[tokio::test]
async fn corrupted_download_never_lands() {
    let live = live_host().await;
    let client = live.client().await;
    let local = tempfile::tempdir().unwrap();
    let file = local.path().join("f");
    std::fs::write(&file, "real").unwrap();
    client.upload("docs", &p("f"), &file, None, 0).await.unwrap();
    let dest = local.path().join("out");
    let result = client.download("docs", &p("f"), "not-the-hash", &dest).await;
    assert!(matches!(result, Err(ClientError::Corrupted)));
    assert!(!dest.exists());
    assert_eq!(std::fs::read_dir(local.path()).unwrap().count(), 1);
}
