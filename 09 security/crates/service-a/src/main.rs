use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use anyhow::{Context, Result};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status":"ok"}))
}

#[actix_web::main]
async fn main() -> Result<()> {
    let cert_path = std::env::var("TLS_CERT_PATH").unwrap_or_else(|_| "/tls/tls.crt".into());
    let key_path = std::env::var("TLS_KEY_PATH").unwrap_or_else(|_| "/tls/tls.key".into());
    let addr = "0.0.0.0:8443";

    let tls_config = load_rustls(&cert_path, &key_path)?;

    println!("🔐 service-a ouvindo em https://{addr}");
    HttpServer::new(|| App::new().service(healthz))
        .bind_rustls(addr, tls_config)?
        .run()
        .await?;
    Ok(())
}

fn load_rustls(cert_path: &str, key_path: &str) -> Result<ServerConfig> {
    // Cert chain
    let mut cert_reader = BufReader::new(File::open(cert_path).with_context(|| format!("abrindo {}", cert_path))?);
    let certs: Vec<Certificate> = certs(&mut cert_reader)
        .context("lendo cadeia de certificados")?
        .into_iter()
        .map(Certificate)
        .collect();

    // Private key (PKCS#8)
    let mut key_reader = BufReader::new(File::open(key_path).with_context(|| format!("abrindo {}", key_path))?);
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(&mut key_reader)
        .context("lendo chave privada pkcs8")?
        .into_iter()
        .map(PrivateKey)
        .collect();

    let key = keys
        .pop()
        .context("nenhuma chave privada PKCS#8 encontrada em tls.key")?;

    let cfg = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    Ok(cfg)
}
