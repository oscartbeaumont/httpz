use std::str::FromStr;

use cookie::{Cookie, CookieJar};
use http::{
    header::{HeaderName, COOKIE, SET_COOKIE},
    uri::InvalidUri,
    HeaderValue, Request, StatusCode,
};
use worker::ResponseBody;

use crate::{Endpoint, HttpEndpoint};

impl<TEndpoint> Endpoint<TEndpoint>
where
    TEndpoint: HttpEndpoint,
{
    /// is called to handle the request in a Cloudflare worker
    pub async fn workers(self, mut request: worker::Request) -> worker::Result<worker::Response> {
        // let methods = self.endpoint.register(); // TODO: Handle HTTP methods

        let mut req = Request::new(request.bytes().await?);
        *req.method_mut() = match request.method() {
            worker::Method::Get => http::Method::GET,
            worker::Method::Post => http::Method::POST,
            worker::Method::Put => http::Method::PUT,
            worker::Method::Delete => http::Method::DELETE,
            worker::Method::Head => http::Method::HEAD,
            worker::Method::Connect => http::Method::CONNECT,
            worker::Method::Options => http::Method::OPTIONS,
            worker::Method::Trace => http::Method::TRACE,
            worker::Method::Patch => http::Method::PATCH,
        };
        *req.uri_mut() = request
            .url()?
            .as_str()
            .try_into()
            .map_err(|err: InvalidUri| worker::Error::RustError(err.to_string()))?;
        // *req.version_mut() = ; // TODO: Does Cloudflare not give us this?
        for (k, v) in request.headers() {
            req.headers_mut().insert(
                HeaderName::from_str(&k)
                    .map_err(|err| worker::Error::RustError(err.to_string()))?,
                HeaderValue::from_str(&v)
                    .map_err(|err| worker::Error::RustError(err.to_string()))?,
            );
        }
        // *req.extensions_mut() = request.extensions().get_mut() // TODO: Pass extensions through

        let mut cookies = CookieJar::new();
        for cookie in req
            .headers()
            .get_all(COOKIE)
            .into_iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.split(';'))
            .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok())
        {
            cookies.add_original(cookie);
        }

        match self.endpoint.handler(req, cookies).await {
            Ok((resp, cookies)) => {
                let (mut parts, body) = resp.into_parts();
                for cookie in cookies.delta() {
                    if let Ok(header_value) = cookie.encoded().to_string().parse() {
                        parts.headers.append(SET_COOKIE, header_value);
                    }
                }

                worker::Response::from_body(ResponseBody::Body(body)).map(|r| {
                    r.with_status(parts.status.as_u16())
                        .with_headers(parts.headers.into())
                })
            }
            Err(err) => {
                worker::Response::error(err.to_string(), StatusCode::INTERNAL_SERVER_ERROR.as_u16())
            }
        }
    }
}
