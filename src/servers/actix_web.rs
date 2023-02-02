use actix_web::{
    guard::{Guard, GuardContext},
    web::{self, Bytes},
    HttpRequest, HttpResponse as ActixHttpResponse, Resource,
};
use http::{header::HeaderName, Request, StatusCode};

use crate::{Endpoint, HttpEndpoint, HttpResponse, Server};

/// TODO
pub struct ActixMounter<TEndpoint: HttpEndpoint>(TEndpoint)
where
    TEndpoint: HttpEndpoint + Clone;

impl<TEndpoint> Clone for ActixMounter<TEndpoint>
where
    TEndpoint: HttpEndpoint + Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<TEndpoint> ActixMounter<TEndpoint>
where
    TEndpoint: HttpEndpoint + Clone,
{
    /// TODO
    pub fn mount(&self) -> Resource {
        let mut endpoint = self.0.clone();
        let (url, routes) = endpoint.register();

        web::resource(url.as_ref())
            .guard(MethodGuard::<TEndpoint>(routes))
            .to(move |request: HttpRequest, body: Bytes| {
                let endpoint = endpoint.clone();
                async move {
                    let mut req = Request::new(body.to_vec());
                    // TODO: Reducing the cloning here
                    *req.method_mut() = request.method().clone();
                    *req.uri_mut() = request.uri().clone();
                    *req.version_mut() = request.version().clone();
                    for (k, v) in request.headers() {
                        req.headers_mut().insert(HeaderName::from(k), v.clone());
                    }
                    // *req.extensions_mut() = request.extensions().get_mut() // TODO: Pass extensions through

                    match endpoint
                        .handler(crate::Request(req, Server::ActixWeb))
                        .await
                        .into_response()
                    {
                        Ok(resp) => {
                            let (parts, body) = resp.into_parts();
                            let mut resp = ActixHttpResponse::new(parts.status);
                            for (k, v) in parts.headers {
                                if let Some(k) = k {
                                    resp.headers_mut().insert(HeaderName::from(k), v);
                                }
                            }
                            resp.set_body(body)
                        }
                        Err(err) => ActixHttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
                            .set_body(err.to_string().as_bytes().to_vec()),
                    }
                }
            })
    }
}

impl<TEndpoint> Endpoint<TEndpoint>
where
    TEndpoint: HttpEndpoint + Clone,
{
    /// is called to create a builder for mounting this endpoint to your actix-web router.
    pub fn actix(self) -> ActixMounter<TEndpoint> {
        ActixMounter(self.endpoint)
    }
}

struct MethodGuard<TEndpoint: HttpEndpoint>(TEndpoint::Routes);

impl<TEndpoint: HttpEndpoint> Guard for MethodGuard<TEndpoint> {
    fn check(&self, ctx: &GuardContext<'_>) -> bool {
        for method in self.0.as_ref().iter() {
            if method == ctx.head().method {
                return true;
            }
        }

        false
    }
}
