// use std::sync::Arc;

// use axum::{
//     body::Bytes,
//     routing::{on, MethodFilter, MethodRouter},
// };
// use cookie::{Cookie, CookieJar};
// use http::{
//     header::{COOKIE, SET_COOKIE},
//     request::Parts,
//     HeaderMap, Request, StatusCode,
// };

// use crate::{Endpoint, HttpEndpoint};

// impl<TEndpoint> Endpoint<TEndpoint>
// where
//     TEndpoint: HttpEndpoint,
// {
//     /// is called to mount the endpoint onto an Axum router.
//     pub fn axum(mut self) -> MethodRouter {
//         let methods = self.endpoint.register();
//         let endpoint = Arc::new(self.endpoint);

//         let mut method_filter = MethodFilter::empty();
//         for method in methods.as_ref().iter() {
//             method_filter.insert(MethodFilter::try_from(method.clone()).unwrap());
//             // TODO: Error handling
//         }

//         on(method_filter, |parts: Parts, body: Bytes| async move {
//             let mut cookies = CookieJar::new();
//             for cookie in parts
//                 .headers
//                 .get_all(COOKIE)
//                 .into_iter()
//                 .filter_map(|value| value.to_str().ok())
//                 .flat_map(|value| value.split(';'))
//                 .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok())
//             {
//                 cookies.add_original(cookie);
//             }

//             match endpoint
//                 .handler(Request::from_parts(parts, body.to_vec()), &mut cookies)
//                 .await
//             {
//                 Ok(resp) => {
//                     let (mut parts, body) = resp.into_parts();
//                     for cookie in cookies.delta() {
//                         if let Ok(header_value) = cookie.encoded().to_string().parse() {
//                             parts.headers.append(SET_COOKIE, header_value);
//                         }
//                     }
//                     (parts.status, parts.headers, body)
//                 }
//                 Err(err) => (
//                     StatusCode::INTERNAL_SERVER_ERROR,
//                     HeaderMap::new(),
//                     err.to_string().as_bytes().to_vec(),
//                 ),
//             }
//         })
//     }
// }
