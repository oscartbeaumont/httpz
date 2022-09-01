//! Code once, support every Rust webserver!
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod endpoint;
mod error;
mod generic_endpoint;
mod servers;

/// is the module containing code related to handling incoming websockets.
#[cfg(feature = "ws")]
pub mod ws;

pub use endpoint::*;
pub use error::*;
pub use generic_endpoint::*;
pub use servers::*;

pub use cookie;
pub use http;

/// is a trait which allows you to get a pass query parameter from a [http::Uri].
pub trait QueryParms {
    /// query_pairs returns an iterator of the query parameters.
    fn query_pairs(&self) -> Option<form_urlencoded::Parse<'_>>;
}

impl QueryParms for http::Uri {
    fn query_pairs(&self) -> Option<form_urlencoded::Parse<'_>> {
        self.query()
            .map(|query| form_urlencoded::parse(query.as_bytes()))
    }
}
