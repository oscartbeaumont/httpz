use actix_web::{web, App, HttpServer};
use httpz::{
    http::{Method, Response, StatusCode},
    GenericEndpoint, Request,
};

#[cfg(feature = "actix-web")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let endpoint = GenericEndpoint::new(
        "/{any:.*}", // TODO: This is inconsistent across webservers
        [Method::GET, Method::POST],
        |req: Request| async move {
            println!("{:?}", req.uri().path()); // TODO: Strip the `/prefix` bit for consistency with other webservers
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(b"Hello httpz World!".to_vec())?)
        },
    );

    let addr = "[::]:9001".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("actix-web listening on http://{}", addr);
    HttpServer::new({
        let endpoint = endpoint.actix();
        move || App::new().service(web::scope("/prefix").service(endpoint.mount()))
    })
    .bind(addr)?
    .run()
    .await
}
