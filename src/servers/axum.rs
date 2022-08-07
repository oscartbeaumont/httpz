use axum::{
    body::Bytes,
    routing::{on, MethodFilter, MethodRouter},
};
use cookie::{Cookie, CookieJar};
use http::{
    header::{COOKIE, SET_COOKIE},
    request::Parts,
    HeaderMap, Request, StatusCode,
};

use crate::{Endpoint, EndpointFunc};

impl<const N_METHODS: usize, TCtx, TEndpoint> Endpoint<N_METHODS, TCtx, TEndpoint>
where
    TCtx: Clone + Send + 'static,
    TEndpoint: for<'a> EndpointFunc<'a, TCtx>,
{
    pub fn axum(self) -> MethodRouter {
        let mut methods_iter = self.methods.into_iter();
        let method_filter = if let Some(method) = methods_iter.next() {
            let mut method_filter = MethodFilter::try_from(method).unwrap();
            for method in methods_iter {
                method_filter = method_filter | MethodFilter::try_from(method).unwrap();
            }
            method_filter
        } else {
            MethodFilter::empty()
        };

        on(method_filter, move |parts: Parts, body: Bytes| async move {
            let mut cookies = CookieJar::new();
            for cookie in parts
                .headers
                .get_all(COOKIE)
                .into_iter()
                .filter_map(|value| value.to_str().ok())
                .flat_map(|value| value.split(';'))
                .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok())
            {
                cookies.add_original(cookie);
            }

            match self
                .endpoint
                .call(
                    self.ctx,
                    Request::from_parts(parts, body.to_vec()),
                    &mut cookies,
                )
                .await
            {
                Ok(resp) => {
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

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, Response, StatusCode},
        Router,
    };
    use cookie::{Cookie, CookieJar};
    use http::{HeaderValue, Method};
    #[cfg(feature = "axum")]
    use tower::util::ServiceExt;

    use crate::{ConcreteRequest, Endpoint, EndpointResult};

    #[derive(Clone)]
    pub struct Context {
        req_method: Method,
        req_uri: &'static str,
        req_headers: Vec<(&'static str, &'static str)>,
        req_cookies: Vec<Cookie<'static>>,
        req_body: Option<Vec<u8>>,

        resp_status: u16,
        resp_headers: Vec<(&'static str, &'static str)>,
        resp_cookies: Vec<Cookie<'static>>,
        resp_body: Vec<u8>,
    }

    async fn handler(
        ctx: Context,
        req: ConcreteRequest,
        cookies: &mut CookieJar,
    ) -> EndpointResult {
        // Check Request
        assert_eq!(req.method(), ctx.req_method);
        assert_eq!(*req.uri(), *ctx.req_uri);
        for (key, value) in ctx.req_headers {
            assert_eq!(
                req.headers().get(key).as_deref(),
                Some(&HeaderValue::from_static(value))
            );
        }
        for cookie in ctx.req_cookies {
            assert_eq!(cookies.get(cookie.name()).unwrap().value(), cookie.value());
        }
        if let Some(body) = ctx.req_body {
            assert_eq!(*req.body(), body);
        }

        // Create response
        let mut resp = Response::builder().status(StatusCode::from_u16(ctx.resp_status).unwrap());
        for cookie in ctx.resp_cookies {
            cookies.add(cookie);
        }
        for (key, value) in ctx.resp_headers {
            resp = resp.header(key, value);
        }
        Ok(resp.body(ctx.resp_body)?)
    }

    #[tokio::test]
    async fn test_axum_get() {
        let endpoint = Endpoint::new(
            Context {
                req_method: Method::GET,
                req_uri: "/axum_get",
                req_headers: vec![("X-Request", "HelloWorld")],
                req_cookies: vec![Cookie::new("incoming", "cookie")],
                req_body: None,

                resp_status: 200,
                resp_headers: vec![("X-Response", "HelloWorld")],
                resp_cookies: vec![Cookie::new("response", "cookie")],
                resp_body: b"Hello, World!".to_vec(),
            },
            [Method::GET],
            handler,
        );
        let router = <Router>::new().route("/axum_get", endpoint.axum());

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/axum_get")
                    .header("X-Request", "HelloWorld")
                    .header("Cookie", "incoming=cookie")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("X-Response").as_deref(),
            Some(&HeaderValue::from_static("HelloWorld"))
        );
        assert_eq!(
            response.headers().get("Set-Cookie").as_deref(),
            Some(&HeaderValue::from_static("response=cookie"))
        );

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"Hello, World!");
    }

    #[tokio::test]
    async fn test_axum_post() {
        let endpoint = Endpoint::new(
            Context {
                req_method: Method::POST,
                req_uri: "/axum_post",
                req_headers: vec![("X-One", "OneValue"), ("X-Two", "TwoValue")],
                req_cookies: vec![Cookie::new("one", "a"), Cookie::new("two", "b")],
                req_body: Some(b"Hello, World Request!".to_vec()),

                resp_status: 204,
                resp_headers: vec![("X-Resp-One", "OneValue"), ("X-Resp-Two", "TwoValue")],
                resp_cookies: vec![Cookie::new("oneresp", "a"), Cookie::new("tworesp", "b")],
                resp_body: vec![],
            },
            [Method::POST],
            handler,
        );
        let router = <Router>::new().route("/axum_post", endpoint.axum());

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/axum_post")
                    .header("X-One", "OneValue")
                    .header("X-Two", "TwoValue")
                    .header("Cookie", "one=a")
                    .header("Cookie", "two=b")
                    .body(Body::from(b"Hello, World Request!".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            response.headers().get("X-Resp-One").as_deref(),
            Some(&HeaderValue::from_static("OneValue"))
        );
        assert_eq!(
            response.headers().get("X-Resp-Two").as_deref(),
            Some(&HeaderValue::from_static("TwoValue"))
        );

        let mut iter = response.headers().get_all("Set-Cookie").into_iter();
        assert_eq!(
            iter.next().as_deref(),
            Some(&HeaderValue::from_static("oneresp=a"))
        );
        assert_eq!(
            iter.next().as_deref(),
            Some(&HeaderValue::from_static("tworesp=b"))
        );
        assert_eq!(iter.next(), None);
    }

    #[tokio::test]
    async fn test_axum_method_matching() {
        async fn handler(
            _ctx: (),
            _req: ConcreteRequest,
            _cookies: &mut CookieJar,
        ) -> EndpointResult {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(b"Hello, World Response!".to_vec())?)
        }

        let endpoint = Endpoint::new((), [Method::GET, Method::PATCH], handler);
        let router = <Router>::new().route("/axum", endpoint.axum());

        // Matched methods
        for method in [Method::GET, Method::PATCH] {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri("/axum")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            assert_eq!(&body[..], b"Hello, World Response!");
        }

        // Unmatched methods
        for method in [Method::POST, Method::PUT] {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri("/axum")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        }
    }
}
