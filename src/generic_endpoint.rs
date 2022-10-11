use http::Method;
use std::future::Future;

use crate::{Endpoint, HttpEndpoint, HttpResponse, Request};

/// represents an async function used to handle the HTTP request. I would highly recommend using [GenericHttpEndpoint] instead of implementing this trait yourself.
pub trait EndpointFn<'this>
where
    Self: Send + Sync + 'this,
{
    /// TODO
    type Response: HttpResponse;

    /// the type of the future returned by the handler function.
    type Fut: Future<Output = Self::Response> + Send + 'this;

    /// is called to handle the HTTP request.
    fn call(&self, req: Request) -> Self::Fut;
}

impl<'this, TFut, TFunc, TRes> EndpointFn<'this> for TFunc
where
    TFunc: Fn(Request) -> TFut + Send + Sync + 'this,
    TFut: Future<Output = TRes> + Send + 'this,
    TRes: HttpResponse,
{
    type Response = TRes;
    type Fut = TFut;

    fn call(&self, req: Request) -> Self::Fut {
        self(req)
    }
}

/// is an easy way of constructing an endpoint from an async function you provide.
pub struct GenericEndpoint<TMethods, TEndpointFn>
where
    TMethods: AsRef<[Method]> + Send + Sync + 'static,
    TEndpointFn: for<'this> EndpointFn<'this>,
{
    methods: Option<TMethods>,
    func: TEndpointFn,
}
impl<TMethods, TEndpointFn> GenericEndpoint<TMethods, TEndpointFn>
where
    TMethods: AsRef<[Method]> + Send + Sync + 'static,
    TEndpointFn: for<'this> EndpointFn<'this>,
{
    /// create a new [Endpoint] from a context, a list of methods and a function to handle the request.
    pub fn new(methods: TMethods, func: TEndpointFn) -> Endpoint<Self> {
        Endpoint::from_endpoint(Self::new_raw(methods, func))
    }

    /// create a new generic endpoint from a context, a list of methods and a function to handle the request.
    pub fn new_raw(methods: TMethods, func: TEndpointFn) -> Self {
        Self {
            methods: Some(methods),
            func,
        }
    }
}

impl<TMethods, TEndpointFn> HttpEndpoint for GenericEndpoint<TMethods, TEndpointFn>
where
    TMethods: AsRef<[Method]> + Send + Sync + 'static,
    TEndpointFn: for<'this> EndpointFn<'this>,
{
    type Routes = TMethods;
    type EndpointFn = TEndpointFn;

    fn register(&mut self) -> Self::Routes {
        match self.methods.take() {
            Some(methods) => methods,
            None => unreachable!(),
        }
    }

    fn handler(&self, req: Request) -> <Self::EndpointFn as EndpointFn<'_>>::Fut {
        self.func.call(req)
    }
}
