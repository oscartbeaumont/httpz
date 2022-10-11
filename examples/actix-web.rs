use httpz::{
    http::{Method, Response, StatusCode},
    GenericEndpoint, Request,
};

#[cfg(feature = "actix-web")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _endpoint = GenericEndpoint::new([Method::GET, Method::POST], |_req: Request| async move {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(b"Hello httpz World!".to_vec())?)
    });

    let addr = "[::]:9001".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("actix-web listening on http://{}", addr);
    // HttpServer::new({
    //     let endpoint = endpoint.actix();
    //     move || App::new().route("/", endpoint.mount())
    // })
    // .bind(addr)?
    // .run()
    // .await

    todo!();
}
