#[cfg(all(feature = "axum", feature = "ws"))]
#[tokio::main]
async fn main() {
    use futures::sink::SinkExt;
    use httpz::{
        http::Method,
        ws::{Message, WebsocketUpgrade},
        GenericEndpoint, Request,
    };

    let endpoint = GenericEndpoint::new(
        "/",
        [Method::GET, Method::POST],
        |req: Request| async move {
            WebsocketUpgrade::from_req(req, |_req, mut socket| async move {
                socket
                    .send(Message::Text("Hello World".to_string()))
                    .await
                    .unwrap();
            })
        },
    );

    #[cfg(feature = "cookies")]
    let endpoint2 = GenericEndpoint::new(
        "/",
        [Method::GET, Method::POST],
        |req: Request| async move {
            WebsocketUpgrade::from_req(req, |_req, mut socket| async move {
                socket
                    .send(Message::Text("Hello World".to_string()))
                    .await
                    .unwrap();
            })
        },
    );

    // Attach your endpoint to a HTTP server. This example uses Axum but it could be any other one.
    let app = axum::Router::new().nest("/", endpoint.axum());
    #[cfg(feature = "cookies")]
    let app = app.route("/cookiesws", endpoint2.axum());

    let addr = "[::]:9000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("Axum listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
