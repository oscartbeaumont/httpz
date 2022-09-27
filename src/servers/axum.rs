use std::sync::Arc;

use axum::routing::{on, MethodFilter, MethodRouter};
use cookie::{Cookie, CookieJar};
use http::{
    header::{COOKIE, SET_COOKIE},
    HeaderMap, Request, StatusCode,
};
use hyper::{body::to_bytes, Body};

use crate::{Endpoint, HttpEndpoint};

pub use axum;

impl<TEndpoint> Endpoint<TEndpoint>
where
    TEndpoint: HttpEndpoint,
{
    /// is called to mount the endpoint onto an Axum router.
    pub fn axum(mut self) -> MethodRouter {
        let methods = self.endpoint.register();
        let endpoint = Arc::new(self.endpoint);

        let mut method_filter = MethodFilter::empty();
        for method in methods.as_ref().iter() {
            // TODO: Error handling
            method_filter.insert(MethodFilter::try_from(method.clone()).unwrap());
        }

        on(method_filter, |request: Request<Body>| async move {
            let mut cookies = CookieJar::new();
            for cookie in request
                .headers()
                .get_all(COOKIE)
                .into_iter()
                .filter_map(|value| value.to_str().ok())
                .flat_map(|value| value.split(';'))
                .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok())
            {
                cookies.add_original(cookie);
            }

            let (parts, body) = request.into_parts();
            match endpoint
                .handler(
                    // TODO: Error handling on incoming body
                    Request::from_parts(parts, to_bytes(body).await.unwrap().to_vec()),
                    cookies,
                )
                .await
            {
                Ok((resp, cookies)) => {
                    let (mut parts, body) = resp.into_parts();
                    for cookie in cookies.delta() {
                        if let Ok(header_value) = cookie.encoded().to_string().parse() {
                            parts.headers.append(SET_COOKIE, header_value);
                        }
                    }
                    (parts.status, parts.headers, body)
                }
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    HeaderMap::new(),
                    err.to_string().as_bytes().to_vec(),
                ),
            }
        })
    }
}
