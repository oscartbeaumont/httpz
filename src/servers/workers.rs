use std::str::FromStr;

use http::{header::HeaderName, uri::InvalidUri, HeaderValue, Request, StatusCode};
use worker::ResponseBody;

use crate::{Endpoint, HttpEndpoint, HttpResponse, Server};

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

        match self
            .endpoint
            .handler(crate::Request(req, Server::CloudflareWorkers))
            .await
            .into_response()
        {
            Ok(resp) => {
                let (parts, body) = resp.into_parts();
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
