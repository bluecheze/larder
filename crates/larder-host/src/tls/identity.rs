use std::fs;
use std::io;
use std::path::Path;

use sha2::{Digest, Sha256};

/// The host's self-signed certificate. Made once on first start and kept,
/// because changing it would break every client that pinned the old one.
pub struct Identity {
    pub cert_der: Vec<u8>,
    pub key_der: Vec<u8>,
    pub fingerprint: String,
}

pub fn load_or_create(dir: &Path) -> io::Result<Identity> {
    let cert_path = dir.join("cert.der");
    let key_path = dir.join("key.der");
    let (cert_der, key_der) = if cert_path.exists() && key_path.exists() {
        (fs::read(&cert_path)?, fs::read(&key_path)?)
    } else {
        let made = create()?;
        fs::create_dir_all(dir)?;
        fs::write(&cert_path, &made.0)?;
        write_private(&key_path, &made.1)?;
        made
    };
    let fingerprint = fingerprint(&cert_der);
    Ok(Identity { cert_der, key_der, fingerprint })
}

pub fn fingerprint(cert_der: &[u8]) -> String {
    hex::encode(Sha256::digest(cert_der))
}

fn create() -> io::Result<(Vec<u8>, Vec<u8>)> {
    // Clients check the fingerprint, not the hostname, so these names are
    // only there to keep picky TLS libraries happy.
    let names = vec!["larder".to_string(), "localhost".to_string()];
    let made = rcgen::generate_simple_self_signed(names).map_err(io::Error::other)?;
    Ok((made.cert.der().to_vec(), made.signing_key.serialize_der()))
}

#[cfg(unix)]
fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(path)?;
    file.write_all(bytes)
}

// TODO: lock the key file down to the current user on Windows too. The
// default ACLs on the app data folder already keep other users out, so
// this is belt and braces rather than a hole.
#[cfg(not(unix))]
fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    fs::write(path, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_identity_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        let first = load_or_create(dir.path()).unwrap();
        let second = load_or_create(dir.path()).unwrap();
        assert_eq!(first.fingerprint, second.fingerprint);
        assert_eq!(first.fingerprint.len(), 64);
    }

    #[cfg(unix)]
    #[test]
    fn key_is_private() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        load_or_create(dir.path()).unwrap();
        let mode = fs::metadata(dir.path().join("key.der")).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
