use actix_web::{
    guard::{self},
    web::{self, Bytes},
    HttpRequest, HttpResponse as ActixHttpResponse, Resource,
};
use http::{header::HeaderName, Method, Request, StatusCode};

use crate::{Endpoint, HttpEndpoint, HttpResponse};

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
        // TODO: Handle HTTP methods
        let mut endpoint = self.0.clone();
        let (url, methods) = endpoint.register();
        let mut methods = methods.as_ref().iter();

        #[allow(unused_assignments)]
        let mut method_filter = None;
        match methods.next() {
            Some(method) if method == Method::GET => method_filter = Some(guard::Any(guard::Get())),
            Some(method) if method == Method::POST => {
                method_filter = Some(guard::Any(guard::Post()))
            }
            Some(method) if method == Method::PUT => method_filter = Some(guard::Any(guard::Put())),
            Some(method) if method == Method::DELETE => {
                method_filter = Some(guard::Any(guard::Delete()))
            }
            Some(method) if method == Method::HEAD => {
                method_filter = Some(guard::Any(guard::Head()))
            }
            Some(method) if method == Method::OPTIONS => {
                method_filter = Some(guard::Any(guard::Options()))
            }
            Some(method) if method == Method::CONNECT => {
                method_filter = Some(guard::Any(guard::Connect()))
            }
            Some(method) if method == Method::TRACE => {
                method_filter = Some(guard::Any(guard::Trace()))
            }
            Some(method) if method == Method::PATCH => {
                method_filter = Some(guard::Any(guard::Patch()))
            }
            Some(_) => unreachable!(),
            None => todo!(),
        }

        let method_filter = if let Some(mut method_filter) = method_filter {
            for method in methods {
                match *method {
                    Method::GET => method_filter = method_filter.or(guard::Get()),
                    Method::POST => method_filter = method_filter.or(guard::Post()),
                    Method::PUT => method_filter = method_filter.or(guard::Put()),
                    Method::DELETE => method_filter = method_filter.or(guard::Delete()),
                    Method::HEAD => method_filter = method_filter.or(guard::Head()),
                    Method::OPTIONS => method_filter = method_filter.or(guard::Options()),
                    Method::CONNECT => method_filter = method_filter.or(guard::Connect()),
                    Method::TRACE => method_filter = method_filter.or(guard::Trace()),
                    Method::PATCH => method_filter = method_filter.or(guard::Patch()),
                    _ => unreachable!(),
                }
            }
            method_filter
        } else {
            unreachable!();
        };

        web::resource(url.as_ref()).guard(method_filter).to(
            move |request: HttpRequest, body: Bytes| {
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

                    match endpoint.handler(crate::Request(req)).await.into_response() {
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
            },
        )
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
