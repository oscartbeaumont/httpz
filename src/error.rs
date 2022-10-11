use thiserror::Error;

/// a generic error type to represent all possible errors from httpz
#[derive(Error, Debug)]
pub enum Error {
    /// an error that occurred in the HTTP library
    #[error("http error: {0}")]
    HTTPError(#[from] http::Error),
    /// UTF coding error.
    #[error("UTF-8 encoding error")]
    Utf8,
    /// TODO
    #[error("UTF-8 encoding error")]
    #[cfg(feature = "tokio-ws")]
    TungsteniteError(#[from] async_tungstenite::tungstenite::Error),
}
