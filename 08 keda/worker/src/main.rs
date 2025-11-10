use anyhow::Result;
use axum::{routing::get, Router};
use amqprs::{
    channel::{BasicAckArguments, BasicConsumeArguments, BasicQosArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::task;
use tokio::time::sleep;
use tracing::{error, info};

async fn run_health_server() -> Result<()> {
    let app = Router::new().route("/healthz", get(|| async { "ok" }));
    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

struct Worker {
    queue: String,
}

#[async_trait::async_trait]
impl AsyncConsumer for Worker {
    async fn consume(
        &mut self,
        channel: &amqprs::channel::Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let payload = String::from_utf8_lossy(&content);
        info!("Processando mensagem: {}", payload);
        
        // Simula processamento (tune via PROCESS_MS)
        let ms = std::env::var("PROCESS_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(100);
        sleep(Duration::from_millis(ms)).await;
        
        let args = BasicAckArguments::new(deliver.delivery_tag(), false);
        if let Err(e) = channel.basic_ack(args).await {
            error!("Falha ao confirmar mensagem: {}", e);
        }
    }
}

async fn consume_loop(amqp_addr: String, queue: String) -> Result<()> {
    loop {
        info!("Conectando a {}", amqp_addr);
        
        // Parse AMQP URL
        let url = url::Url::parse(&amqp_addr)?;
        let host = url.host_str().unwrap_or("localhost");
        let port = url.port().unwrap_or(5672);
        let username = if url.username().is_empty() {
            "guest"
        } else {
            url.username()
        };
        let password = url.password().unwrap_or("guest");
        let vhost = url.path().trim_start_matches('/');
        let vhost = if vhost.is_empty() { "/" } else { vhost };
        
        let args = OpenConnectionArguments::new(host, port, username, password)
            .virtual_host(vhost)
            .finish();
        
        match Connection::open(&args).await {
            Ok(conn) => {
                let channel = conn.open_channel(None).await?;
                
                // Declare queue
                let queue_args = QueueDeclareArguments::new(&queue)
                    .durable(true)
                    .finish();
                channel.queue_declare(queue_args).await?;
                
                // Set QoS (prefetch)
                let prefetch = std::env::var("PREFETCH")
                    .ok()
                    .and_then(|v| v.parse::<u16>().ok())
                    .unwrap_or(10);
                let qos_args = BasicQosArguments::new(0, prefetch, false);
                channel.basic_qos(qos_args).await?;
                
                // Start consuming
                let consume_args = BasicConsumeArguments::new(&queue, "worker");
                let worker = Worker {
                    queue: queue.clone(),
                };
                
                info!("Consumindo fila '{}', prefetch={}", queue, prefetch);
                channel.basic_consume(worker, consume_args).await?;
                
                // Keep connection alive
                loop {
                    if conn.is_open() {
                        sleep(Duration::from_secs(1)).await;
                    } else {
                        error!("Conexão fechada");
                        break;
                    }
                }
            }
            Err(e) => {
                error!("Falha ao conectar: {e} — nova tentativa em 3s");
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    let amqp_addr = std::env::var("AMQP_ADDR").expect("AMQP_ADDR não definido");
    let queue = std::env::var("QUEUE").unwrap_or_else(|_| "orders".into());

    // Health server em background
    task::spawn(async move {
        if let Err(e) = run_health_server().await {
            eprintln!("health server error: {e}");
        }
    });

    consume_loop(amqp_addr, queue).await?;
    Ok(())
}
