use http::StatusCode;
use lambda_http::{Body, Request, Response, Service};
use std::{
    future::Future,
    sync::Arc,
    task::{Context, Poll},
};

use crate::{Endpoint, HttpEndpoint, HttpResponse, Server};

/// TODO
pub trait InternalTowerHandlerFunc<TEndpoint>: Fn(Arc<TEndpoint>, Request) -> Self::Fut
where
    TEndpoint: HttpEndpoint,
{
    /// TODO
    type Fut: Future<Output = Result<Response<Body>, http::Error>> + Send + 'static;
}

impl<TEndpoint, TFunc, TFut> InternalTowerHandlerFunc<TEndpoint> for TFunc
where
    TEndpoint: HttpEndpoint,
    TFunc: Fn(Arc<TEndpoint>, Request) -> TFut,
    TFut: Future<Output = Result<Response<Body>, http::Error>> + Send + 'static,
{
    type Fut = TFut;
}

/// TODO
#[derive(Debug)]
pub struct TowerEndpoint<TEndpoint, TFunc>(Arc<TEndpoint>, TFunc)
where
    TEndpoint: HttpEndpoint,
    TFunc: InternalTowerHandlerFunc<TEndpoint>;

impl<TEndpoint, TFunc> Service<Request> for TowerEndpoint<TEndpoint, TFunc>
where
    TEndpoint: HttpEndpoint,
    TFunc: InternalTowerHandlerFunc<TEndpoint>,
{
    type Response = Response<Body>;
    type Error = http::Error;
    type Future = <TFunc as InternalTowerHandlerFunc<TEndpoint>>::Fut;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request) -> Self::Future {
        (self.1)(self.0.clone(), request)
    }
}

impl<TEndpoint> Endpoint<TEndpoint>
where
    TEndpoint: HttpEndpoint,
{
    /// is called to mount the endpoint onto the AWS lambda runtime.
    pub fn lambda(mut self) -> TowerEndpoint<TEndpoint, impl InternalTowerHandlerFunc<TEndpoint>> {
        let (_url, methods) = self.endpoint.register();
        // TODO: Handle `_url`??

        TowerEndpoint(
            Arc::new(self.endpoint),
            is_send(move |endpoint: Arc<TEndpoint>, request: Request| {
                let is_correct_method = methods.as_ref().contains(request.method());

                is_send(async move {
                    if !is_correct_method {
                        return Response::builder()
                            .status(StatusCode::METHOD_NOT_ALLOWED)
                            .body("Method Not Allowed".into());
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
                            .body(err.to_string().into()),
                    }
                })
            }),
        )
    }
}

// Lambda runtime get's angry. These act as a safety net to prevent breaking the Lambda runtime inside user code.
#[inline]
fn is_send<T: Send>(t: T) -> T {
    t
}
