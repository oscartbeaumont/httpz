use std::{
    pin::Pin,
    task::{Context, Poll},
};

use async_tungstenite::{self, tokio::TokioAdapter, tungstenite::protocol, WebSocketStream};
use base64::{engine::general_purpose::STANDARD, Engine};
use futures::{Future, Sink, Stream};
use http::{
    header::{self, HeaderName, SET_COOKIE},
    HeaderValue, Method, Response, StatusCode,
};
use hyper::upgrade::{OnUpgrade, Upgraded};
use sha1::{Digest, Sha1};

#[allow(clippy::declare_interior_mutable_const)] // TODO: Fix
const UPGRADE: HeaderValue = HeaderValue::from_static("upgrade");
#[allow(clippy::declare_interior_mutable_const)] // TODO: Fix
const WEBSOCKET: HeaderValue = HeaderValue::from_static("websocket");

use crate::{Error, HttpResponse, Request};

use super::{Message, Websocket};

/// TODO
pub enum WebsocketUpgrade {}

impl WebsocketUpgrade {
    /// TODO
    pub fn from_req<THandler, TFut>(
        req: Request,
        handler: THandler,
    ) -> WebSocketUpgradeResponse<THandler, TFut>
    where
        THandler: FnOnce(Request, Box<dyn Websocket + Send>) -> TFut + Send + Sync + 'static,
        TFut: Future<Output = ()> + Send + 'static,
    {
        WebSocketUpgradeResponse {
            req,
            handler,
            #[cfg(feature = "cookies")]
            cookies: None,
        }
    }

    /// TODO
    #[cfg(feature = "cookies")]
    pub fn from_req_with_cookies<THandler, TFut>(
        req: Request,
        cookies: cookie::CookieJar,
        handler: THandler,
    ) -> WebSocketUpgradeResponse<THandler, TFut>
    where
        THandler: FnOnce(Request, Box<dyn Websocket + Send>) -> TFut + Send + Sync + 'static,
        TFut: Future<Output = ()> + Send + 'static,
    {
        WebSocketUpgradeResponse {
            req,
            handler,
            cookies: Some(cookies),
        }
    }
}

/// TODO
pub struct WebSocketUpgradeResponse<THandler, TFut>
where
    THandler: FnOnce(Request, Box<dyn Websocket + Send>) -> TFut + Send + Sync + 'static,
    TFut: Future<Output = ()> + Send + 'static,
{
    req: Request,
    handler: THandler,
    #[cfg(feature = "cookies")]
    cookies: Option<cookie::CookieJar>,
}

// By only spawning the tokio task here, we ensure we aren't spawning tasks if the user forgets to return the websocket upgrade response from the handler.
impl<THandler, TFut> HttpResponse for WebSocketUpgradeResponse<THandler, TFut>
where
    THandler: FnOnce(Request, Box<dyn Websocket + Send>) -> TFut + Send + Sync + 'static,
    TFut: Future<Output = ()> + Send + 'static,
{
    fn into_response(mut self) -> Result<Response<Vec<u8>>, Error> {
        let mut resp = Response::builder();

        #[cfg(feature = "cookies")]
        if let Some(jar) = self.cookies {
            if let Some(headers) = resp.headers_mut() {
                for cookie in jar.delta() {
                    if let Ok(header_value) = cookie.encoded().to_string().parse() {
                        headers.append(SET_COOKIE, header_value);
                    }
                }
            }
        }

        if self.req.method() != Method::GET {
            return Ok(resp.status(StatusCode::METHOD_NOT_ALLOWED).body(vec![])?);
        }

        if !header_contains(&self.req, header::CONNECTION, "upgrade") {
            return Ok(resp.status(StatusCode::BAD_REQUEST).body(vec![])?);
        }

        if !header_eq(&self.req, header::UPGRADE, "websocket") {
            return Ok(resp.status(StatusCode::BAD_REQUEST).body(vec![])?);
        }

        if !header_eq(&self.req, header::SEC_WEBSOCKET_VERSION, "13") {
            return Ok(resp.status(StatusCode::BAD_REQUEST).body(vec![])?);
        }

        let sec_websocket_key = match self.req.headers_mut().remove(header::SEC_WEBSOCKET_KEY) {
            Some(sec_websocket_key) => sec_websocket_key,
            None => return Ok(resp.status(StatusCode::BAD_REQUEST).body(vec![])?),
        };

        // TODO: This is an Axum thing. Support for other services will be needed.
        let on_upgrade = match self.req.extensions_mut().remove::<OnUpgrade>() {
            Some(on_upgrade) => on_upgrade,
            None => return Ok(resp.status(StatusCode::BAD_REQUEST).body(vec![])?),
        };

        // let sec_websocket_protocol = self.req.headers().get(header::SEC_WEBSOCKET_PROTOCOL).cloned();

        tokio::spawn(async move {
            let upgraded = on_upgrade.await.expect("connection upgrade failed");

            let upgraded = async_tungstenite::tokio::TokioAdapter::new(upgraded);

            let socket = WebSocketStream::from_raw_socket(upgraded, protocol::Role::Server, None) // TODO: Specify context: Some(config)
                .await;

            (self.handler)(self.req, Box::new(TokioSocket(socket))).await;
        });

        let builder = resp
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

fn header_eq(req: &Request, key: HeaderName, value: &'static str) -> bool {
    if let Some(header) = req.headers().get(&key) {
        header.as_bytes().eq_ignore_ascii_case(value.as_bytes())
    } else {
        false
    }
}

fn header_contains(req: &Request, key: HeaderName, value: &'static str) -> bool {
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
    HeaderValue::from_maybe_shared(STANDARD.encode(sha1.finalize()))
        .expect("base64 is a valid value")
}

pub(crate) struct TokioSocket(WebSocketStream<TokioAdapter<Upgraded>>);

impl Sink<Message> for TokioSocket {
    type Error = crate::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        <WebSocketStream<TokioAdapter<Upgraded>> as Sink<async_tungstenite::tungstenite::Message>>::poll_ready(
            Pin::new(&mut self.0),
            cx,
        ).map_err(|e| e.into())
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        <WebSocketStream<TokioAdapter<Upgraded>> as Sink<async_tungstenite::tungstenite::Message>>::start_send(
            Pin::new(&mut self.0),
            item.into(),
        ).map_err(|e| e.into())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        <WebSocketStream<TokioAdapter<Upgraded>> as Sink<async_tungstenite::tungstenite::Message>>::poll_flush(
            Pin::new(&mut self.0),
            cx,
        ).map_err(|e| e.into())
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        <WebSocketStream<TokioAdapter<Upgraded>> as Sink<async_tungstenite::tungstenite::Message>>::poll_close(
            Pin::new(&mut self.0),
            cx,
        ).map_err(|e| e.into())
    }
}

impl Stream for TokioSocket {
    type Item = Result<Message, crate::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match <WebSocketStream<TokioAdapter<Upgraded>> as Stream>::poll_next(
            Pin::new(&mut self.0),
            cx,
        ) {
            Poll::Ready(msg) => Poll::Ready(msg.map(|v| v.map(Into::into).map_err(Into::into))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
