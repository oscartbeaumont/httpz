use std::fmt::{Display, Formatter};

pub enum Error {
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
