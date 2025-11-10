
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use prometheus::{Encoder, TextEncoder, HistogramVec, IntCounterVec, IntGauge, IntGaugeVec, register_histogram_vec, register_int_counter_vec, register_int_gauge, register_int_gauge_vec};
use std::{future::Future, net::SocketAddr, pin::Pin, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Instant};
use tokio::{net::TcpListener, signal};
use tracing::{info, warn};
use tower_http::trace::TraceLayer;
use tower::{ServiceBuilder, Layer};
use lazy_static::lazy_static;
use serde::Deserialize;

#[allow(unused_imports)]
use axum::response::Response;


// ---------- Metrics ----------
lazy_static! {
    static ref REQS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "http_requests_total",
        "Number of HTTP requests",
        &["method", "path", "status"]
    ).unwrap();

    static ref REQ_INFLIGHT: IntGauge = register_int_gauge!(
        "http_requests_in_flight",
        "Number of in-flight HTTP requests"
    ).unwrap();

    static ref REQ_DURATION: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request latencies in seconds",
        &["method", "path", "status"]
    ).unwrap();

    static ref ERR_TOTAL: IntCounterVec = register_int_counter_vec!(
        "http_errors_total",
        "Number of HTTP 5xx errors",
        &["path", "status"]
    ).unwrap();

    static ref SATURATION_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "saturation_gauge",
        "A toy gauge for saturation (e.g., number of concurrent work units)",
        &["type"]
    ).unwrap();
}


// ---------- App State ----------
#[derive(Clone)]
struct AppState {
    ready: Arc<AtomicBool>,
    live: Arc<AtomicBool>,
    started: Arc<AtomicBool>,
    // leaked memory store for demo
    mem_leak: Arc<parking_lot::RwLock<Vec<Vec<u8>>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            ready: Arc::new(AtomicBool::new(false)),
            live: Arc::new(AtomicBool::new(true)),
            started: Arc::new(AtomicBool::new(false)),
            mem_leak: Arc::new(parking_lot::RwLock::new(Vec::new())),
        }
    }
}


// ---------- Query structs ----------
#[derive(Deserialize)]
struct CpuParams { seconds: Option<u64> }

#[derive(Deserialize)]
struct MemParams { mb: Option<usize>, leak: Option<bool> }

#[derive(Deserialize)]
struct LatencyParams { ms: Option<u64> }

#[derive(Deserialize)]
struct ErrorParams { rate: Option<f64> }

#[derive(Deserialize)]
struct FlipParams { value: Option<bool> }


// ---------- Handlers ----------
async fn metrics() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    (StatusCode::OK, String::from_utf8(buffer).unwrap())
}

async fn live(State(state): State<AppState>) -> impl IntoResponse {
    if state.live.load(Ordering::SeqCst) { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE }
}

async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    let ok = state.started.load(Ordering::SeqCst) && state.ready.load(Ordering::SeqCst);
    if ok { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE }
}

async fn startup(State(state): State<AppState>) -> impl IntoResponse {
    if state.started.load(Ordering::SeqCst) { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE }
}

async fn info() -> impl IntoResponse {
    let body = serde_json::json!({
        "name": "app-health-demo",
        "version": env!("CARGO_PKG_VERSION"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
    });
    (StatusCode::OK, axum::Json(body))
}

async fn simulate_cpu(Query(params): Query<CpuParams>) -> impl IntoResponse {
    let seconds = params.seconds.unwrap_or(5);
    let until = Instant::now() + std::time::Duration::from_secs(seconds);
    // Busy loop to burn CPU
    let mut n: u64 = 0;
    while Instant::now() < until {
        // simple math to keep CPU busy
        n = n.wrapping_mul(1664525).wrapping_add(1013904223);
        if n % 1_000_003 == 0 {
            tokio::task::yield_now().await;
        }
    }
    format!("CPU busy loop completed in ~{}s", seconds)
}

async fn simulate_mem(State(state): State<AppState>, Query(params): Query<MemParams>) -> impl IntoResponse {
    let mb = params.mb.unwrap_or(64);
    let leak = params.leak.unwrap_or(true);
    let bytes = mb * 1024 * 1024;
    let block: Vec<u8> = vec![0u8; bytes];
    if leak {
        state.mem_leak.write().push(block);
        format!("Leaked ~{} MiB (total blocks: {})", mb, state.mem_leak.read().len())
    } else {
        format!("Allocated ~{} MiB and dropped", mb)
    }
}

async fn simulate_latency(Query(params): Query<LatencyParams>) -> impl IntoResponse {
    let ms = params.ms.unwrap_or(500);
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    format!("Slept {} ms", ms)
}

async fn simulate_error(Query(params): Query<ErrorParams>) -> impl IntoResponse {
    let rate = params.rate.unwrap_or(0.5).clamp(0.0, 1.0);
    let rnd: f64 = rand::random();
    if rnd < rate {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Injected error").into_response();
    }
    "OK".into_response()
}

async fn unstable() -> impl IntoResponse {
    use rand::Rng;
    let choice: u8 = rand::thread_rng().gen_range(0..100);
    if choice < 10 {
        (StatusCode::INTERNAL_SERVER_ERROR, "flaky 500").into_response()
    } else if choice < 40 {
        let delay = rand::thread_rng().gen_range(300..1500);
        SATURATION_GAUGE.with_label_values(&["inflight"]).inc();
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        SATURATION_GAUGE.with_label_values(&["inflight"]).dec();
        format!("slow path {} ms", delay).into_response()
    } else {
        "fast path".into_response()
    }
}

// Admin flips to demonstrate probes
async fn flip_ready(State(state): State<AppState>, Query(p): Query<FlipParams>) -> impl IntoResponse {
    let v = p.value.unwrap_or(true);
    state.ready.store(v, Ordering::SeqCst);
    format!("readiness = {v}")
}
async fn flip_live(State(state): State<AppState>, Query(p): Query<FlipParams>) -> impl IntoResponse {
    let v = p.value.unwrap_or(true);
    state.live.store(v, Ordering::SeqCst);
    format!("liveness = {v}")
}

// ---------- Middleware for metrics ----------
#[derive(Clone)]
struct MetricsLayer;

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;
    fn layer(&self, inner: S) -> Self::Service { MetricsService { inner } }
}

#[derive(Clone)]
struct MetricsService<S> {
    inner: S,
}

impl<S, ReqBody> tower::Service<axum::http::Request<ReqBody>> for MetricsService<S>
where
    S: tower::Service<axum::http::Request<ReqBody>, Response = axum::response::Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        tower::Service::<axum::http::Request<ReqBody>>::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        let mut svc = self.inner.clone();
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        REQ_INFLIGHT.inc();
        let start = Instant::now();
        Box::pin(async move {
            let res = svc.call(req).await;
            let status = match &res {
                Ok(r) => r.status(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };
            let status_str = status.as_u16().to_string();
            REQS_TOTAL.with_label_values(&[method.as_str(), &path, &status_str]).inc();
            REQ_DURATION.with_label_values(&[method.as_str(), &path, &status_str]).observe(start.elapsed().as_secs_f64());
            if status.is_server_error() {
                ERR_TOTAL.with_label_values(&[&path, &status_str]).inc();
            }
            REQ_INFLIGHT.dec();
            res
        })
    }
}

// ---------- Main ----------
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Tracing init
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let state = AppState::default();

    // Simulate slow startup gate by env var
    let startup_delay_ms: u64 = std::env::var("STARTUP_DELAY_MS").ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);
    let ready_after_ms: u64 = std::env::var("READY_AFTER_MS").ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5000);

    let state_clone = state.clone();
    tokio::spawn(async move {
        info!("Startup: sleeping {} ms before startupProbe passes", startup_delay_ms);
        tokio::time::sleep(std::time::Duration::from_millis(startup_delay_ms)).await;
        state_clone.started.store(true, Ordering::SeqCst);
        info!("Startup: started=true");
        tokio::time::sleep(std::time::Duration::from_millis(ready_after_ms)).await;
        state_clone.ready.store(true, Ordering::SeqCst);
        info!("Readiness: ready=true");
    });

    let app = Router::new()
        .route("/metrics", get(metrics))
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .route("/health/startup", get(startup))
        .route("/info", get(info))
        .route("/simulate/cpu", get(simulate_cpu))
        .route("/simulate/mem", get(simulate_mem))
        .route("/simulate/latency", get(simulate_latency))
        .route("/simulate/error", get(simulate_error))
    .route("/unstable", get(unstable))
        .route("/admin/flip_ready", get(flip_ready))
        .route("/admin/flip_live", get(flip_live))
        .with_state(state.clone())
        .layer(ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(MetricsLayer)
        );

    let port: u16 = std::env::var("PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    info!("Listening on {}", local_addr);

    let server = axum::serve(listener, app.into_make_service());

    // Graceful shutdown (SIGTERM)
    let shutdown_state = state.clone();
    let shutdown = async move {
        signal::ctrl_c().await.expect("failed to listen for event");
        warn!("Received shutdown signal; setting ready=false");
        shutdown_state.ready.store(false, Ordering::SeqCst);
        // give time to drain
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    };

    server.with_graceful_shutdown(shutdown).await?;
    Ok(())
}
