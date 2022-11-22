use httpz::{
    http::{Method, Response, StatusCode},
    GenericEndpoint, Request,
};
use worker::{console_log, event, Date};

mod utils;

#[event(fetch, respond_with_errors)]
pub async fn main(
    req: worker::Request,
    _env: worker::Env,
    _ctx: worker::Context,
) -> worker::Result<worker::Response> {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );

    utils::set_panic_hook();

    let endpoint = GenericEndpoint::new(
        "/*any", // TODO: Make this wildcard work
        [Method::GET, Method::POST],
        |_req: Request| async move {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(b"httpz running on Cloudflare Workers!".to_vec())?)
        },
    );

    // TODO: Compatibility with the built in HTTP router
    // TODO: URL Prefix
    endpoint.workers(req).await
}
