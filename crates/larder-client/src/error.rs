#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("couldn't reach the host: {0}")]
    Unreachable(String),
    #[error("this isn't your host, its certificate doesn't match the invite")]
    WrongHost,
    #[error("the host says this device isn't paired anymore")]
    Unpaired,
    #[error("that invite was already used or has expired, make a new one on the host")]
    BadInvite,
    /// The file changed on the host since we last looked. Carries what the
    /// host has now, None if the file is gone.
    #[error("the file changed on the host")]
    Conflict(Option<String>),
    #[error("the host doesn't have that")]
    NotFound,
    #[error("the host said no: {0}")]
    Refused(String),
    #[error("the download got corrupted on the way")]
    Corrupted,
    #[error("the host runs a newer version of Larder, update this one")]
    TooOld,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl From<reqwest::Error> for ClientError {
    fn from(err: reqwest::Error) -> Self {
        // rustls reports a pin mismatch as a generic TLS failure, so we
        // dig for our own marker in the error chain.
        let mut source: Option<&dyn std::error::Error> = Some(&err);
        while let Some(current) = source {
            if current.to_string().contains(crate::tls::MISMATCH) {
                return ClientError::WrongHost;
            }
            source = current.source();
        }
        ClientError::Unreachable(err.to_string())
    }
}
