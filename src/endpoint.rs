use cookie::CookieJar;
use http::{Method, Request, Response};
use std::future::Future;

use crate::Error;

pub type ConcreteRequest = Request<Vec<u8>>;

pub type EndpointResult = Result<Response<Vec<u8>>, Error>;

pub struct Endpoint<const N_METHODS: usize, TCtx, TEndpoint>
where
    TCtx: Clone + Send + 'static,
    TEndpoint: for<'a> EndpointFunc<'a, TCtx>,
{
    // Theoretically, max is 9 but I added an extra one to be safe.
    pub(crate) methods: [Method; N_METHODS],
    pub(crate) ctx: TCtx,
    pub(crate) endpoint: TEndpoint,
}

impl<const N_METHODS: usize, TCtx, TEndpoint> Endpoint<N_METHODS, TCtx, TEndpoint>
where
    TCtx: Clone + Send + 'static,
    TEndpoint: for<'a> EndpointFunc<'a, TCtx>,
{
    pub fn new(ctx: TCtx, methods: [Method; N_METHODS], endpoint: TEndpoint) -> Self {
        Self {
            methods,
            ctx,
            endpoint,
        }
    }
}

pub trait EndpointFunc<'a, TCtx>: Send + Clone + 'static {
    type Fut: Future<Output = EndpointResult> + Send;

    fn call(self, ctx: TCtx, req: ConcreteRequest, cookies: &'a mut CookieJar) -> Self::Fut;
}

impl<'a, TCtx, TFunc, TFut> EndpointFunc<'a, TCtx> for TFunc
where
    TFunc: Fn(TCtx, ConcreteRequest, &'a mut CookieJar) -> TFut + Clone + Send + 'static,
    TFut: Future<Output = EndpointResult> + Send,
{
    type Fut = TFut;

    fn call(self, ctx: TCtx, req: ConcreteRequest, cookies: &'a mut CookieJar) -> Self::Fut {
        self(ctx, req, cookies)
    }
}
