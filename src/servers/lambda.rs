use http::StatusCode;
use lambda_http::{service_fn, Body, Error, Request, Response};
use std::sync::Arc;

use crate::{Endpoint, HttpEndpoint, HttpResponse, Server};

impl<TEndpoint> Endpoint<TEndpoint>
where
    TEndpoint: HttpEndpoint,
{
    /// is called to mount the endpoint onto the AWS lambda runtime.
    pub async fn lambda(mut self) -> Result<(), Error> {
        let (_url, methods) = self.endpoint.register();
        // TODO: Handle `_url`??

        let endpoint = Arc::new(self.endpoint);

        lambda_http::run(service_fn(move |request: Request| {
            let endpoint = endpoint.clone();
            let is_correct_method = methods.as_ref().contains(request.method());

            async move {
                if !is_correct_method {
                    return Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body("Method Not Allowed".to_string().into());
                }

                let (parts, body) = request.into_parts();
                let fut = endpoint.handler(crate::Request(
                    parts,
                    match body {
                        Body::Empty => vec![],
                        Body::Text(text) => text.into_bytes(),
                        Body::Binary(binary) => binary,
                    },
                    Server::Lambda,
                ));

                match fut.await.into_response() {
                    Ok(resp) => {
                        let (parts, body) = resp.into_parts();
                        Ok(Response::from_parts(parts, body.into()))
                    }
                    Err(err) => Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("content-type", "text/html")
                        .body(Body::Text(err.to_string())),
                }
            }
        }))
        .await
    }
}
