use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use k8s_openapi::{
    api::apps::v1::{Deployment, DeploymentSpec},
    api::core::v1::{Container, Namespace, PodSpec, PodTemplateSpec, Service, ServicePort, ServiceSpec},
    apimachinery::pkg::apis::meta::v1::ObjectMeta,
};
use kube::{api::{Api, DeleteParams, Patch, PatchParams, PostParams}, Client};
use serde_json::json;
use std::collections::BTreeMap;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "orchestrator")]
#[command(about = "Blue/Green Orchestrator (kube-rs)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Cria namespace, deployments blue/green e service apontando pra blue
    Init,
    /// Remove recursos criados (namespace inteira)
    Cleanup,
    /// Mostra para qual env o Service está apontando
    Status,
    /// Faz cutover do Service para a cor informada
    Switch { #[arg(long, value_enum)] to: Color },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Color { Blue, Green }

const NS: &str = "aula05";
const APP: &str = "myapp";
const IMAGE: &str = "myapp:latest";
const PORT: i32 = 8080;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let client = Client::try_default().await?;

    match cli.command {
        Commands::Init => init(&client).await?,
        Commands::Cleanup => cleanup(&client).await?,
        Commands::Status => status(&client).await?,
        Commands::Switch { to } => switch_cmd(&client, to).await?,
    }
    Ok(())
}

async fn init(client: &Client) -> Result<()> {
    // Namespace
    let ns_api: Api<Namespace> = Api::all(client.clone());
    if ns_api.get_opt(NS).await?.is_none() {
        ns_api.create(&PostParams::default(), &Namespace {
            metadata: ObjectMeta { name: Some(NS.into()), ..Default::default() },
            ..Default::default()
        }).await?;
        tracing::info!("Namespace '{}' criado.", NS);
    } else {
        tracing::info!("Namespace '{}' já existe (ok).", NS);
    }

    // Deployments Blue & Green
    let dep_api: Api<Deployment> = Api::namespaced(client.clone(), NS);
    let _ = dep_api.create(&PostParams::default(), &deployment("blue", "1.0")).await;
    let _ = dep_api.create(&PostParams::default(), &deployment("green", "2.0")).await;
    tracing::info!("Deployments blue/green aplicados (idempotente).");

    // Service apontando para blue
    let svc_api: Api<Service> = Api::namespaced(client.clone(), NS);
    if svc_api.get_opt(APP).await?.is_none() {
        svc_api.create(&PostParams::default(), &service("blue")).await?;
        tracing::info!("Service criado apontando para BLUE.");
    } else {
        tracing::info!("Service já existe (ok).");
    }
    Ok(())
}

async fn cleanup(client: &Client) -> Result<()> {
    let ns_api: Api<Namespace> = Api::all(client.clone());
    match ns_api.delete(NS, &DeleteParams::default()).await {
        Ok(_) => tracing::info!("Namespace '{}' removido.", NS),
        Err(e) => tracing::warn!("Falha ao remover namespace (talvez não exista): {e}"),
    }
    Ok(())
}

async fn status(client: &Client) -> Result<()> {
    let svc_api: Api<Service> = Api::namespaced(client.clone(), NS);
    if let Some(svc) = svc_api.get_opt(APP).await? {
        if let Some(spec) = svc.spec {
            if let Some(sel) = spec.selector {
                let color = sel.get("env").cloned().unwrap_or_else(|| "unknown".into());
                println!("Service '{}' está apontando para env = {}", APP, color);
                return Ok(());
            }
        }
    }
    println!("Service '{}' não encontrado.", APP);
    Ok(())
}

async fn switch_cmd(client: &Client, to: Color) -> Result<()> {
    let svc_api: Api<Service> = Api::namespaced(client.clone(), NS);
    let color = match to { Color::Blue => "blue", Color::Green => "green" };
    let patch = json!({
        "spec": { "selector": { "app": APP, "env": color } }
    });
    svc_api.patch(APP, &Patch::Apply("orchestrator").force(), &Patch::Merge(&patch)).await?;
    tracing::info!("Service apontado para {}.", color);
    Ok(())
}

fn deployment(color: &str, version: &str) -> Deployment {
    let mut labels = BTreeMap::new();
    labels.insert("app".into(), APP.into());
    labels.insert("env".into(), color.into());
    labels.insert("version".into(), version.into());

    Deployment {
        metadata: ObjectMeta {
            name: Some(format!("{}-{}", APP, color)),
            namespace: Some(NS.into()),
            labels: Some(labels.clone()),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(2),
            selector: k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector {
                match_labels: Some({
                    let mut m = BTreeMap::new();
                    m.insert("app".into(), APP.into());
                    m.insert("env".into(), color.into());
                    m
                }),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta { labels: Some(labels), ..Default::default() }),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: APP.into(),
                        image: Some(IMAGE.into()),
                        image_pull_policy: Some("IfNotPresent".into()),
                        env: Some(vec![
                            env_kv("VERSION", version),
                            env_kv("COLOR", color),
                            env_kv("PORT", "8080"),
                        ]),
                        ports: Some(vec![container_port(PORT)]),
                        readiness_probe: Some(http_probe("/healthz", PORT, 2, 5)),
                        liveness_probe: Some(http_probe("/healthz", PORT, 5, 10)),
                        resources: None,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn service(color: &str) -> Service {
    let mut selector = BTreeMap::new();
    selector.insert("app".into(), APP.into());
    selector.insert("env".into(), color.into());

    Service {
        metadata: ObjectMeta { name: Some(APP.into()), namespace: Some(NS.into()), ..Default::default() },
        spec: Some(ServiceSpec {
            selector: Some(selector),
            ports: Some(vec![ServicePort {
                name: Some("http".into()),
                port: 80,
                target_port: Some(k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(PORT)),
                protocol: Some("TCP".into()),
                ..Default::default()
            }]),
            type_: Some("ClusterIP".into()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn env_kv(name: &str, value: &str) -> k8s_openapi::api::core::v1::EnvVar {
    k8s_openapi::api::core::v1::EnvVar {
        name: name.into(),
        value: Some(value.into()),
        ..Default::default()
    }
}

fn container_port(port: i32) -> k8s_openapi::api::core::v1::ContainerPort {
    k8s_openapi::api::core::v1::ContainerPort {
        container_port: port,
        ..Default::default()
    }
}

fn http_probe(path: &str, port: i32, initial_delay: i32, period: i32) -> k8s_openapi::api::core::v1::Probe {
    use k8s_openapi::api::core::v1::{HTTPGetAction, Probe};
    Probe {
        http_get: Some(HTTPGetAction {
            path: Some(path.into()),
            port: k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(port),
            scheme: Some("HTTP".into()),
            ..Default::default()
        }),
        initial_delay_seconds: Some(initial_delay),
        period_seconds: Some(period),
        ..Default::default()
    }
}
