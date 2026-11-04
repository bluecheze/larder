use std::path::PathBuf;
use std::sync::Arc;

use futures_util::StreamExt;
use rustls_acme::caches::DirCache;
use rustls_acme::axum::AxumAcceptor;
use rustls_acme::AcmeConfig;

pub struct AcmeSettings {
    pub domain: String,
    pub email: String,
    pub cache_dir: PathBuf,
    /// Let's Encrypt's staging server hands out untrusted certs but has
    /// much looser rate limits. Handy while you're still poking at DNS.
    pub staging: bool,
}

/// Uses the TLS-ALPN-01 challenge, so everything happens on the one port
/// Larder already listens on. Your router has to send outside port 443 to
/// it, Let's Encrypt only ever knocks on 443.
pub fn acceptor(settings: AcmeSettings) -> AxumAcceptor {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut state = AcmeConfig::new_with_provider([settings.domain], provider)
        .contact_push(format!("mailto:{}", settings.email))
        .cache(DirCache::new(settings.cache_dir))
        .directory_lets_encrypt(!settings.staging)
        .state();
    let acceptor = state.axum_acceptor(state.default_rustls_config());
    tokio::spawn(async move {
        while let Some(event) = state.next().await {
            match event {
                Ok(ok) => tracing::info!("certificate: {ok:?}"),
                Err(err) => tracing::warn!("couldn't get a certificate yet: {err}"),
            }
        }
    });
    acceptor
}
