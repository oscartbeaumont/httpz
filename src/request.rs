use http::{request::Parts, HeaderMap, Method, Uri, Version};

use crate::Server;

/// Represent a HTTP request
#[derive(Debug)]
pub struct Request(pub(crate) Parts, pub(crate) Vec<u8>, pub(crate) Server);

impl Request {
    /// Create a new [Request] from a [http::Request] and a [httpz::Server].
    pub fn new(req: http::Request<Vec<u8>>, server: Server) -> Self {
        let (parts, body) = req.into_parts();
        Self(parts, body, server)
    }

    /// Get the uri of the request.
    pub fn uri(&self) -> &Uri {
        &self.0.uri
    }

    /// Get the version of the request.
    pub fn version(&self) -> Version {
        self.0.version
    }

    /// Get the method of the request.
    pub fn method(&self) -> &Method {
        &self.0.method
    }

    /// Get the headers of the request.
    pub fn headers(&self) -> &HeaderMap {
        &self.0.headers
    }

    /// Get the headers of the request.
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.0.headers
    }

    /// Get the body of the request.
    pub fn body(&self) -> &Vec<u8> {
        &self.1
    }

    /// Get a new [CookieJar] which is derived from the cookies in the request.
    #[cfg(feature = "cookies")]
    pub fn cookies(&self) -> cookie::CookieJar {
        use {
            cookie::{Cookie, CookieJar},
            http::header::COOKIE,
        };

        let mut jar = CookieJar::new();
        for cookie in self
            .0
            .headers
            .get_all(COOKIE)
            .into_iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.split(';'))
            .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok())
        {
            jar.add_original(cookie);
        }

        jar
    }

    /// query_pairs returns an iterator of the query parameters.
    pub fn query_pairs(&self) -> Option<form_urlencoded::Parse<'_>> {
        self.0
            .uri
            .query()
            .map(|query| form_urlencoded::parse(query.as_bytes()))
    }

    /// split the [http::Parts]] and the body
    pub fn into_parts(self) -> (Parts, Vec<u8>) {
        (self.0, self.1)
    }

    /// get the [http::Parts] of the request
    pub fn parts(&self) -> &Parts {
        &self.0
    }

    /// get the [http::Parts] of the request
    pub fn parts_mut(&mut self) -> &mut Parts {
        &mut self.0
    }

    /// expose the inner [http::Request]
    pub fn expose(self) -> http::Request<Vec<u8>> {
        http::Request::from_parts(self.0, self.1)
    }

    /// Get the extensions of the request.
    pub fn extensions(&self) -> &http::Extensions {
        &self.0.extensions
    }

    /// Get the extensions of the request.
    pub fn extensions_mut(&mut self) -> &mut http::Extensions {
        &mut self.0.extensions
    }

    /// get the type of the server that handled this request
    pub fn server(&self) -> Server {
        self.2
    }

    // TODO: Remove usage of this from rspc and then here.
    #[doc(hidden)]
    pub fn _internal_dangerously_clone(&self) -> Self {
        let mut parts = http::Request::<()>::default().into_parts().0;
        parts.method = self.0.method.clone();
        parts.uri = self.0.uri.clone();
        parts.version = self.0.version;
        parts.headers = self.0.headers.clone();
        // parts.extensions = self.0.extensions().clone(); // TODO: Can't `Clone` extensions. Hence why this method is dangerous.

        Self(parts, self.1.clone(), self.2)
    }
}
