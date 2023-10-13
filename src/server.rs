/// Server represents the server that the request is coming from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Server {
    /// support for [Actix Web](https://actix.rs)
    #[cfg(feature = "actix-web")]
    ActixWeb,
    /// support for [Axum](https://github.com/tokio-rs/axum)
    #[cfg(feature = "axum")]
    Axum,
    /// support for [Poem](https://github.com/poem-web/poem)
    #[cfg(feature = "poem")]
    Poem,
    /// support for [Rocket](https://rocket.rs)
    #[cfg(feature = "rocket")]
    Rocket,
    /// support for [Warp](https://github.com/seanmonstar/warp)
    #[cfg(feature = "warp")]
    Warp,
    /// support for [AWS Lambda](https://github.com/awslabs/aws-lambda-rust-runtime) & [Netlify functions](https://docs.netlify.com/functions/overview)
    #[cfg(feature = "lambda")]
    Lambda,
    /// support for [Cloudflare Workers](https://developers.cloudflare.com/workers/)
    #[cfg(feature = "workers")]
    CloudflareWorkers,
    /// support for [Cloudflare Workers](https://developers.cloudflare.com/workers/)
    #[cfg(feature = "tauri")]
    Tauri,
    /// support for [Hyper](https://github.com/hyperium/hyper)
    #[cfg(feature = "hyper")]
    Hyper,
    /// support for [Vercel](https://github.com/vercel-community/rust)
    #[cfg(feature = "vercel")]
    Vercel,
}

impl Server {
    /// convert the server into a string
    #[allow(unreachable_patterns)]
    pub fn to_str(&self) -> &'static str {
        match self {
            #[cfg(feature = "actix-web")]
            Self::ActixWeb => "actix-web",
            #[cfg(feature = "axum")]
            Self::Axum => "axum",
            #[cfg(feature = "poem")]
            Self::Poem => "poem",
            #[cfg(feature = "rocket")]
            Self::Rocket => "rocket",
            #[cfg(feature = "warp")]
            Self::Warp => "warp",
            #[cfg(feature = "lambda")]
            Self::Lambda => "lambda",
            #[cfg(feature = "workers")]
            Self::CloudflareWorkers => "workers",
            #[cfg(feature = "tauri")]
            Self::Tauri => "tauri",
            #[cfg(feature = "hyper")]
            Self::Hyper => "hyper",
            #[cfg(feature = "vercel")]
            Self::Vercel => "vercel",
            _ => unreachable!(),
        }
    }

    /// check if the server that handled this request supports upgrading the connection to a websocket.
    #[allow(unreachable_patterns)]
    pub fn supports_websockets(&self) -> bool {
        match self {
            #[cfg(feature = "actix-web")]
            Self::ActixWeb => false,
            #[cfg(feature = "axum")]
            Self::Axum => true,
            #[cfg(feature = "poem")]
            Self::Poem => false,
            #[cfg(feature = "rocket")]
            Self::Rocket => false,
            #[cfg(feature = "warp")]
            Self::Warp => false,
            #[cfg(feature = "lambda")]
            Self::Lambda => false,
            #[cfg(feature = "workers")]
            Self::CloudflareWorkers => false,
            #[cfg(feature = "tauri")]
            Self::Tauri => false,
            #[cfg(feature = "hyper")]
            Self::Hyper => false,
            #[cfg(feature = "vercel")]
            Self::Vercel => false,
            _ => unreachable!(),
        }
    }
}
