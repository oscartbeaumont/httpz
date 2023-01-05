use http::{request::Parts, HeaderMap, Method, Version};

/// TODO
pub struct Request(pub(crate) http::Request<Vec<u8>>);

impl Request {
    pub(crate) fn new(req: http::Request<Vec<u8>>) -> Self {
        Self(req)
    }

    /// Get the version of the request.
    pub fn version(&self) -> Version {
        self.0.version()
    }

    /// Get the method of the request.
    pub fn method(&self) -> &Method {
        self.0.method()
    }

    /// Get the path of the request.
    pub fn path(&self) -> &str {
        self.0.uri().path()
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

    /// TODO
    pub fn into_parts(self) -> (Parts, Vec<u8>) {
        self.0.into_parts()
    }

    /// TODO
    pub fn expose(self) -> http::Request<Vec<u8>> {
        self.0
    }

    // TODO: Downcasting extensions both `mut` and `ref`
    // TODO: Inserting extensions
}
