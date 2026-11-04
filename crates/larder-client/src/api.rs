use std::time::Duration;

use larder_core::protocol::{ApiError, HostInfo, MoveRequest, NewShelf, PairRequest, PairResponse, Shelf, API_VERSION};
use larder_core::{FileEntry, Manifest, RelPath};
use reqwest::{Method, RequestBuilder, Response, StatusCode, Url};

use crate::error::ClientError;
use crate::tls::client_config;

/// Talks to one host. Cheap to clone, the connection pool is shared.
#[derive(Clone)]
pub struct HostClient {
    pub(crate) http: reqwest::Client,
    base: Url,
    token: Option<String>,
}

impl HostClient {
    pub fn new(address: &str, fingerprint: Option<&str>, token: Option<String>) -> Result<Self, ClientError> {
        let base = Url::parse(&format!("https://{address}/"))
            .map_err(|_| ClientError::Unreachable(format!("{address:?} isn't a usable address")))?;
        let http = reqwest::Client::builder()
            .use_preconfigured_tls(client_config(fingerprint))
            .connect_timeout(Duration::from_secs(8))
            .build()?;
        Ok(HostClient { http, base, token })
    }

    /// Builds `base/v1/<parts...>` with every part percent-encoded, so a
    /// file called "notes #2.txt" doesn't turn into a URL fragment.
    pub(crate) fn url(&self, parts: &[&str]) -> Url {
        let mut url = self.base.clone();
        url.path_segments_mut().expect("https urls have paths").pop_if_empty().push("v1").extend(parts);
        url
    }

    pub(crate) fn file_url(&self, shelf: &str, path: &RelPath) -> Url {
        let mut parts = vec!["shelves", shelf, "files"];
        parts.extend(path.as_str().split('/'));
        self.url(&parts)
    }

    pub(crate) fn request(&self, method: Method, url: Url) -> RequestBuilder {
        let req = self.http.request(method, url);
        match &self.token {
            Some(token) => req.bearer_auth(token),
            None => req,
        }
    }

    pub async fn info(&self) -> Result<HostInfo, ClientError> {
        let info: HostInfo = json(self.request(Method::GET, self.url(&["info"])).send().await?).await?;
        if info.api_version > API_VERSION {
            return Err(ClientError::TooOld);
        }
        Ok(info)
    }

    pub async fn pair(&self, token: &str, device_name: &str) -> Result<PairResponse, ClientError> {
        let body = PairRequest { token: token.into(), device_name: device_name.into() };
        json(self.request(Method::POST, self.url(&["pair"])).json(&body).send().await?).await
    }

    pub async fn shelves(&self) -> Result<Vec<Shelf>, ClientError> {
        json(self.request(Method::GET, self.url(&["shelves"])).send().await?).await
    }

    pub async fn create_shelf(&self, name: &str) -> Result<Shelf, ClientError> {
        let body = NewShelf { name: name.into() };
        json(self.request(Method::POST, self.url(&["shelves"])).json(&body).send().await?).await
    }

    pub async fn manifest(&self, shelf: &str) -> Result<Manifest, ClientError> {
        json(self.request(Method::GET, self.url(&["shelves", shelf, "manifest"])).send().await?).await
    }

    pub async fn move_file(&self, shelf: &str, from: &RelPath, to: &RelPath) -> Result<FileEntry, ClientError> {
        let body = MoveRequest { from: from.to_string(), to: to.to_string() };
        json(self.request(Method::POST, self.url(&["shelves", shelf, "move"])).json(&body).send().await?).await
    }
}

pub(crate) async fn json<T: serde::de::DeserializeOwned>(res: Response) -> Result<T, ClientError> {
    Ok(check(res).await?.json().await?)
}

/// Turns the host's error responses into something the app can act on.
pub(crate) async fn check(res: Response) -> Result<Response, ClientError> {
    let status = res.status();
    if status.is_success() {
        return Ok(res);
    }
    let body: Option<ApiError> = res.json().await.ok();
    Err(match status {
        StatusCode::UNAUTHORIZED => ClientError::Unpaired,
        StatusCode::NOT_FOUND => ClientError::NotFound,
        StatusCode::CONFLICT => ClientError::Conflict(body.and_then(|b| b.current)),
        _ => ClientError::Refused(body.map_or_else(|| status.to_string(), |b| b.error)),
    })
}
