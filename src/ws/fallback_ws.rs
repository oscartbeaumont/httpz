use crate::{ws::Websocket, Error, HttpResponse, Request};
use http::Response;
use std::{future::Future, marker::PhantomData};

/// TODO
pub enum WebsocketUpgrade {}

impl WebsocketUpgrade {
    /// TODO
    pub fn from_req<THandler, TFut>(
        _req: Request,
        _handler: THandler,
    ) -> WebSocketUpgradeResponse<THandler, TFut>
    where
        THandler:
            for<'a> FnOnce(Request, Box<dyn Websocket + Send>) -> TFut + Send + Sync + 'static,
        TFut: Future<Output = ()> + Send + 'static,
    {
        WebSocketUpgradeResponse(PhantomData)
    }

    #[cfg(feature = "cookies")]
    pub fn from_req_with_cookies<THandler, TFut>(
        mut req: Request,
        cookies: cookie::CookieJar,
        handler: THandler,
    ) -> WebSocketUpgradeResponse<THandler, TFut>
    where
        THandler:
            for<'a> FnOnce(Request, Box<dyn Websocket + Send>) -> TFut + Send + Sync + 'static,
        TFut: Future<Output = ()> + Send + 'static,
    {
        WebSocketUpgradeResponse(PhantomData)
    }
}

/// TODO
pub struct WebSocketUpgradeResponse<THandler, TFut>(PhantomData<(THandler, TFut)>)
where
    THandler: for<'a> FnOnce(Request, Box<dyn Websocket + Send>) -> TFut + Send + Sync + 'static,
    TFut: Future<Output = ()> + Send + 'static;

// By only spawning the tokio task here, we ensure we aren't spawning tasks if the user forgets to return the websocket upgrade response from the handler.
impl<THandler, TFut> HttpResponse for WebSocketUpgradeResponse<THandler, TFut>
where
    THandler: for<'a> FnOnce(Request, Box<dyn Websocket + Send>) -> TFut + Send + Sync + 'static,
    TFut: Future<Output = ()> + Send + 'static,
{
    fn into_response(self) -> Result<Response<Vec<u8>>, Error> {
        println!("[Error] Websocket upgrade not supported by current server.");
        Ok(Response::builder().status(500).body(Vec::new())?)
    }
}
