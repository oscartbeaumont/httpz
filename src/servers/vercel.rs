use http::{Response, StatusCode};
use std::sync::Arc;
use vercel_runtime::{Body, Error};

use crate::{Endpoint, HttpEndpoint, HttpResponse, Server};

impl<TEndpoint> Endpoint<TEndpoint>
where
    TEndpoint: HttpEndpoint,
{
    /// is called to mount the endpoint onto the Vercel runtime.
    pub async fn vercel(mut self) -> Result<(), Error> {
        let (_url, methods) = self.endpoint.register();
        // TODO: Handle `_url`??

        let endpoint = Arc::new(self.endpoint);

        vercel_runtime::run(|request| {
            let endpoint = endpoint.clone();
            let methods = &methods;
            async move {
                let is_correct_method = methods.as_ref().contains(request.method());

                if !is_correct_method {
                    return Ok(Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body("Method Not Allowed".to_string().into())?);
                }

                let (parts, body) = request.into_parts();
                let fut = endpoint.handler(crate::Request(
                    parts,
                    match body {
                        Body::Empty => vec![],
                        Body::Text(text) => text.into_bytes(),
                        Body::Binary(binary) => binary,
                    },
                    Server::Vercel,
                ));

                match fut.await.into_response() {
                    Ok(resp) => {
                        let (parts, body) = resp.into_parts();
                        Ok(Response::from_parts(parts, body.into()))
                    }
                    Err(err) => Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("content-type", "text/html")
                        .body(err.to_string().into())?),
                }
            }
        })
        .await
    }
}
