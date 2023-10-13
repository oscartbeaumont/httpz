use httpz::{
    http::{Method, Response, StatusCode},
    GenericEndpoint, Request,
};

#[tokio::main]
async fn main() {
    let endpoint = GenericEndpoint::new(
        "/*any", // TODO: Make this wildcard work
        [Method::GET, Method::POST],
        |_req: Request| async move {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(b"httpz running on Vercel!".to_vec())?)
        },
    );

    endpoint.vercel().await.unwrap()
}
