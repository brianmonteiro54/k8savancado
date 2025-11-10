use anyhow::Result;
use clap::Parser;
use amqprs::{
    channel::{BasicPublishArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
    BasicProperties,
};
use std::time::Duration;
use tracing::info;
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[command(name="publisher")]
#[command(about="Publica mensagens na fila RabbitMQ para gerar backlog")]
struct Args {
    /// AMQP connection string (ex: amqp://user:pass@rabbitmq.keda-demo.svc.cluster.local:5672/%2f)
    #[arg(long, env = "AMQP_ADDR")]
    amqp_addr: String,

    /// Nome da fila
    #[arg(long, default_value = "orders")]
    queue: String,

    /// Quantidade total de mensagens a publicar
    #[arg(long, default_value_t = 200)]
    count: u32,

    /// Mensagens simultâneas por burst (não precisa ser alto)
    #[arg(long, default_value_t = 20)]
    concurrency: u16,

    /// Atraso (ms) entre bursts (ajuda a visualizar a dinâmica)
    #[arg(long, default_value_t = 50)]
    burst_delay_ms: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    let args = Args::parse();
    info!("Conectando a {}", args.amqp_addr);
    
    // Parse AMQP URL
    let url = url::Url::parse(&args.amqp_addr)?;
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
    
    let conn_args = OpenConnectionArguments::new(host, port, username, password)
        .virtual_host(vhost)
        .finish();
    
    let conn = Connection::open(&conn_args).await?;
    let channel = conn.open_channel(None).await?;

    // Declare queue
    let queue_args = QueueDeclareArguments::new(&args.queue)
        .durable(true)
        .finish();
    channel.queue_declare(queue_args).await?;
    info!("Fila '{}' declarada", &args.queue);

    let mut published = 0u32;
    while published < args.count {
        let burst = (args.count - published).min(args.concurrency as u32);
        for i in 0..burst {
            let body = format!("pedido-{}", published + i + 1).into_bytes();
            let publish_args = BasicPublishArguments::new("", &args.queue);
            let props = BasicProperties::default().with_delivery_mode(2).finish();
            
            channel.basic_publish(props, body, publish_args).await?;
        }
        published += burst;
        info!("Publicado burst; total = {}", published);
        sleep(Duration::from_millis(args.burst_delay_ms)).await;
    }

    info!("Concluído. Publicadas {} mensagens.", published);
    Ok(())
}
