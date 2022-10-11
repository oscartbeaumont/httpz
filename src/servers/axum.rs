use std::sync::Arc;

use axum::routing::{on, MethodFilter, MethodRouter};
use http::{HeaderMap, Request, StatusCode};
use hyper::{body::to_bytes, Body};

use crate::{Endpoint, HttpEndpoint, HttpResponse};

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
            let (parts, body) = request.into_parts();

            let body = match to_bytes(body).await {
                Ok(body) => body.to_vec(),
                Err(err) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        HeaderMap::new(),
                        err.to_string().as_bytes().to_vec(),
                    );
                }
            };

            let body = Request::from_parts(parts, body);

            match endpoint
                .handler(crate::Request::new(body))
                .await
                .into_response()
            {
                Ok(resp) => {
                    let (parts, body) = resp.into_parts();
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
