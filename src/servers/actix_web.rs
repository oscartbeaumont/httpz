use std::{mem, sync::Arc};

use actix_web::{
    guard::{self, AnyGuard},
    web::{self, Bytes},
    HttpRequest, HttpResponse, Route,
};
use cookie::{Cookie, CookieJar};
use http::{
    header::{HeaderName, COOKIE, SET_COOKIE},
    Method, Request, StatusCode,
};

use crate::{Endpoint, HttpEndpoint};

/// TODO
pub struct ActixMounter<TEndpoint>(Arc<TEndpoint>, Option<AnyGuard>)
where
    TEndpoint: HttpEndpoint;

impl<TEndpoint> Clone for ActixMounter<TEndpoint>
where
    TEndpoint: HttpEndpoint,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), unsafe { mem::transmute_copy(&self.1) }) // TODO: This is incredibly stupid. PR upstream for a better API.
    }
}

impl<TEndpoint> ActixMounter<TEndpoint>
where
    TEndpoint: HttpEndpoint,
{
    /// TODO
    pub fn mount(&self) -> Route {
        // TODO: Handle HTTP methods

        let endpoint = self.0.clone();

        let method_guard = guard::Any(guard::Get()).or(guard::Post());

        web::route()
            .guard(method_guard)
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

                    let mut cookies = CookieJar::new();
                    for cookie in req
                        .headers()
                        .get_all(COOKIE)
                        .into_iter()
                        .filter_map(|value| value.to_str().ok())
                        .flat_map(|value| value.split(';'))
                        .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok())
                    {
                        cookies.add_original(cookie);
                    }

                    match endpoint.handler(req, cookies).await {
                        Ok((resp, cookies)) => {
                            let (mut parts, body) = resp.into_parts();
                            for cookie in cookies.delta() {
                                if let Ok(header_value) = cookie.encoded().to_string().parse() {
                                    parts.headers.append(SET_COOKIE, header_value);
                                }
                            }

                            let mut resp = HttpResponse::new(parts.status);
                            for (k, v) in parts.headers {
                                if let Some(k) = k {
                                    resp.headers_mut().insert(HeaderName::from(k), v);
                                }
                            }
                            resp.set_body(body)
                        }
                        Err(err) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
                            .set_body(err.to_string().as_bytes().to_vec()),
                    }
                }
            })
    }
}

impl<TEndpoint> Endpoint<TEndpoint>
where
    TEndpoint: HttpEndpoint,
{
    /// is called to create a builder for mounting this endpoint to your actix-web router.
    pub fn actix(mut self) -> ActixMounter<TEndpoint> {
        #[warn(unused_assignments)]
        let mut method_filter = None;
        let methods = self.endpoint.register();
        let mut methods = methods.as_ref().iter();

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

        // let method_filter = if let Some(mut method_filter) = method_filter {
        //     for method in methods {
        //         match *method {
        //             Method::GET => method_filter = method_filter.or(guard::Get()),
        //             Method::POST => method_filter = method_filter.or(guard::Post()),
        //             Method::PUT => method_filter = method_filter.or(guard::Put()),
        //             Method::DELETE => method_filter = method_filter.or(guard::Delete()),
        //             Method::HEAD => method_filter = method_filter.or(guard::Head()),
        //             Method::OPTIONS => method_filter = method_filter.or(guard::Options()),
        //             Method::CONNECT => method_filter = method_filter.or(guard::Connect()),
        //             Method::TRACE => method_filter = method_filter.or(guard::Trace()),
        //             Method::PATCH => method_filter = method_filter.or(guard::Patch()),
        //             _ => unreachable!(),
        //         }
        //     }
        //     method_filter
        // } else {
        //     method_filter
        // };

        let method_filter = method_filter.map(|mut method_filter| {
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
        });

        ActixMounter(Arc::new(self.endpoint), method_filter)
    }
}
