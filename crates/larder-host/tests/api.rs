use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use larder_core::protocol::{PairResponse, HEADER_BASE, HEADER_HASH};
use larder_core::Manifest;
use larder_host::{Host, HostConfig};
use tower::ServiceExt;

struct TestHost {
    _library: tempfile::TempDir,
    _data: tempfile::TempDir,
    library_path: std::path::PathBuf,
    router: Router,
    token: String,
}

async fn setup() -> TestHost {
    let library = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    let host = Host::open(HostConfig {
        name: "test host".into(),
        library: library.path().to_path_buf(),
        data_dir: data.path().to_path_buf(),
    })
    .unwrap();
    host.create_shelf("docs").unwrap();
    let invite = host.create_invite_token(Duration::from_secs(60));
    let router = host.router();
    let body = format!(r#"{{"token":"{invite}","device_name":"laptop"}}"#);
    let (status, bytes) = send(&router, request("POST", "/v1/pair", None).header("content-type", "application/json"), body).await;
    assert_eq!(status, StatusCode::OK);
    let paired: PairResponse = serde_json::from_slice(&bytes).unwrap();
    TestHost {
        library_path: library.path().to_path_buf(),
        _library: library,
        _data: data,
        router,
        token: paired.device_token,
    }
}

fn request(method: &str, uri: &str, token: Option<&str>) -> axum::http::request::Builder {
    let builder = Request::builder().method(method).uri(uri);
    match token {
        Some(token) => builder.header("authorization", format!("Bearer {token}")),
        None => builder,
    }
}

async fn send(router: &Router, builder: axum::http::request::Builder, body: impl Into<Body>) -> (StatusCode, Vec<u8>) {
    let res = router.clone().oneshot(builder.body(body.into()).unwrap()).await.unwrap();
    let status = res.status();
    (status, res.into_body().collect().await.unwrap().to_bytes().to_vec())
}

fn hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

impl TestHost {
    async fn put(&self, path: &str, data: &'static [u8], base: Option<&str>) -> StatusCode {
        let uri = format!("/v1/shelves/docs/files/{path}");
        let builder = request("PUT", &uri, Some(&self.token))
            .header(HEADER_HASH, hash(data))
            .header(HEADER_BASE, base.unwrap_or(""));
        send(&self.router, builder, data).await.0
    }

    async fn manifest(&self) -> Manifest {
        let (status, bytes) = send(&self.router, request("GET", "/v1/shelves/docs/manifest", Some(&self.token)), Body::empty()).await;
        assert_eq!(status, StatusCode::OK);
        serde_json::from_slice(&bytes).unwrap()
    }
}

#[tokio::test]
async fn info_is_public_but_shelves_need_pairing() {
    let host = setup().await;
    let (status, _) = send(&host.router, request("GET", "/v1/info", None), Body::empty()).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send(&host.router, request("GET", "/v1/shelves", None), Body::empty()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = send(&host.router, request("GET", "/v1/shelves", Some("made-up")), Body::empty()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn upload_list_and_download() {
    let host = setup().await;
    assert_eq!(host.put("notes/today.txt", b"hello larder", None).await, StatusCode::OK);
    let manifest = host.manifest().await;
    let entry = manifest.values().next().unwrap();
    assert_eq!(entry.hash, hash(b"hello larder"));

    let uri = "/v1/shelves/docs/files/notes/today.txt";
    let (status, body) = send(&host.router, request("GET", uri, Some(&host.token)), Body::empty()).await;
    assert_eq!((status, body.as_slice()), (StatusCode::OK, b"hello larder".as_slice()));

    let ranged = request("GET", uri, Some(&host.token)).header("range", "bytes=6-");
    let (status, body) = send(&host.router, ranged, Body::empty()).await;
    assert_eq!((status, body.as_slice()), (StatusCode::PARTIAL_CONTENT, b"larder".as_slice()));
}

#[tokio::test]
async fn stale_uploads_are_refused() {
    let host = setup().await;
    assert_eq!(host.put("a.txt", b"one", None).await, StatusCode::OK);
    assert_eq!(host.put("a.txt", b"two", None).await, StatusCode::CONFLICT);
    assert_eq!(host.put("a.txt", b"two", Some("not-the-hash")).await, StatusCode::CONFLICT);
    assert_eq!(host.put("a.txt", b"two", Some(&hash(b"one"))).await, StatusCode::OK);
}

#[tokio::test]
async fn corrupted_uploads_leave_nothing_behind() {
    let host = setup().await;
    let builder = request("PUT", "/v1/shelves/docs/files/x.bin", Some(&host.token)).header(HEADER_HASH, "wrong");
    let (status, _) = send(&host.router, builder, "data").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(host.manifest().await.is_empty());
    let tmp = host.library_path.join(".larder/tmp");
    assert_eq!(std::fs::read_dir(tmp).unwrap().count(), 0);
}

#[tokio::test]
async fn delete_checks_base_and_prunes_folders() {
    let host = setup().await;
    host.put("deep/folder/f.txt", b"bye", None).await;
    let uri = "/v1/shelves/docs/files/deep/folder/f.txt";
    let wrong = request("DELETE", uri, Some(&host.token)).header(HEADER_BASE, "nope");
    assert_eq!(send(&host.router, wrong, Body::empty()).await.0, StatusCode::CONFLICT);
    let right = request("DELETE", uri, Some(&host.token)).header(HEADER_BASE, hash(b"bye"));
    assert_eq!(send(&host.router, right, Body::empty()).await.0, StatusCode::NO_CONTENT);
    assert!(host.manifest().await.is_empty());
    assert!(!host.library_path.join("docs/deep").exists());
}

#[tokio::test]
async fn move_renames_but_never_overwrites() {
    let host = setup().await;
    host.put("old.txt", b"x", None).await;
    host.put("taken.txt", b"y", None).await;
    let mv = |to: &str| {
        let body = format!(r#"{{"from":"old.txt","to":"{to}"}}"#);
        let builder = request("POST", "/v1/shelves/docs/move", Some(&host.token)).header("content-type", "application/json");
        send(&host.router, builder, body)
    };
    assert_eq!(mv("taken.txt").await.0, StatusCode::CONFLICT);
    assert_eq!(mv("sub/new.txt").await.0, StatusCode::OK);
    let names: Vec<_> = host.manifest().await.keys().map(|k| k.to_string()).collect();
    assert_eq!(names, ["sub/new.txt", "taken.txt"]);
}

#[tokio::test]
async fn paths_cant_escape_the_shelf() {
    let host = setup().await;
    std::fs::write(host.library_path.join("secret.txt"), "private").unwrap();
    for uri in ["/v1/shelves/docs/files/..%2Fsecret.txt", "/v1/shelves/..%2Fdocs/manifest"] {
        let (status, _) = send(&host.router, request("GET", uri, Some(&host.token)), Body::empty()).await;
        assert!(status == StatusCode::BAD_REQUEST || status == StatusCode::NOT_FOUND, "{uri} gave {status}");
    }
}
