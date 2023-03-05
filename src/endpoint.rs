use std::sync::Arc;

use http::Method;

use crate::{EndpointFn, Request};

/// is a endpoint defined on the http router corresponding to a specific URL. An endpoint may handle any number of HTTP methods.
/// Your library should create an HttpEndpoint and return it to the user so they can register it with the HTTP router of the web framework they are using.
/// For most use cases you will want [GenericHttpEndpoint](httpz::GenericHttpEndpoint) instead of implementing this trait yourself.
pub trait HttpEndpoint: Sized + Sync + Send + 'static {
    /// the type of your routes array. This allows the user to return either [Vec<http::Method>], [&[http::Method]] or [[http::Method; N]].
    type Routes: AsRef<[Method]> + Send;
    /// the type of the URL string
    type Url: AsRef<str>;
    /// the type of the function to handle the endpoint.
    type EndpointFn: for<'a> EndpointFn<'a>;

    /// is called once and tells the router what HTTP methods this endpoint will handle.
    fn register(&mut self) -> (Self::Url, Self::Routes);

    /// is called for every request and returns a future that will return the HttpResponse.
    fn handler(&self, req: Request) -> <Self::EndpointFn as EndpointFn<'_>>::Fut;
}

/// is a generic HTTP endpoint. This wraps around the [HttpEndpoint](httpz::HttpEndpoint) trait providing support for each of the HTTP servers without the HttpEndpoint trait needing to be imported into the code intended to use the endpoint.
pub struct Endpoint<TEndpoint>
where
    TEndpoint: HttpEndpoint,
{
    /// the endpoint which is being wrapped.
    pub endpoint: TEndpoint,
}

impl<TEndpoint> Endpoint<TEndpoint>
where
    TEndpoint: HttpEndpoint,
{
    /// create a new endpoint from a [HttpEndpoint](httpz::HttpEndpoint).
    pub fn from_endpoint(endpoint: TEndpoint) -> Self {
        Endpoint { endpoint }
    }

    /// Shortcut to arc the endpoint.
    pub fn arced(self) -> Arc<Self> {
        Arc::new(self)
    }
}
