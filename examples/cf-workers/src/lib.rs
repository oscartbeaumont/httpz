use httpz::{
    cookie::CookieJar,
    http::{Method, Response, StatusCode},
    ConcreteRequest, EndpointResult, GenericEndpoint,
};
use worker::{console_log, event, Date};

mod utils;

async fn handler<'a>(_ctx: (), _req: ConcreteRequest, cookies: CookieJar) -> EndpointResult {
    Ok((
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(b"Hello httpz World!".to_vec())?,
        cookies,
    ))
}

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

    let endpoint = GenericEndpoint::new((), [Method::GET, Method::POST], handler);

    endpoint.workers(req).await
}
