
use axum::{routing::get, Json, Router};
use std::env;
use serde::Serialize;
use std::net::SocketAddr;

#[derive(Serialize)]
struct Version {
    version: String,
    hostname: String,
}

async fn root() -> &'static str { "ok" }
async fn healthz() -> &'static str { "ok" }
async fn readyz() -> &'static str { "ready" }

async fn version() -> Json<Version> {
    let version = env::var("APP_VERSION").unwrap_or_else(|_| "1.0.0".to_string());
    let hostname = hostname::get().unwrap_or_default().to_string_lossy().into_owned();
    Json(Version { version, hostname })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/version", get(version));

    let port: u16 = env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await.unwrap();
}
