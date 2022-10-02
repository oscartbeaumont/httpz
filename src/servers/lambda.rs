use cookie::{Cookie, CookieJar};
use http::{
    header::{COOKIE, SET_COOKIE},
    StatusCode,
};
use lambda_http::{Body, Request, Response, Service};
use std::{
    future::Future,
    sync::Arc,
    task::{Context, Poll},
};

use crate::{Endpoint, HttpEndpoint};

/// TODO
pub trait InternalTowerHandlerFunc<TEndpoint>: Fn(Arc<TEndpoint>, Request) -> Self::Fut
where
    TEndpoint: HttpEndpoint,
{
    /// TODO
    type Fut: Future<Output = Result<Response<Body>, http::Error>>;
}

impl<TEndpoint, TFunc, TFut> InternalTowerHandlerFunc<TEndpoint> for TFunc
where
    TEndpoint: HttpEndpoint,
    TFunc: Fn(Arc<TEndpoint>, Request) -> TFut,
    TFut: Future<Output = Result<Response<Body>, http::Error>>,
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
        let methods = self.endpoint.register();

        TowerEndpoint(
            Arc::new(self.endpoint),
            move |endpoint: Arc<TEndpoint>, request: Request| {
                let is_correct_method = methods.as_ref().contains(request.method());

                async move {
                    if !is_correct_method {
                        return Response::builder()
                            .status(StatusCode::METHOD_NOT_ALLOWED)
                            .body("Method Not Allowed".into());
                    }

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
                    let fut = endpoint.handler(
                        // TODO: Error handling on incoming body
                        http::Request::from_parts(
                            parts,
                            match body {
                                Body::Empty => vec![],
                                Body::Text(text) => text.into_bytes(),
                                Body::Binary(binary) => binary,
                            },
                        ),
                        cookies,
                    );

                    match fut.await {
                        Ok((resp, cookies)) => {
                            let (mut parts, body) = resp.into_parts();
                            for cookie in cookies.delta() {
                                if let Ok(header_value) = cookie.encoded().to_string().parse() {
                                    parts.headers.append(SET_COOKIE, header_value);
                                }
                            }
                            Ok(Response::from_parts(parts, body.into()))
                        }
                        Err(err) => Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header("content-type", "text/html")
                            .body(err.to_string().into()),
                    }
                }
            },
        )
    }
}
