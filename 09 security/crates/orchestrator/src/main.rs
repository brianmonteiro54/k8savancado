use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use kube::{Api, Client};
use kube::api::{PostParams, DeleteParams, ObjectMeta, PatchParams, Patch};
use k8s_openapi::api::core::v1::{Namespace, ServiceAccount, Pod, PodSpec, Container};
use k8s_openapi::api::rbac::v1::{Role, RoleBinding, PolicyRule, RoleRef, Subject};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser)]
#[command(name = "orchestrator", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Cria namespace, ServiceAccount, Role, RoleBinding e Pod de demonstração
    Bootstrap,
    /// Remove recursos criados pela demo
    Cleanup,
    /// Anota a ServiceAccount analytics-sa para identidade federada (IRSA / Workload Identity)
    Annotate {
        #[arg(long, value_enum)]
        provider: Provider,
        /// Valor da anotação (ARN da Role no EKS, GSA no GKE, ClientId da Managed Identity no AKS)
        #[arg(long)]
        value: String,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Provider { Eks, Gke, Aks }

const NS:&str = "analytics";
const SA:&str = "analytics-sa";

#[tokio::main]
async fn main() -> Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Bootstrap => bootstrap().await?,
        Commands::Cleanup => cleanup().await?,
        Commands::Annotate { provider, value } => annotate(provider, &value).await?,
    }
    Ok(())
}

async fn bootstrap() -> Result<()> {
    let client = Client::try_default().await?;

    // Namespace
    let ns_api: Api<Namespace> = Api::all(client.clone());
    let ns = Namespace {
        metadata: ObjectMeta{ name: Some(NS.into()), ..Default::default() },
        ..Default::default()
    };
    match ns_api.create(&PostParams::default(), &ns).await {
        Ok(_) => println!("Namespace '{}' criado.", NS),
        Err(e) if is_already_exists(&e.to_string()) => println!("Namespace '{}' já existe.", NS),
        Err(e) => return Err(e.into())
    }

    // ServiceAccount
    let sa_api: Api<ServiceAccount> = Api::namespaced(client.clone(), NS);
    let sa = ServiceAccount{
        metadata: ObjectMeta{ name: Some(SA.into()), ..Default::default() },
        ..Default::default()
    };
    match sa_api.create(&PostParams::default(), &sa).await {
        Ok(_) => println!("ServiceAccount '{}' criada.", SA),
        Err(e) if is_already_exists(&e.to_string()) => println!("ServiceAccount '{}' já existe.", SA),
        Err(e) => return Err(e.into())
    }

    // Role mínima
    let role_api: Api<Role> = Api::namespaced(client.clone(), NS);
    let role = Role{
        metadata: ObjectMeta{ name: Some("analytics-read".into()), ..Default::default() },
        rules: Some(vec![
            PolicyRule{
                api_groups: Some(vec!["".into()]), // core/v1
                resources: Some(vec!["pods".into()]),
                verbs: vec!["get".into(), "list".into()],
                ..Default::default()
            },
            PolicyRule{
                api_groups: Some(vec!["".into()]),
                resources: Some(vec!["configmaps".into()]),
                verbs: vec!["get".into()],
                ..Default::default()
            }
        ]),
        ..Default::default()
    };
    match role_api.create(&PostParams::default(), &role).await {
        Ok(_) => println!("Role 'analytics-read' criada."),
        Err(e) if is_already_exists(&e.to_string()) => println!("Role 'analytics-read' já existe."),
        Err(e) => return Err(e.into())
    }

    // RoleBinding
    let rb_api: Api<RoleBinding> = Api::namespaced(client.clone(), NS);
    let rb = RoleBinding{
        metadata: ObjectMeta{ name: Some("bind-analytics-read".into()), ..Default::default()},
        role_ref: RoleRef{
            api_group: "rbac.authorization.k8s.io".into(),
            kind: "Role".into(),
            name: "analytics-read".into(),
        },
        subjects: Some(vec![Subject{
            kind: "ServiceAccount".into(),
            name: SA.into(),
            namespace: Some(NS.into()),
            ..Default::default()
        }]),
        ..Default::default()
    };
    match rb_api.create(&PostParams::default(), &rb).await {
        Ok(_) => println!("RoleBinding 'bind-analytics-read' criado."),
        Err(e) if is_already_exists(&e.to_string()) => println!("RoleBinding 'bind-analytics-read' já existe."),
        Err(e) => return Err(e.into())
    }

    // Pod demonstrativo
    let pod_api: Api<Pod> = Api::namespaced(client.clone(), NS);
    let pod = Pod{
        metadata: ObjectMeta{ name: Some("test-pod".into()), ..Default::default() },
        spec: Some(PodSpec{
            service_account_name: Some(SA.into()),
            containers: vec![Container{
                name: "app".into(),
                image: Some("alpine:3.19".into()),
                command: Some(vec!["/bin/sh".into(), "-lc".into(), "echo hello; sleep 3600".into()]),
                ..Default::default()
            }],
            restart_policy: Some("Always".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    match pod_api.create(&PostParams::default(), &pod).await {
        Ok(_) => println!("Pod 'test-pod' criado no namespace '{}'.", NS),
        Err(e) if is_already_exists(&e.to_string()) => println!("Pod 'test-pod' já existe."),
        Err(e) => return Err(e.into())
    }

    println!("
✅ Bootstrap concluído.");
    Ok(())
}

async fn annotate(provider: Provider, value: &str) -> Result<()> {
    let client = Client::try_default().await?;
    let sa_api: Api<ServiceAccount> = Api::namespaced(client, NS);
    let (key, val) = match provider {
        Provider::Eks => ("eks.amazonaws.com/role-arn", value.to_string() ),
        Provider::Gke => ("iam.gke.io/gcp-service-account", value.to_string() ),
        Provider::Aks => ("azure.workload.identity/client-id", value.to_string() ),
    };
    let patch = serde_json::json!({
        "metadata": { "annotations": { key: val } }
    });
    sa_api.patch(SA, &PatchParams::default(), &Patch::Merge(&patch)).await?;
    println!("✅ Anotação aplicada na SA '{}': {}={}", SA, key, value);
    Ok(())
}

async fn cleanup() -> Result<()> {
    let client = Client::try_default().await?;

    let pod_api: Api<Pod> = Api::namespaced(client.clone(), NS);
    let _ = pod_api.delete("test-pod", &DeleteParams::default()).await;

    let rb_api: Api<RoleBinding> = Api::namespaced(client.clone(), NS);
    let _ = rb_api.delete("bind-analytics-read", &DeleteParams::default()).await;

    let role_api: Api<Role> = Api::namespaced(client.clone(), NS);
    let _ = role_api.delete("analytics-read", &DeleteParams::default()).await;

    let sa_api: Api<ServiceAccount> = Api::namespaced(client.clone(), NS);
    let _ = sa_api.delete(SA, &DeleteParams::default()).await;

    // Namespace por último
    let ns_api: Api<Namespace> = Api::all(client.clone());
    let _ = ns_api.delete(NS, &DeleteParams::default()).await;

    println!("🧹 Cleanup solicitado (recursos serão removidos em background).");
    Ok(())
}

fn is_already_exists(e: &str) -> bool {
    e.contains("AlreadyExists") || e.contains("Conflict") // cobre casos comuns
}
