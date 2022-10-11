use httpz::{
    http::{Method, Response, StatusCode},
    GenericEndpoint, Request,
};

#[cfg(all(feature = "axum", feature = "cookies"))]
#[tokio::main]
async fn main() {
    use cookie::Cookie;

    let endpoint =
        GenericEndpoint::new([Method::GET, Method::POST], |mut req: Request| async move {
            // DO THIS
            let mut cookies = req.cookies(); // This creates a new CookieJar from the request.
            cookies.add(Cookie::new("foo", "bar"));

            // DON'T DO THIS
            // It will create a new CookieJar, set a cookie on it and then drop it. The cookies won't be set on the response because it's not returned from handler.
            // req.cookies().add(...);

            Ok((
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(b"Hello httpz World!".to_vec())?,
                cookies, // If you don't return the CookieJar the cookies won't be set.
            ))
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
