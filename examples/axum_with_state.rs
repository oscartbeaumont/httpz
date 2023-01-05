use axum::{extract::State, response::IntoResponse, Extension, RequestPartsExt};
use httpz::{
    http::{Method, Response, StatusCode},
    GenericEndpoint, Request,
};

#[derive(Debug, Clone)]
pub struct MyCtx;

#[cfg(feature = "axum")]
#[tokio::main]
async fn main() {
    let endpoint = GenericEndpoint::new(
        "/*any",
        [Method::GET, Method::POST],
        |req: Request| async move {
            // If the generic here doesn't match your Axum router it will return `None`. This isn't super typesafe but it's what you get for having to support 10 different web frameworks.
            let axum_state = req.get_axum_state::<MyCtx>().unwrap();
            let (mut parts, body) = req.into_parts();

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(b"Hello httpz World!".to_vec())?)
        },
    );

    // Attach your endpoint to a HTTP server. This example uses Axum but it could be any other one.
    let app = axum::Router::<MyCtx>::new()
        .nest("/", endpoint.axum())
        .with_state(MyCtx {});

    let addr = "[::]:9000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("Axum listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
