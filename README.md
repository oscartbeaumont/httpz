<div align="center">
    <h1>httpz</h1>
    <p><b>Code once, support every Rust webserver!</b></p>
    <a href="https://discord.gg/4V9M5sksw8"><img src="https://img.shields.io/discord/1011665225809924136?style=flat-square" alt="Discord"></a>
    <a href="https://crates.io/crates/httpz"><img src="https://img.shields.io/crates/d/httpz?style=flat-square" alt="Crates.io"></a>
    <a href="/LICENSE.md"><img src="https://img.shields.io/crates/l/httpz?style=flat-square" alt="License"></a>
</div>

<br>

This project is a 🚧 work in progress 🚧. Currently it is designed around the goals of [rspc](https://rspc.otbeaumont.me) but feel free to reach to me if you want to collaborate on using it in your own project.

## Usage

```rust
// Define your a single HTTP handler which is supported by all major Rust webservers.
async fn handler<'a>(
    _ctx: (),
    _req: ConcreteRequest,
    _cookies: &'a mut CookieJar,
) -> EndpointResult {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(b"Hello httpz World!".to_vec())?)
}

// Define a httpz generic endpoint. handler MUST be a function, closures will not work.
let endpoint = GenericEndpoint::new((), [Method::GET, Method::POST], handler); 

// Attach your endpoint to any HTTP server you like (Axum shown here).
let app = axum::Router::new().route("/", endpoint.axum());
```

Check out the rest of the [examples](/examples)!
## Features

 - Write your HTTP handler once and support [Axum](https://github.com/tokio-rs/axum), [Actix Web](https://actix.rs/), [Poem](https://github.com/poem-web/poem), [Rocket](https://rocket.rs), [Warp](https://github.com/seanmonstar/warp) and more.
 - Support for websockets on compatible webservers.

## Projects using httpz

httpz is primarily designed to make life easier for library authors. It allows a library author to write and test a HTTP endpoint once and know it will work for all major Rust HTTP servers.

Libraries using httpz:

 - [rspc](https://github.com/oscartbeaumont/rspc)

If you are interested in using httpz and have questions jump in [the Discord](https://discord.gg/4V9M5sksw8)!