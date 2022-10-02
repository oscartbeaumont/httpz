use httpz::{
    cookie::CookieJar,
    http::{Method, StatusCode},
    ConcreteRequest, EndpointResult, GenericEndpoint,
};
use lambda_http::{run, Error, Response};

async fn handler<'a>(_ctx: (), _req: ConcreteRequest, cookies: CookieJar) -> EndpointResult {
    Ok((
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(b"httpz running on Netlify Functions!".to_vec())?,
        cookies, // You must pass the CookieJar back so the cookies are set of the response.
    ))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    let endpoint = GenericEndpoint::new((), [Method::GET, Method::POST], handler);
    run(endpoint.lambda()).await
}
