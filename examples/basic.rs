use httpz::{
    http::{Method, Response, StatusCode},
    GenericEndpoint, Request,
};

#[cfg(feature = "axum")]
#[tokio::main]
async fn main() {
    let endpoint = GenericEndpoint::new([Method::GET, Method::POST], |_req: Request| async move {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(b"Hello httpz World!".to_vec())?)
    });

    // Attach your endpoint to a HTTP server. This example uses Axum but it could be any other one.
    let app = axum::Router::new().route("/", endpoint.axum());

    let addr = "[::]:9000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("Axum listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
