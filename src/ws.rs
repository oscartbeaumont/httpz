use async_tungstenite::{
    self,
    tungstenite::{self, protocol},
    WebSocketStream,
};
use futures::{Future, Sink, Stream};
use http::{
    header::{self, HeaderName},
    HeaderValue, Method, Response, StatusCode,
};
use hyper::upgrade::OnUpgrade;
use sha1::{Digest, Sha1};

use crate::ConcreteRequest;

const UPGRADE: HeaderValue = HeaderValue::from_static("upgrade");
const WEBSOCKET: HeaderValue = HeaderValue::from_static("websocket");

pub use async_tungstenite::tungstenite::Message;

/// TODO
pub trait Websocket:
    Sink<Message, Error = tungstenite::Error>
    + Stream<Item = Result<async_tungstenite::tungstenite::Message, tungstenite::Error>>
    + Send
    + Unpin
{
}

impl<T> Websocket for T where
    T: Sink<Message, Error = tungstenite::Error>
        + Stream<Item = Result<async_tungstenite::tungstenite::Message, tungstenite::Error>>
        + Send
        + Unpin
{
}

/// TODO
pub struct WebsocketUpgrade {}

impl WebsocketUpgrade {
    /// TODO: Error handling + unit testing this code
    pub fn from_req<TFut>(
        mut req: ConcreteRequest,
        mut handler: impl for<'a> FnOnce(Box<dyn Websocket + Send>) -> TFut + Send + Sync + 'static,
    ) -> Result<Response<Vec<u8>>, crate::Error>
    where
        TFut: Future<Output = ()> + Send + 'static,
    {
        if req.method() != Method::GET {
            return Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(vec![])?);
        }

        if !header_contains(&req, header::CONNECTION, "upgrade") {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(vec![])?);
        }

        if !header_eq(&req, header::UPGRADE, "websocket") {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(vec![])?);
        }

        if !header_eq(&req, header::SEC_WEBSOCKET_VERSION, "13") {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(vec![])?);
        }

        let sec_websocket_key = match req.headers_mut().remove(header::SEC_WEBSOCKET_KEY) {
            Some(sec_websocket_key) => sec_websocket_key,
            None => {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(vec![])?)
            }
        };

        let on_upgrade = match req.extensions_mut().remove::<OnUpgrade>() {
            Some(on_upgrade) => on_upgrade,
            None => {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(vec![])?)
            }
        };

        // let sec_websocket_protocol = req.headers().get(header::SEC_WEBSOCKET_PROTOCOL).cloned();

        tokio::spawn(async move {
            let upgraded = on_upgrade.await.expect("connection upgrade failed");

            let upgraded = async_tungstenite::tokio::TokioAdapter::new(upgraded);

            let socket = WebSocketStream::from_raw_socket(upgraded, protocol::Role::Server, None) // TODO: Specify context: Some(config)
                .await;

            handler(Box::new(socket)).await;
        });

        let builder = Response::builder()
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .header(header::CONNECTION, UPGRADE)
            .header(header::UPGRADE, WEBSOCKET)
            .header(
                header::SEC_WEBSOCKET_ACCEPT,
                sign(sec_websocket_key.as_bytes()),
            );

        // if let Some(protocol) = protocol {
        //     builder = builder.header(header::SEC_WEBSOCKET_PROTOCOL, protocol);
        // }

        Ok(builder.body([].to_vec())?)
    }
}

fn header_eq(req: &ConcreteRequest, key: HeaderName, value: &'static str) -> bool {
    if let Some(header) = req.headers().get(&key) {
        header.as_bytes().eq_ignore_ascii_case(value.as_bytes())
    } else {
        false
    }
}

fn header_contains(req: &ConcreteRequest, key: HeaderName, value: &'static str) -> bool {
    let header = if let Some(header) = req.headers().get(&key) {
        header
    } else {
        return false;
    };

    if let Ok(header) = std::str::from_utf8(header.as_bytes()) {
        header.to_ascii_lowercase().contains(value)
    } else {
        false
    }
}

fn sign(key: &[u8]) -> HeaderValue {
    let mut sha1 = Sha1::default();
    sha1.update(key);
    sha1.update(&b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"[..]);
    HeaderValue::from_maybe_shared(base64::encode(&sha1.finalize()))
        .expect("base64 is a valid value")
}
