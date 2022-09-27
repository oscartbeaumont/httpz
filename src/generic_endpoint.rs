use cookie::CookieJar;
use http::Method;
use std::future::Future;

use crate::{ConcreteRequest, Endpoint, EndpointResult, HttpEndpoint};

/// represents an async function used to handle the HTTP request. I would highly recommend using [GenericHttpEndpoint] instead of implementing this trait yourself.
pub trait EndpointFn<'this, TCtx>
where
    Self: Send + Sync + 'this,
{
    /// the type of the future returned by the handler function.
    type Fut: Future<Output = EndpointResult> + Send + 'this;

    /// is called to handle the HTTP request.
    fn call(&self, ctx: TCtx, req: ConcreteRequest, cookies: CookieJar) -> Self::Fut;
}

impl<'this, TCtx, TFut, TFunc> EndpointFn<'this, TCtx> for TFunc
where
    TFunc: Fn(TCtx, ConcreteRequest, CookieJar) -> TFut + Send + Sync + 'static,
    TFut: Future<Output = EndpointResult> + Send + 'this,
{
    type Fut = TFut;

    fn call(&self, ctx: TCtx, req: ConcreteRequest, cookies: CookieJar) -> Self::Fut {
        self(ctx, req, cookies)
    }
}

/// is an easy way of constructing an endpoint from an async function you provide.
pub struct GenericEndpoint<TCtx, TMethods, TEndpointFn>
where
    TCtx: Sync + Send + 'static,
    TMethods: AsRef<[Method]> + Send + Sync + 'static,
    TEndpointFn: for<'this> EndpointFn<'this, TCtx>,
{
    ctx: TCtx,
    methods: Option<TMethods>,
    func: TEndpointFn,
}
impl<TCtx, TMethods, TEndpointFn> GenericEndpoint<TCtx, TMethods, TEndpointFn>
where
    TCtx: Sync + Send + Clone + 'static,
    TMethods: AsRef<[Method]> + Send + Sync + 'static,
    TEndpointFn: for<'this> EndpointFn<'this, TCtx>,
{
    /// create a new [Endpoint] from a context, a list of methods and a function to handle the request.
    pub fn new(ctx: TCtx, methods: TMethods, func: TEndpointFn) -> Endpoint<Self> {
        Endpoint::from_endpoint(Self::new_raw(ctx, methods, func))
    }

    /// create a new generic endpoint from a context, a list of methods and a function to handle the request.
    pub fn new_raw(ctx: TCtx, methods: TMethods, func: TEndpointFn) -> Self {
        Self {
            ctx,
            methods: Some(methods),
            func,
        }
    }
}

impl<TCtx, TMethods, TEndpointFn> HttpEndpoint for GenericEndpoint<TCtx, TMethods, TEndpointFn>
where
    TCtx: Sync + Send + Clone + 'static,
    TMethods: AsRef<[Method]> + Send + Sync + 'static,
    TEndpointFn: for<'this> EndpointFn<'this, TCtx>,
{
    type Ctx = TCtx;
    type Routes = TMethods;
    type EndpointFn = TEndpointFn;

    fn register(&mut self) -> Self::Routes {
        match self.methods.take() {
            Some(methods) => methods,
            None => unreachable!(),
        }
    }

    fn handler<'a, 'b: 'a>(
        &'a self,
        req: ConcreteRequest,
        cookies: CookieJar,
    ) -> <Self::EndpointFn as EndpointFn<'_, TCtx>>::Fut {
        self.func.call(self.ctx.clone(), req, cookies)
    }
}
