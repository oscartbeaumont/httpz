// #[allow(unused)]
// mod utils;

// #[cfg(feature = "axum")]
// mod tests {
//     use super::utils::{run_http_test_suite, run_ws_test_suite};
//     use axum::{body::Body, http::Request, Router};
//     use hyper::body::to_bytes;
//     use tower::ServiceExt;

//     #[tokio::test]
//     async fn test_axum_http() {
//         run_http_test_suite(|req, resp| async move {
//             let endpoint = req.endpoint(
//                 [req.method.clone()],
//                 resp.status,
//                 resp.headers.clone(),
//                 resp.cookies.clone(),
//                 resp.body.clone(),
//             );

//             let router = <Router>::new().route(req.uri, endpoint.axum());

//             let mut request = Request::builder().method(req.method.clone()).uri(req.uri);

//             for (k, v) in req.headers {
//                 request = request.header(k, v);
//             }

//             let response = router
//                 .oneshot(
//                     request
//                         .header(
//                             "Cookie",
//                             req.cookies
//                                 .into_iter()
//                                 .map(|c| format!("{}={}", c.name(), c.value()))
//                                 .collect::<Vec<_>>()
//                                 .join("; "),
//                         )
//                         .body(match req.body {
//                             Some(body) => Body::from(body),
//                             None => Body::empty(),
//                         })
//                         .unwrap(),
//                 )
//                 .await
//                 .unwrap();

//             assert_eq!(resp.status, response.status());
//             assert_eq!(
//                 resp.headers
//                     .into_iter()
//                     .map(|(k, v)| format!("{}:{}", k.to_lowercase(), v))
//                     .collect::<Vec<_>>()
//                     .join(","),
//                 response
//                     .headers()
//                     .into_iter()
//                     .filter(|(k, _)| *k != "content-type"
//                         && *k != "set-cookie"
//                         && *k != "content-length")
//                     .map(|(k, v)| format!("{}:{}", k, v.to_str().unwrap()))
//                     .collect::<Vec<_>>()
//                     .join(","),
//             );
//             assert_eq!(
//                 {
//                     let mut c = resp
//                         .cookies
//                         .into_iter()
//                         .map(|c| format!("{}={}", c.name(), c.value()))
//                         .collect::<Vec<_>>();
//                     c.sort();
//                     c.join("; ")
//                 },
//                 {
//                     let mut c = response
//                         .headers()
//                         .get_all("set-cookie")
//                         .into_iter()
//                         .map(|v| v.to_str().unwrap().to_string())
//                         .collect::<Vec<_>>();
//                     c.sort();
//                     c.join("; ")
//                 },
//             );
//             assert_eq!(
//                 resp.body.unwrap_or(vec![]),
//                 to_bytes(response.into_body()).await.unwrap().to_vec()
//             );
//         })
//         .await;
//     }

//     #[tokio::test]
//     async fn test_axum_ws() {
//         run_ws_test_suite().await;
//     }
// }
