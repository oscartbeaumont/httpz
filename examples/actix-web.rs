use actix_web::{web, App, HttpResponse, HttpServer};
use httpz::{
    cookie::CookieJar,
    http::{Method, Response, StatusCode},
    ConcreteRequest, EndpointResult, GenericEndpoint,
};

async fn handler<'a>(_ctx: (), _req: ConcreteRequest, cookies: CookieJar) -> EndpointResult {
    Ok((
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(b"Hello httpz World!".to_vec())?,
        cookies,
    ))
}

#[cfg(feature = "actix-web")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let endpoint = GenericEndpoint::new((), [Method::GET, Method::POST], handler);

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
