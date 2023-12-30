use std::sync::Arc;

use crate::{Endpoint, HttpEndpoint, HttpResponse, Server};

use tauri::{async_runtime::block_on, http::ResponseBuilder, AppHandle, Runtime};
use tokio::task::block_in_place;

impl<TEndpoint> Endpoint<TEndpoint>
where
    TEndpoint: HttpEndpoint,
{
    /// is called to mount the endpoint onto an Tauri Custom URI protocol
    pub fn tauri_uri_scheme<R: Runtime>(
        mut self,
        uri_scheme: impl Into<String>,
    ) -> impl Fn(
        &AppHandle<R>,
        &tauri::http::Request,
    ) -> Result<tauri::http::Response, Box<dyn std::error::Error>>
           + Send
           + Sync
           + 'static {
        let (_, methods) = self.endpoint.register();
        let endpoint = Arc::new(self.endpoint);
        let methods = methods.as_ref().to_vec();

        let uri_scheme = uri_scheme.into();
        let url1 = format!("{}://localhost/", uri_scheme);
        let url2 = format!("{}://", uri_scheme);
        let url3 = format!("http://{}.localhost/", uri_scheme);

        move |handle, req| {
            let resp = if !methods.contains(req.method()) {
                #[allow(clippy::unwrap_used)] // TODO: Error handling
                http::Response::builder().status(405).body(vec![]).unwrap()
            } else {
                let uri = req.uri();
                let uri = uri
                    .replace(&url1, &url3) // Windows
                    .replace(&url2, &url3); // Unix style

                // Encoded by `convertFileSrc` on the frontend
                let uri = percent_encoding::percent_decode(uri.as_bytes())
                    .decode_utf8_lossy()
                    .to_string();
                let mut r = http::Request::builder()
                    .method(req.method())
                    .uri(uri)
                    .extension(handle.clone());
                for (key, value) in req.headers() {
                    r = r.header(key, value);
                }
                #[allow(clippy::unwrap_used)] // TODO: Error handling
                let req = r.body(req.body().clone()).unwrap(); // TODO: Avoid clone once my upstream PR merges + error handling

                // TODO: This blocking sucks but is required for now. https://github.com/tauri-apps/wry/pull/872
                match block_in_place(|| {
                    block_on(endpoint.handler(crate::Request::new(req, Server::Tauri)))
                })
                .into_response()
                {
                    Ok(resp) => resp,
                    Err(_err) => {
                        // TODO: Do something with `_err`
                        #[allow(clippy::unwrap_used)] // TODO: Error handling
                        http::Response::builder().status(500).body(vec![]).unwrap()
                    }
                }
            };

            let mut r = ResponseBuilder::new()
                .version(resp.version())
                .status(resp.status());
            for (key, value) in resp.headers() {
                r = r.header(key, value);
            }
            r.body(resp.into_body())
        }
    }
}

impl crate::Request {
    /// TODO
    pub fn get_tauri_app_handle<R>(&self) -> Option<&AppHandle<R>>
    where
        R: Runtime,
    {
        self.0.extensions.get::<AppHandle<R>>()
    }
}
