
use anyhow::Result;
use kube::{
    api::{Api, ListParams, PostParams, Patch, PatchParams, ResourceExt},
    Client,
};
use k8s_openapi::{
    api::core::v1::{ConfigMap, Node, Pod},
};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

const NS: &str = "aula07";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PodReq {
    cpu_millicores: i64,
    mem_bytes: i64,
    needs_gpu: bool,
    prefer_spot: bool,
    zone_hint: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct NodeSize {
    name: &'static str,
    cpu_millicores: i64,
    mem_bytes: i64,
}

lazy_static! {
    static ref DEFAULT_SIZES: Vec<NodeSize> = vec![
        NodeSize { name: "nano",  cpu_millicores:  500,  mem_bytes:  512 * 1024 * 1024 },
        NodeSize { name: "small", cpu_millicores: 2000,  mem_bytes:  2 * 1024 * 1024 * 1024 },
        NodeSize { name: "med",   cpu_millicores: 4000,  mem_bytes:  8 * 1024 * 1024 * 1024 },
        NodeSize { name: "large", cpu_millicores: 8000,  mem_bytes: 16 * 1024 * 1024 * 1024 },
        NodeSize { name: "xlarge",cpu_millicores:16000,  mem_bytes: 32 * 1024 * 1024 * 1024 },
    ];
}

fn parse_cpu_to_millicores(s: &str) -> Option<i64> {
    // Accept forms like "500m", "1", "2.5"
    if s.ends_with('m') {
        let num = s.trim_end_matches('m').parse::<f64>().ok()?;
        return Some(num as i64);
    }
    let num = s.parse::<f64>().ok()?;
    Some((num * 1000.0) as i64)
}

fn parse_mem_to_bytes(s: &str) -> Option<i64> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?i)^\s*([0-9]*\.?[0-9]+)\s*([kmgte]i?)?\s*$").unwrap();
    }
    let caps = RE.captures(s)?;
    let val = caps.get(1)?.as_str().parse::<f64>().ok()?;
    let unit = caps.get(2).map(|m| m.as_str().to_lowercase()).unwrap_or_default();
    let mul: f64 = match unit.as_str() {
        "" => 1.0,
        "k" => 1e3,   "m" => 1e6,    "g" => 1e9,    "t" => 1e12,   "e" => 1e18,
        "ki" => 1024.0,
        "mi" => 1024.0_f64.powi(2),
        "gi" => 1024.0_f64.powi(3),
        "ti" => 1024.0_f64.powi(4),
        "ei" => 1024.0_f64.powi(6),
        _ => 1.0,
    };
    Some((val * mul) as i64)
}

fn extract_pod_requests(pod: &Pod) -> PodReq {
    let mut r = PodReq::default();
    if let Some(spec) = &pod.spec {
        for c in &spec.containers {
            if let Some(res) = &c.resources {
                if let Some(req) = &res.requests {
                    if let Some(cpu) = req.get("cpu") {
                        if let Some(v) = parse_cpu_to_millicores(cpu.0.as_str()) {
                            r.cpu_millicores += v;
                        }
                    }
                    if let Some(mem) = req.get("memory") {
                        if let Some(v) = parse_mem_to_bytes(mem.0.as_str()) {
                            r.mem_bytes += v;
                        }
                    }
                }
            }
        }
        if let Some(ns) = &spec.node_selector {
            if let Some(z) = ns.get("topology.kubernetes.io/zone") {
                r.zone_hint = Some(z.clone());
            }
            if let Some(gpu) = ns.get("demo/require-gpu") {
                if gpu == "true" { r.needs_gpu = true; }
            }
        }
    }
    if let Some(ann) = &pod.metadata.annotations {
        if ann.get("prefer-spot").map(|s| s == "true").unwrap_or(false) {
            r.prefer_spot = true;
        }
    }
    r
}

fn choose_smallest_fitting_size(req: &PodReq) -> NodeSize {
    for s in DEFAULT_SIZES.iter() {
        if s.cpu_millicores >= req.cpu_millicores && s.mem_bytes >= req.mem_bytes {
            return s.clone();
        }
    }
    DEFAULT_SIZES.last().cloned().unwrap()
}

fn build_plan_yaml(pod: &Pod, req: &PodReq, size: &NodeSize) -> String {
    let zone = req.zone_hint.clone().unwrap_or_else(|| "us-east-1a".to_string());
    let capacity = if req.prefer_spot { "spot" } else { "on-demand" };
    let gpu_req = if req.needs_gpu {
                "  - key: nvidia.com/gpu\n    operator: Exists\n"
    } else {
        ""
    };

        let mem = format!("{}Mi", size.mem_bytes / 1024 / 1024);

        format!(
                concat!(
                        "apiVersion: karpenter.sh/v1alpha5\n",
                        "kind: Provisioner\n",
                        "metadata:\n",
                        "  name: prov-{pod}\n",
                        "spec:\n",
                        "  requirements:\n",
                        "  - key: topology.kubernetes.io/zone\n",
                        "    operator: In\n",
                        "    values: [\"{zone}\"]\n",
                        "  - key: karpenter.sh/capacity-type\n",
                        "    operator: In\n",
                        "    values: [\"{capacity}\"]\n",
                        "{gpu_req}",
                        "  limits:\n",
                        "    resources:\n",
                        "      cpu: \"{cpu}m\"\n",
                        "      memory: \"{mem}\"\n",
                        "  consolidation:\n",
                        "    enabled: true\n",
                        "  labels:\n",
                        "    demo/size: \"{size}\"\n"
                ),
                pod = pod.name_any(),
                zone = zone,
                capacity = capacity,
                gpu_req = gpu_req,
                cpu = size.cpu_millicores,
                mem = mem,
                size = size.name
        )
}

async fn upsert_configmap_planned(client: Client, name: &str, data: serde_json::Value) -> Result<()> {
    let api: Api<ConfigMap> = Api::namespaced(client, NS);
    let pp = PostParams::default();
    let mut cm = ConfigMap::default();
    cm.metadata.name = Some(name.to_string());
    let mut data_map: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    if let Some(plan) = data.get("plan_yaml").and_then(|v| v.as_str()) {
        data_map.insert("plan.yaml".into(), plan.into());
    }
    data_map.insert("info.json".into(), serde_json::to_string_pretty(&data)?);
    cm.data = Some(data_map);
    match api.create(&pp, &cm).await {
        Ok(_) => {
            info!("ConfigMap {} created", name);
        },
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            let patch = Patch::Apply(&cm);
            let params = PatchParams::apply("aula07").force();
            let _ = api.patch(name, &params, &patch).await?;
            info!("ConfigMap {} patched", name);
        },
        Err(e) => {
            error!("Failed to upsert ConfigMap {}: {:?}", name, e);
        }
    }
    Ok(())
}

async fn list_pending_pods(client: Client) -> Result<Vec<Pod>> {
    let pods: Api<Pod> = Api::all(client);
    let lp = ListParams::default().fields("status.phase=Pending");
    let list = pods.list(&lp).await?;
    Ok(list.items)
}

fn node_capacity(node: &Node) -> (i64, i64) {
    let mut cpu_m = 0_i64;
    let mut mem_b = 0_i64;
    if let Some(status) = &node.status {
        if let Some(cap) = &status.capacity {
            if let Some(cpu) = cap.get("cpu") {
                if let Some(v) = parse_cpu_to_millicores(cpu.0.as_str()) {
                    cpu_m = v;
                } else if let Ok(v) = cpu.0.parse::<i64>() {
                    cpu_m = v * 1000;
                }
            }
            if let Some(mem) = cap.get("memory") {
                if let Some(v) = parse_mem_to_bytes(mem.0.as_str()) {
                    mem_b = v;
                }
            }
        }
    }
    (cpu_m, mem_b)
}

async fn pods_on_node(client: Client, nodename: &str) -> Result<Vec<Pod>> {
    let pods: Api<Pod> = Api::all(client);
    let lp = ListParams::default().fields(&format!("spec.nodeName={}", nodename));
    let list = pods.list(&lp).await?;
    Ok(list.items)
}

fn sum_requests(pods: &[Pod]) -> (i64, i64) {
    let mut cpu = 0_i64;
    let mut mem = 0_i64;
    for p in pods {
        let r = extract_pod_requests(p);
        cpu += r.cpu_millicores;
        mem += r.mem_bytes;
    }
    (cpu, mem)
}

#[tokio::main]
async fn main() -> Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env()
        .add_directive("mini_karpenter_rs=info".parse().unwrap_or_else(|_| "info".parse().unwrap()))
        .add_directive("kube=info".parse().unwrap())
    ).init();

    let client = Client::try_default().await?;

    // Best-effort: verificar namespace
    let ns_api: Api<k8s_openapi::api::core::v1::Namespace> = Api::all(client.clone());
    let _ = ns_api.get(NS).await;

    loop {
        // Parte 1: Pending => Plan
        match list_pending_pods(client.clone()).await {
            Ok(pending) => {
                for pod in pending {
                    let name = pod.name_any();
                    let req = extract_pod_requests(&pod);
                    let size = choose_smallest_fitting_size(&req);
                    let plan_yaml = build_plan_yaml(&pod, &req, &size);
                    let data = json!({
                        "pod": name,
                        "namespace": pod.namespace().unwrap_or_default(),
                        "requests": req,
                        "chosen_size": size,
                        "plan_yaml": plan_yaml
                    });
                    let cm_name = format!("plan-{}", name);
                    if let Err(e) = upsert_configmap_planned(client.clone(), &cm_name, data).await {
                        error!("upsert plan cm error: {:?}", e);
                    } else {
                        info!("Plan generated for Pending pod {}", name);
                    }
                }
            }
            Err(e) => error!("list pending pods error: {:?}", e),
        }

        // Parte 2: Consolidação
        let nodes: Api<Node> = Api::all(client.clone());
        if let Ok(list) = nodes.list(&ListParams::default()).await {
            for n in list.items {
                let nodename = n.name_any();
                let (cpu_cap_m, mem_cap_b) = node_capacity(&n);
                if cpu_cap_m == 0 || mem_cap_b == 0 {
                    continue;
                }
                if let Ok(pods) = pods_on_node(client.clone(), &nodename).await {
                    if pods.is_empty() { continue; }
                    let (cpu_req_m, mem_req_b) = sum_requests(&pods);
                    let cpu_util = cpu_req_m as f64 / cpu_cap_m as f64;
                    let mem_util = mem_req_b as f64 / mem_cap_b as f64;
                    let util = cpu_util.max(mem_util);
                    if util < 0.20 {
                        let pod_names: Vec<String> = pods.iter().map(|p| p.name_any()).collect();
                        let data = json!({
                            "node": nodename,
                            "cpu_req_m": cpu_req_m,
                            "mem_req_b": mem_req_b,
                            "cpu_cap_m": cpu_cap_m,
                            "mem_cap_b": mem_cap_b,
                            "utilization_max_of_cpu_mem": util,
                            "suggestion": "Drain this node respecting PDB/TopologySpread and reschedule pods elsewhere. If stable, terminate node."
                        });
                        let cm_name = format!("consolidation-{}", nodename);
                        let api: Api<ConfigMap> = Api::namespaced(client.clone(), NS);
                        let pp = PostParams::default();
                        let mut cm = ConfigMap::default();
                        cm.metadata.name = Some(cm_name.clone());
                        let mut m = std::collections::BTreeMap::new();
                        m.insert("pods.txt".into(), pod_names.join("\n"));
                        m.insert("info.json".into(), serde_json::to_string_pretty(&data).unwrap());
                        cm.data = Some(m);
                        match api.create(&pp, &cm).await {
                            Ok(_) => info!("Consolidation suggestion for node {}", nodename),
                            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                                let patch = Patch::Apply(&cm);
                                let params = PatchParams::apply("aula07").force();
                                let _ = api.patch(&cm_name, &params, &patch).await;
                                info!("Consolidation suggestion updated for node {}", nodename);
                            },
                            Err(e) => error!("consolidation cm error: {:?}", e),
                        }
                    }
                }
            }
        }

        sleep(Duration::from_secs(10)).await;
    }
}
