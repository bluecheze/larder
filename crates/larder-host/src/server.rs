use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use axum_server::Handle;

use crate::tls::{acme_acceptor, install_crypto, AcmeSettings, Identity};

pub enum Tls {
    SelfSigned(Identity),
    Acme(AcmeSettings),
}

/// A host server that's up and accepting connections.
pub struct RunningServer {
    pub addr: SocketAddr,
    handle: Handle<SocketAddr>,
}

impl RunningServer {
    /// Lets in-flight transfers finish for a couple of seconds, then cuts
    /// whatever's left.
    pub fn stop(&self) {
        self.handle.graceful_shutdown(Some(Duration::from_secs(2)));
    }
}

/// Starts serving and only returns once the port is actually bound, so a
/// "port already in use" shows up here instead of failing quietly later.
pub async fn start(router: Router, addr: SocketAddr, tls: Tls) -> io::Result<RunningServer> {
    install_crypto();
    let handle = Handle::new();
    let app = router.into_make_service();
    let task = match tls {
        Tls::SelfSigned(identity) => {
            let config = RustlsConfig::from_der(vec![identity.cert_der], identity.key_der).await?;
            tokio::spawn(axum_server::bind_rustls(addr, config).handle(handle.clone()).serve(app))
        }
        Tls::Acme(settings) => {
            let acceptor = acme_acceptor(settings);
            tokio::spawn(axum_server::bind(addr).acceptor(acceptor).handle(handle.clone()).serve(app))
        }
    };
    match handle.listening().await {
        Some(addr) => Ok(RunningServer { addr, handle }),
        None => Err(match task.await {
            Ok(Err(err)) => err,
            Ok(Ok(())) => io::Error::other("the server stopped before it started"),
            Err(join) => io::Error::other(join),
        }),
    }
}
