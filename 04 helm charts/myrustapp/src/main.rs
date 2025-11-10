use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize)]
struct RootResponse {
    app: &'static str,
    version: String,
    greeting: String,
    pod: String,
    note: &'static str,
}

async fn root() -> Json<RootResponse> {
    let greeting = env::var("GREETING").unwrap_or_else(|_| "Olá, Helm + Rust!".to_string());
    let version = env::var("APP_VERSION").unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());
    let pod = env::var("POD_NAME").unwrap_or_else(|_| "local-dev".to_string());

    Json(RootResponse {
        app: "myrustapp",
        version,
        greeting,
        pod,
        note: "Deployado via Helm Chart",
    })
}

async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    // Logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health));

    let port: u16 = env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
