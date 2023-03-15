use std::sync::Arc;

use axum::{
    extract::State,
    routing::{on, MethodFilter},
    Router,
};
use http::{HeaderMap, Request, StatusCode};
use hyper::{body::to_bytes, Body};

use crate::{Endpoint, HttpEndpoint, HttpResponse, Server};

pub use axum;

impl<TEndpoint> Endpoint<TEndpoint>
where
    TEndpoint: HttpEndpoint,
{
    /// is called to mount the endpoint onto an Axum router.
    pub fn axum<S>(mut self) -> Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        let (url, methods) = self.endpoint.register();
        let endpoint = Arc::new(self.endpoint);

        let mut method_filter = MethodFilter::empty();
        for method in methods.as_ref().iter() {
            #[allow(clippy::unwrap_used)] // TODO: Error handling
            method_filter.insert(MethodFilter::try_from(method.clone()).unwrap());
        }

        Router::<S>::new().route(
            url.as_ref(),
            on(
                method_filter,
                |state: State<S>, request: Request<Body>| async move {
                    let (mut parts, body) = request.into_parts();
                    parts.extensions.insert(state);

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
                        .handler(crate::Request::new(body, Server::Axum))
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
                },
            ),
        )
    }
}

impl crate::Request {
    /// TODO
    pub fn get_axum_state<S>(&self) -> Option<&S>
    where
        S: Clone + Send + Sync + 'static,
    {
        self.extensions().get::<State<S>>().map(|state| &state.0)
    }
}
