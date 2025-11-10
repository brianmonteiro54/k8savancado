use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use kube::{Api, Client};
use kube::api::{PostParams, PatchParams, Patch, DeleteParams};
use kube::core::{ApiResource, DynamicObject, GroupVersionKind};
use k8s_openapi::api::apps::v1::Deployment;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "canaryctl")]
#[command(about = "Controle simples de Canary com Kubernetes + Istio", long_about=None)]
struct Cli {
    /// Namespace alvo (default: default)
    #[arg(long, default_value = "default")]
    ns: String,

    /// Nome base do app (Deployment/Service/VS)
    #[arg(long, default_value = "versioned-echo")]
    app: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Cria o Deployment canário (v2) via API
    CreateCanary {
        /// Imagem do canário (default: versioned-echo:v2)
        #[arg(long, default_value = "versioned-echo:v2")]
        image: String,

        /// Réplicas da v2
        #[arg(long, default_value_t = 2)]
        replicas: i32,
    },

    /// Remove o Deployment canário (v2)
    DeleteCanary,

    /// Ajusta pesos do VirtualService (ex: 90 10)
    SetTraffic {
        /// Peso para v1 (0-100)
        v1: i32,
        /// Peso para v2 (0-100)
        v2: i32,
        /// Nome do VirtualService
        #[arg(long, default_value = "versioned-echo-virtualservice")]
        vs: String,
        /// Host do serviço no VS (ex: versioned-echo)
        #[arg(long, default_value = "versioned-echo")]
        host: String,
    },

    /// Rollback imediato para 100% v1
    Rollback {
        #[arg(long, default_value = "versioned-echo-virtualservice")]
        vs: String,
        #[arg(long, default_value = "versioned-echo")]
        host: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct PromResult {
    status: String,
    data: PromData,
}
#[derive(Debug, Serialize, Deserialize)]
struct PromData {
    result: Vec<PromVector>,
}
#[derive(Debug, Serialize, Deserialize)]
struct PromVector {
    value: (f64, String),
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::CreateCanary { image, replicas } => create_canary(&cli.ns, &cli.app, &image, replicas).await?,
        Commands::DeleteCanary => delete_canary(&cli.ns, &cli.app).await?,
        Commands::SetTraffic { v1, v2, vs, host } => set_traffic(&cli.ns, &vs, &host, v1, v2).await?,
        Commands::Rollback { vs, host } => set_traffic(&cli.ns, &vs, &host, 100, 0).await?,
    }
    Ok(())
}

async fn create_canary(ns: &str, app: &str, image: &str, replicas: i32) -> Result<()> {
    let client = Client::try_default().await?;
    let api: Api<Deployment> = Api::namespaced(client, ns);
    let name = format!("{app}-v2");

    let deploy_json = json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": { "name": name },
        "spec": {
            "replicas": replicas,
            "selector": { "matchLabels": { "app": app, "version": "v2" } },
            "template": {
                "metadata": { "labels": { "app": app, "version": "v2" } },
                "spec": {
                    "containers": [{
                        "name": app,
                        "image": image,
                        "env": [{ "name": "APP_VERSION", "value": "v2" }],
                        "ports": [{ "containerPort": 8080 }],
                        "readinessProbe": { "httpGet": { "path": "/health", "port": 8080 }, "initialDelaySeconds": 2 },
                        "livenessProbe": { "httpGet": { "path": "/health", "port": 8080 }, "initialDelaySeconds": 2 }
                    }]
                }
            }
        }
    });

    let d: Deployment = serde_json::from_value(deploy_json)?;
    match api.create(&PostParams::default(), &d).await {
        Ok(_) => println!("✅ Canary v2 criado: imagem={image}, replicas={replicas}"),
        Err(kube::Error::Api(err)) if err.code == 409 => {
            println!("ℹ️  Canary v2 já existe, nada a fazer.");
        }
        Err(e) => {
            return Err(e).with_context(|| "Falha ao criar canário v2");
        }
    }
    Ok(())
}

async fn delete_canary(ns: &str, app: &str) -> Result<()> {
    let client = Client::try_default().await?;
    let api: Api<Deployment> = Api::namespaced(client, ns);
    let name = format!("{app}-v2");
    api.delete(&name, &DeleteParams::default()).await
        .with_context(|| "Falha ao deletar canário v2")?;
    println!("🗑️  Canary v2 removido.");
    Ok(())
}

async fn set_traffic(ns: &str, vs_name: &str, host: &str, v1: i32, v2: i32) -> Result<()> {
    if v1 + v2 != 100 || v1 < 0 || v2 < 0 {
        anyhow::bail!("Pesos inválidos: v1+v2 deve ser 100 e não negativos.");
    }
    let client = Client::try_default().await?;
    let gvk = GroupVersionKind::gvk("networking.istio.io", "v1beta1", "VirtualService");
    let mut ar = ApiResource::from_gvk(&gvk);
    ar.plural = "virtualservices".into();
    let api: Api<DynamicObject> = Api::namespaced_with(client, ns, &ar);

    let patch = json!({
        "spec": {
            "http": [{
                "route": [
                    { "destination": { "host": host, "subset": "v1" }, "weight": v1 },
                    { "destination": { "host": host, "subset": "v2" }, "weight": v2 }
                ]
            }]
        }
    });

    let params = PatchParams::default();
    api.patch(vs_name, &params, &Patch::Merge(&patch)).await
        .with_context(|| "Falha ao atualizar VirtualService")?;
    println!("🚦 Tráfego atualizado: v1={v1}% | v2={v2}%");
    Ok(())
}
