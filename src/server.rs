use axum::{
    http::{header, HeaderValue, Method},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

use crate::api;

/// Explicit allowlist of origins permitted to make cross-origin requests.
/// No wildcard ("*") is used so credentials and browser requests stay safe.
const ALLOWED_ORIGINS: &[&str] = &[
    "https://app.example.com",
    "https://admin.example.com",
    "http://localhost:3000",
    "http://localhost:5173",
];

/// Build the CORS layer with an explicit origin allowlist, methods, and headers.
fn cors_layer() -> CorsLayer {
    let origins: Vec<HeaderValue> = ALLOWED_ORIGINS
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::ORIGIN,
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600))
}

/// Build the application router with CORS applied to the API surface.
pub fn build_router() -> Router {
    Router::new()
        .nest("/api", api::router())
        .layer(cors_layer())
}

/// Start the HTTP server, applying the CORS layer to all API routes.
pub async fn run(addr: SocketAddr) -> std::io::Result<()> {
    let app = build_router();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await
}
