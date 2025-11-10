use axum::{routing::get, Router, extract::State, response::IntoResponse};
use std::{env, net::SocketAddr, sync::Arc};
use prometheus::{Encoder, TextEncoder, Registry, Counter, Opts};

#[derive(Clone)]
struct AppState {
    version: String,
    requests_total: Counter,
    errors_total: Counter,
    registry: Registry,
}

#[tokio::main]
async fn main() {
    // VERSION env or default "v1"
    let app_version = env::var("APP_VERSION").unwrap_or_else(|_| "v1".into());

    // Prometheus metrics
    let registry = Registry::new();
    let requests_total = Counter::with_opts(Opts::new("requests_total", "Total HTTP requests")).unwrap();
    let errors_total = Counter::with_opts(Opts::new("errors_total", "Total simulated errors")).unwrap();
    registry.register(Box::new(requests_total.clone())).ok();
    registry.register(Box::new(errors_total.clone())).ok();

    let state = AppState { version: app_version, requests_total, errors_total, registry };

    let app = Router::new()
        .route("/", get(root))
        .route("/version", get(version_handler))
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .with_state(Arc::new(state));

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    println!("versioned-echo listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await.unwrap();
}

async fn root(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    state.requests_total.inc();
    format!("Hello from version {}!\n", state.version)
}

async fn version_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    state.requests_total.inc();
    serde_json::json!({ "version": state.version.clone() }).to_string()
}

async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    state.requests_total.inc();
    "ok"
}

async fn metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    encoder.encode(&state.registry.gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
