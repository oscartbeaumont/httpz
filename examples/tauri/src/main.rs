#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use httpz::{
    http::{Method, Response, StatusCode},
    GenericEndpoint, Request,
};

fn main() {
    let endpoint = GenericEndpoint::new("/", [Method::GET], |_req: Request| async move {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(b"Hello httpz World!".to_vec())?)
    });

    tauri::Builder::default()
        .register_uri_scheme_protocol("spacedrive", endpoint.tauri_uri_scheme("spacedrive"))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
