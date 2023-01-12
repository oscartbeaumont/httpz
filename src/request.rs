use http::{request::Parts, HeaderMap, Method, Uri, Version};

use crate::Server;

/// Represent a HTTP request
#[derive(Debug)]
pub struct Request(pub(crate) http::Request<Vec<u8>>, pub(crate) Server);

impl Clone for Request {
    fn clone(&self) -> Self {
        Self(
            http::Request::from_parts(
                http::Request::<Vec<u8>>::default().into_parts().0,
                self.body().clone(),
            ),
            self.1,
        )
    }
}

impl Request {
    pub(crate) fn new(req: http::Request<Vec<u8>>, server: Server) -> Self {
        Self(req, server)
    }

    /// Get the uri of the request.
    pub fn uri(&self) -> &Uri {
        self.0.uri()
    }

    /// Get the version of the request.
    pub fn version(&self) -> Version {
        self.0.version()
    }

    /// Get the method of the request.
    pub fn method(&self) -> &Method {
        self.0.method()
    }

    /// Get the headers of the request.
    pub fn headers(&self) -> &HeaderMap {
        self.0.headers()
    }

    /// Get the body of the request.
    pub fn body(&self) -> &Vec<u8> {
        self.0.body()
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
            .headers()
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
            .uri()
            .query()
            .map(|query| form_urlencoded::parse(query.as_bytes()))
    }

    /// split the [http::Parts]] and the body
    pub fn into_parts(self) -> (Parts, Vec<u8>) {
        self.0.into_parts()
    }

    /// expose the inner [http::Request]
    pub fn expose(self) -> http::Request<Vec<u8>> {
        self.0
    }

    /// get the type of the server that handled this request
    pub fn server(&self) -> Server {
        self.1
    }

    // TODO: Downcasting extensions both `mut` and `ref`
    // TODO: Inserting extensions
}
