use httpz::{
    http::{Method, StatusCode},
    GenericEndpoint, Request,
};
use lambda_http::{Error, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    let endpoint = GenericEndpoint::new(
        "/*any", // TODO: Make this wildcard work
        [Method::GET, Method::POST],
        |_req: Request| async move {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(b"httpz running on Netlify Functions!".to_vec())?)
        },
    );

    // TODO: URL Prefix
    endpoint.lambda().await
}
