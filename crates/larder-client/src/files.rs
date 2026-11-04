use std::path::Path;

use futures_util::StreamExt;
use larder_core::protocol::{HEADER_BASE, HEADER_HASH, HEADER_MODIFIED};
use larder_core::scan::{hash_file, PARTIAL_SUFFIX};
use larder_core::{FileEntry, RelPath};
use reqwest::header::RANGE;
use reqwest::Method;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

use crate::api::{check, json, HostClient};
use crate::error::ClientError;

impl HostClient {
    /// Downloads into `<dest>.larder-part` and renames it into place once
    /// the hash checks out. A half-finished download never looks like a
    /// real file to whatever is watching the folder.
    pub async fn download(&self, shelf: &str, path: &RelPath, expected: &str, dest: &Path) -> Result<(), ClientError> {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).await?;
        }
        let part = dest.with_file_name(format!("{}{PARTIAL_SUFFIX}", path.file_name()));
        let res = check(self.request(Method::GET, self.file_url(shelf, path)).send().await?).await?;
        let result = write_stream(res, &part, expected).await;
        match result {
            Ok(()) => Ok(fs::rename(&part, dest).await?),
            Err(err) => {
                let _ = fs::remove_file(&part).await;
                Err(err)
            }
        }
    }

    /// One slice of a file, for the drive. Videos and big files get read
    /// like this instead of being downloaded whole.
    pub async fn read_range(&self, shelf: &str, path: &RelPath, start: u64, len: u64) -> Result<Vec<u8>, ClientError> {
        if len == 0 {
            return Ok(Vec::new());
        }
        let range = format!("bytes={start}-{}", start + len - 1);
        let req = self.request(Method::GET, self.file_url(shelf, path)).header(RANGE, range);
        Ok(check(req.send().await?).await?.bytes().await?.to_vec())
    }

    /// `base` is the hash we think the host has. None means "this is new".
    /// The host answers with a Conflict if it's wrong.
    pub async fn upload(
        &self,
        shelf: &str,
        path: &RelPath,
        source: &Path,
        base: Option<&str>,
        modified: i64,
    ) -> Result<FileEntry, ClientError> {
        let source_owned = source.to_path_buf();
        let hash = tokio::task::spawn_blocking(move || hash_file(&source_owned))
            .await
            .map_err(std::io::Error::other)??;
        let file = File::open(source).await?;
        let req = self
            .request(Method::PUT, self.file_url(shelf, path))
            .header(HEADER_HASH, hash)
            .header(HEADER_BASE, base.unwrap_or(""))
            .header(HEADER_MODIFIED, modified.to_string())
            .body(reqwest::Body::wrap_stream(ReaderStream::new(file)));
        json(req.send().await?).await
    }

    pub async fn delete(&self, shelf: &str, path: &RelPath, base: &str) -> Result<(), ClientError> {
        let req = self.request(Method::DELETE, self.file_url(shelf, path)).header(HEADER_BASE, base);
        check(req.send().await?).await?;
        Ok(())
    }
}

async fn write_stream(res: reqwest::Response, part: &Path, expected: &str) -> Result<(), ClientError> {
    let mut file = File::create(part).await?;
    let mut hasher = blake3::Hasher::new();
    let mut stream = res.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        hasher.update(&chunk);
        file.write_all(&chunk).await?;
    }
    file.sync_all().await?;
    if hasher.finalize().to_hex().as_str() != expected {
        return Err(ClientError::Corrupted);
    }
    Ok(())
}
