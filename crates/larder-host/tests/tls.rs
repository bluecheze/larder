use larder_host::tls::{fingerprint, install_crypto, load_or_create};
use larder_host::{start, Host, HostConfig, Tls};

fn test_host() -> (tempfile::TempDir, Host) {
    let dir = tempfile::tempdir().unwrap();
    let host = Host::open(HostConfig {
        name: "tls test".into(),
        library: dir.path().join("library"),
        data_dir: dir.path().join("data"),
    })
    .unwrap();
    (dir, host)
}

#[tokio::test]
async fn serves_the_pinned_certificate() {
    let (dir, host) = test_host();
    let identity = load_or_create(&dir.path().join("data/tls")).unwrap();
    let expected = identity.fingerprint.clone();
    let server = start(host.router(), "127.0.0.1:0".parse().unwrap(), Tls::SelfSigned(identity)).await.unwrap();

    install_crypto();
    // Accepting any cert is only ok here because we check the fingerprint
    // by hand right after. The real client does that check inside TLS.
    let client = reqwest::Client::builder().danger_accept_invalid_certs(true).tls_info(true).build().unwrap();
    let res = client.get(format!("https://{}/v1/info", server.addr)).send().await.unwrap();
    assert!(res.status().is_success());
    let info = res.extensions().get::<reqwest::tls::TlsInfo>().unwrap();
    assert_eq!(fingerprint(info.peer_certificate().unwrap()), expected);
    server.stop();
}

#[tokio::test]
async fn busy_port_fails_right_away() {
    let (dir, host) = test_host();
    let taken = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let identity = load_or_create(&dir.path().join("data/tls")).unwrap();
    let result = start(host.router(), taken.local_addr().unwrap(), Tls::SelfSigned(identity)).await;
    assert!(result.is_err());
}
