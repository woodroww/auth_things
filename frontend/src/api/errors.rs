
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Expired or missing auth token")]
    NotAuthenticated,
    #[error("Unknown network error")]
    Unknown,
    #[error("channel error")]
    ChannelError,
    #[error("http client error")]
    ClientError,
}
