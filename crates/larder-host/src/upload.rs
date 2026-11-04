use std::io;
use std::path::{Path, PathBuf};

use axum::body::Body;
use futures_util::StreamExt;
use larder_core::scan::PARTIAL_SUFFIX;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

use crate::secret::new_secret;

/// An upload that has fully arrived but isn't in the shelf yet.
pub struct Staged {
    pub path: PathBuf,
    pub hash: String,
}

/// Streams the body to a temp file and hashes it on the way, so a 20 GB
/// video never has to fit in memory. The temp file is removed if anything
/// goes wrong halfway.
pub async fn stage(body: Body, tmp_dir: &Path) -> io::Result<Staged> {
    fs::create_dir_all(tmp_dir).await?;
    let path = tmp_dir.join(format!("{}{PARTIAL_SUFFIX}", new_secret(8)));
    match write_body(body, &path).await {
        Ok(hash) => Ok(Staged { path, hash }),
        Err(err) => {
            let _ = fs::remove_file(&path).await;
            Err(err)
        }
    }
}

async fn write_body(body: Body, path: &Path) -> io::Result<String> {
    let mut file = File::create(path).await?;
    let mut hasher = blake3::Hasher::new();
    let mut stream = body.into_data_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(io::Error::other)?;
        hasher.update(&chunk);
        file.write_all(&chunk).await?;
    }
    file.sync_all().await?;
    Ok(hasher.finalize().to_hex().to_string())
}

impl Staged {
    pub async fn discard(self) {
        let _ = fs::remove_file(&self.path).await;
    }
}
