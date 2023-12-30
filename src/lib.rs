//! Code once, support every Rust webserver!
#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::)]

mod endpoint;
mod error;
mod generic_endpoint;
mod request;
mod response;
mod server;

/// is the module containing code related to handling incoming websockets.
#[cfg(feature = "ws")]
pub mod ws;

pub use endpoint::*;
pub use error::*;
pub use generic_endpoint::*;
pub use request::*;
pub use response::*;
pub use server::*;

#[cfg(feature = "cookies")]
pub use cookie;
pub use form_urlencoded;
pub use http;
