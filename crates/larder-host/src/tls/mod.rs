mod acme;
mod identity;

pub use acme::AcmeSettings;
pub use identity::{fingerprint, load_or_create, Identity};

pub(crate) use acme::acceptor as acme_acceptor;

/// Picks ring as the crypto backend for the whole process. aws-lc is the
/// rustls default but needs cmake and nasm to build on Windows, which is a
/// lot to ask of someone just trying to compile a file sync app.
pub fn install_crypto() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}
