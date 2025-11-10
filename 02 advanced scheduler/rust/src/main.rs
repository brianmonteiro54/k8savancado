use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::{net::TcpListener, task};

#[derive(Clone, Default)]
struct AppState {
    allocations: Arc<Mutex<Vec<Vec<u8>>>>,
}

#[tokio::main]
async fn main() {
    let state = AppState::default();

    let app = Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
        .route("/info", get(info))
        .route("/busy", post(busy))
        .route("/alloc", post(alloc))
        .route("/free", post(free))
        .with_state(state);

    let port = env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.unwrap();

    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> impl IntoResponse {
    "aula02-scheduler-demo: up\n"
}

async fn healthz() -> impl IntoResponse {
    StatusCode::OK
}

async fn info() -> impl IntoResponse {
    let pod_name = env::var("POD_NAME").unwrap_or_else(|_| "unknown".into());
    let node_name = env::var("NODE_NAME").unwrap_or_else(|_| "unknown".into());
    let hostname = env::var("HOSTNAME").unwrap_or_else(|_| "unknown".into());

    Json(json!({
        "pod_name": pod_name,
        "node_name": node_name,
        "hostname": hostname
    }))
}

#[derive(Deserialize)]
struct BusyParams {
    ms: Option<u64>,
    threads: Option<usize>,
}

async fn busy(Query(params): Query<BusyParams>) -> impl IntoResponse {
    let ms = params.ms.unwrap_or(2000);
    let threads = params.threads.unwrap_or(1).max(1).min(64);

    let mut handles = Vec::with_capacity(threads);
    for _ in 0..threads {
        let handle = task::spawn_blocking(move || {
            let start = Instant::now();
            let mut x: u64 = 1;
            while start.elapsed() < Duration::from_millis(ms) {
                x = x.wrapping_mul(1664525).wrapping_add(1013904223);
                if x % 97 == 0 {
                    std::hint::black_box(x);
                }
            }
            x
        });
        handles.push(handle);
    }

    for h in handles {
        let _ = h.await;
    }
    Json(json!({"status": "ok", "elapsed_ms": ms, "threads": threads}))
}

#[derive(Deserialize)]
struct AllocParams {
    mb: Option<usize>,
    chunks: Option<usize>,
    touch: Option<bool>,
}

async fn alloc(
    State(state): State<AppState>,
    Query(params): Query<AllocParams>,
) -> impl IntoResponse {
    let mb = params.mb.unwrap_or(64).max(1);
    let chunks = params.chunks.unwrap_or(1).max(1);
    let do_touch = params.touch.unwrap_or(true);

    let mut guard = state.allocations.lock().unwrap();
    for _ in 0..chunks {
        let mut v = vec![0u8; mb * 1024 * 1024];
        if do_touch {
            for i in (0..v.len()).step_by(4096) {
                v[i] = 1;
            }
        }
        guard.push(v);
    }
    let total_mb: usize = guard.iter().map(|v| v.len() / (1024 * 1024)).sum();
    Json(json!({
        "status": "ok",
        "allocated_now_mb": mb * chunks,
        "total_allocated_mb": total_mb
    }))
}

async fn free(State(state): State<AppState>) -> impl IntoResponse {
    let mut guard = state.allocations.lock().unwrap();
    let freed_mb: usize = guard.iter().map(|v| v.len() / (1024 * 1024)).sum();
    guard.clear();
    Json(json!({
        "status": "ok",
        "freed_mb": freed_mb
    }))
}
