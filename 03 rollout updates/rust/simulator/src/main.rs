
use clap::Parser;

/// Simula RollingUpdate: dado N, maxUnavailable (n/%), maxSurge (n/%).
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Réplicas desejadas (N)
    replicas: u32,
    /// maxUnavailable (ex.: 0, 2, 25%)
    max_unavailable: String,
    /// maxSurge (ex.: 0, 2, 25%)
    max_surge: String,
}

fn parse_threshold(replica_count: u32, value: &str, round_up: bool) -> u32 {
    if value.ends_with('%') {
        let pct_str = &value[..value.len()-1];
        if let Ok(pct) = pct_str.parse::<f32>() {
            let raw = (pct / 100.0) * replica_count as f32;
            if round_up { raw.ceil() as u32 } else { raw.floor() as u32 }
        } else {
            eprintln!("Valor percentual inválido: {}", value);
            0
        }
    } else {
        value.parse::<u32>().unwrap_or_else(|_| {
            eprintln!("Valor numérico inválido: {}", value);
            0
        })
    }
}

fn perform_rollout_step(
    replica_count: u32,
    old_running: u32,
    new_running: u32,
    max_unavail_count: u32,
    max_surge_count: u32,
) -> (u32, u32, u32, u32) {
    let can_create = max_surge_count;
    let can_delete = max_unavail_count.min(old_running);

    let to_create = can_create.min(replica_count.saturating_sub(new_running));
    let new_running_after = new_running + to_create;

    let need_to_reduce_to = replica_count;
    let excess_total = new_running_after + old_running;
    let to_delete = if excess_total > need_to_reduce_to {
        (excess_total - need_to_reduce_to).min(can_delete)
    } else {
        0
    };

    let old_running_after = old_running.saturating_sub(to_delete);
    (new_running_after, old_running_after, to_create, to_delete)
}

fn simulate_rolling_update(replica_count: u32, max_unavailable: &str, max_surge: &str) {
    let max_unavail_count = parse_threshold(replica_count, max_unavailable, false);
    let max_surge_count = parse_threshold(replica_count, max_surge, true);
    println!(
        "Iniciando rollout: replicas={} max_unavail={} max_surge={}",
        replica_count, max_unavail_count, max_surge_count
    );
    if max_surge_count == 0 && max_unavail_count == 0 {
        eprintln!("Sem progresso possível: maxSurge e maxUnavailable são 0. Abortando.");
        return;
    }
    let mut old_running = replica_count;
    let mut new_running: u32 = 0;
    let mut step: u32 = 0;

    while new_running < replica_count {
        step += 1;
        let (nr, or_, created, deleted) = perform_rollout_step(
            replica_count,
            old_running,
            new_running,
            max_unavail_count,
            max_surge_count,
        );
        if nr == new_running && or_ == old_running {
            eprintln!("Nenhum progresso nesta etapa (criados=0, deletados=0). Abortando.");
            break;
        }
        new_running = nr;
        old_running = or_;
        println!(
            "Step {}: +{} novos (total novos={}), -{} antigos (restam antigos={}) -> pods disponíveis agora: {}",
            step, created, new_running, deleted, old_running, new_running + old_running
        );
    }
    println!(
        "Rollout completo em {} etapas. Novos pods = {}, Antigos restantes = {}",
        step, new_running, old_running
    );
}

fn main() {
    let args = Args::parse();
    simulate_rolling_update(args.replicas, &args.max_unavailable, &args.max_surge);
}
