use cookie::CookieJar;
use http::{Method, Request, Response};

use crate::{EndpointFn, Error};

/// is a type alias for the request type of an endpoint.
pub type ConcreteRequest = Request<Vec<u8>>;

/// is a type alias for the response type of an endpoint.
pub type EndpointResult = Result<Response<Vec<u8>>, Error>;

/// is a endpoint defined on the http router corresponding to a specific URL. An endpoint may handle any number of HTTP methods.
/// Your library should create an HttpEndpoint and return it to the user so they can register it with the HTTP router of the web framework they are using.
/// For most use cases you will want [GenericHttpEndpoint](httpz::GenericHttpEndpoint) instead of implementing this trait yourself.
pub trait HttpEndpoint: Sized + Sync + Send + 'static {
    /// the type of your own context struct. This is used so you can get data from your application into the HTTP handler.
    type Ctx;
    /// the type of your routes array. This allows the user to return either [Vec<http::Method>], [&[http::Method]] or [[http::Method; N]].
    type Routes: AsRef<[Method]>;
    /// the type of the function to handle the endpoint.
    type EndpointFn: for<'a> EndpointFn<'a, Self::Ctx>;

    /// is called once and tells the router what HTTP methods this endpoint will handle.
    fn register(&mut self) -> Self::Routes;

    /// is called for every request and returns a future that will return the HttpResponse.
    fn handler<'a, 'b: 'a>(
        &'a self,
        req: ConcreteRequest,
        cookies: &'b mut CookieJar,
    ) -> <Self::EndpointFn as EndpointFn<'_, Self::Ctx>>::Fut;
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
}
