/// support for [Actix Web](https://actix.rs)
#[cfg(feature = "actix-web")]
pub mod actix_web;

/// support for [Axum](https://github.com/tokio-rs/axum)
#[cfg(feature = "axum")]
pub mod axum;

/// support for [Poem](https://github.com/poem-web/poem)
#[cfg(feature = "poem")]
pub mod poem;

/// support for [Rocket](https://rocket.rs)
#[cfg(feature = "rocket")]
pub mod rocket;

/// support for [Warp](https://github.com/seanmonstar/warp)
#[cfg(feature = "warp")]
pub mod warp;

/// support for [AWS Lambda](https://github.com/awslabs/aws-lambda-rust-runtime) & [Netlify functions](https://docs.netlify.com/functions/overview)
#[cfg(feature = "lambda")]
pub mod lambda;

/// support for [Cloudflare Workers](https://developers.cloudflare.com/workers/)
#[cfg(feature = "workers")]
pub mod workers;

/// support for [Tauri](https://tauri.app)
#[cfg(feature = "tauri")]
pub mod tauri;

/// support for [Hyper](https://github.com/hyperium/hyper)
#[cfg(feature = "hyper")]
pub mod hyper;

/// support for [Vercel](https://github.com/vercel-community/rust)
#[cfg(feature = "vercel")]
pub mod vercel;
