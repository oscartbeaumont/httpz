// use cookie::CookieJar;
// use httpz::{
//     cookie::Cookie,
//     http::{Method, Response, StatusCode},
//     ConcreteRequest, Endpoint, EndpointResult, GenericEndpoint, HttpEndpoint,
// };
// use std::future::Future;

// #[derive(Debug, Clone)]
// pub struct RequestAssertion {
//     pub(crate) method: Method,
//     pub(crate) uri: &'static str,
//     pub(crate) headers: Vec<(&'static str, &'static str)>,
//     pub(crate) cookies: Vec<Cookie<'static>>,
//     pub(crate) body: Option<&'static [u8]>,
// }

// #[allow(unused)] // Linter is being weird, this should be implied
// pub struct ResponseAssertion {
//     pub(crate) status: StatusCode,
//     pub(crate) headers: Vec<(&'static str, &'static str)>,
//     pub(crate) cookies: Vec<Cookie<'static>>,
//     pub(crate) body: Option<Vec<u8>>,
// }

// async fn handler(
//     (this, resp_status, resp_headers, resp_cookies, resp_body): (
//         RequestAssertion,
//         StatusCode,
//         Vec<(&'static str, &'static str)>,
//         Vec<Cookie<'static>>,
//         Option<Vec<u8>>,
//     ),
//     req: ConcreteRequest,
//     cookies: &mut CookieJar,
// ) -> EndpointResult {
//     assert_eq!(this.method, req.method());
//     assert_eq!(this.uri, req.uri());
//     assert_eq!(
//         this.headers
//             .into_iter()
//             .map(|(k, v)| format!("{}:{}", k.to_lowercase(), v))
//             .collect::<Vec<_>>()
//             .join(","),
//         req.headers()
//             .into_iter()
//             .filter(|(k, _)| k.as_str() != "cookie")
//             .map(|(k, v)| format!("{}:{}", k.as_str(), v.to_str().unwrap()))
//             .collect::<Vec<_>>()
//             .join(","),
//     );
//     assert_eq!(
//         this.cookies
//             .into_iter()
//             .map(|c| format!("{}={}", c.name(), c.value()))
//             .collect::<Vec<_>>()
//             .join("; "),
//         {
//             let mut cookies = cookies.iter().collect::<Vec<_>>();
//             cookies.sort_by(|a, b| a.name().cmp(b.name()));
//             cookies
//                 .into_iter()
//                 .map(|c| format!("{}={}", c.name(), c.value()))
//                 .collect::<Vec<_>>()
//                 .join("; ")
//         }
//     );

//     if let Some(body) = this.body {
//         assert_eq!(body, req.body());
//     }

//     let mut resp = Response::builder().status(resp_status);

//     for (k, v) in resp_headers {
//         resp = resp.header(k, v);
//     }

//     for c in resp_cookies {
//         cookies.add(c);
//     }

//     Ok(resp.body(resp_body.unwrap_or_default())?)
// }

// impl RequestAssertion {
//     pub fn new(
//         method: Method,
//         uri: &'static str,
//         headers: Vec<(&'static str, &'static str)>,
//         cookies: Vec<Cookie<'static>>,
//         body: Option<&'static [u8]>,
//     ) -> Self {
//         Self {
//             method,
//             uri,
//             headers,
//             cookies,
//             body,
//         }
//     }

//     pub fn endpoint<TMethods: AsRef<[Method]> + Send + Sync + 'static>(
//         &self,
//         methods: TMethods,
//         status: StatusCode,
//         headers: Vec<(&'static str, &'static str)>,
//         cookies: Vec<Cookie<'static>>,
//         body: Option<Vec<u8>>,
//     ) -> Endpoint<impl HttpEndpoint> {
//         GenericEndpoint::new(
//             (self.clone(), status, headers, cookies, body),
//             methods,
//             handler,
//         )
//     }
// }

// pub async fn run_http_test_suite<TFunc, TFut>(func: TFunc)
// where
//     TFunc: Fn(RequestAssertion, ResponseAssertion) -> TFut,
//     TFut: Future<Output = ()>,
// {
//     for method in [Method::GET, Method::POST, Method::PUT, Method::DELETE] {
//         for uri in ["/demo", "/httpz/demo"] {
//             for headers in [
//                 vec![],
//                 vec![("X-Demo", "TestingValue")],
//                 vec![
//                     ("X-Demo", "TestingValue"),
//                     ("X-Demo-2", "TestingValue2"),
//                     ("X-Demo-3", "TestingValue2"),
//                 ],
//             ] {
//                 for cookies in [
//                     vec![],
//                     vec![Cookie::new("DemoCookie", "TestingValue")],
//                     vec![
//                         Cookie::new("DemoCookie", "TestingValue"),
//                         Cookie::new("DemoCookie2", "TestingValue2"),
//                         Cookie::new("DemoCookie3", "TestingValue3"),
//                     ],
//                 ] {
//                     for body in [None, Some(b"Hello World".as_slice())] {
//                         for resp_status in [
//                             StatusCode::OK,
//                             StatusCode::BAD_REQUEST,
//                             StatusCode::INTERNAL_SERVER_ERROR,
//                         ] {
//                             for resp_headers in [
//                                 vec![],
//                                 vec![("X-Demo", "TestingValue")],
//                                 vec![
//                                     ("X-Demo", "TestingValue"),
//                                     ("X-Demo-2", "TestingValue2"),
//                                     ("X-Demo-3", "TestingValue2"),
//                                 ],
//                             ] {
//                                 for resp_cookies in [
//                                     vec![],
//                                     vec![Cookie::new("DemoCookie", "TestingValue")],
//                                     vec![
//                                         Cookie::new("DemoCookie", "TestingValue"),
//                                         Cookie::new("DemoCookie2", "TestingValue2"),
//                                         Cookie::new("DemoCookie3", "TestingValue3"),
//                                     ],
//                                 ] {
//                                     for resp_body in [None, Some(b"Hello World".to_vec())] {
//                                         func(
//                                             RequestAssertion {
//                                                 method: method.clone(),
//                                                 uri,
//                                                 headers: headers.clone(),
//                                                 cookies: cookies.clone(),
//                                                 body,
//                                             },
//                                             ResponseAssertion {
//                                                 status: resp_status,
//                                                 headers: resp_headers.clone(),
//                                                 cookies: resp_cookies.clone(),
//                                                 body: resp_body,
//                                             },
//                                         )
//                                         .await;
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

// pub async fn run_ws_test_suite() {
//     // TODO: Test websocket functionality
// }
