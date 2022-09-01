use std::fmt::{Display, Formatter};

/// a generic error type to represent all possible errors from httpz
pub enum Error {
    /// an error that occurred in the HTTP library
    HTTPError(http::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Error::HTTPError(err) => write!(f, "HTTP error: {}", err),
        }
    }
}

impl From<http::Error> for Error {
    fn from(err: http::Error) -> Error {
        Error::HTTPError(err)
    }
}
