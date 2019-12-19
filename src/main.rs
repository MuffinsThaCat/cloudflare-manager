use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use reqwest::{header, Client};
use serde_derive::Deserialize;
use serde_json::json;
use tokio::{fs::File, io::AsyncReadExt};

use std::env;
use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug)]
struct State {
    content: String,
    zone_id: String,
    zone_name: String,
    client: Client,
}

#[derive(Debug, Deserialize)]
struct Request {
    name: String,
}

async fn create_dns_record(
    data: web::Data<Arc<State>>,
    request: web::Json<Request>,
) -> impl Responder {
    handle_create_dns_record(data, request).await
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let (ip, port, workers, secret_path) = read_env();

    let auth_key = read_file(&secret_path, "auth_key")
        .await?
        .trim()
        .to_string();
    let token = read_file(&secret_path, "token").await?.trim().to_string();
    let content = read_file(&secret_path, "content").await?.trim().to_string();
    let zone_name = read_file(&secret_path, "zone_name")
        .await?
        .trim()
        .to_string();
    let zone_id = read_file(&secret_path, "zone_id").await?.trim().to_string();

    let client = build_client(&auth_key, &token).map_err(|_| std::io::ErrorKind::Other)?;

    let state = Arc::new(State {
        content,
        zone_id,
        zone_name,
        client,
    });

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .route("/api/v1/dns_records", web::post().to(create_dns_record))
            .default_service(web::route().to(HttpResponse::NotFound))
            .wrap(middleware::Logger::default())
    })
    .bind(format!("{}:{}", ip, port))?
    .workers(workers)
    .start()
    .await
}

fn read_env() -> (String, u64, usize, String) {
    (
        env::var("SERVER_IP").unwrap_or_else(|_| "127.0.0.1".to_string()),
        env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .expect("can not parse server port"),
        env::var("SERVER_WORKERS")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .expect("can not parse server workers"),
        env::var("SECRET_PATH").unwrap_or_else(|_| "secret".to_string()),
    )
}

fn build_client(auth_key: &str, token: &str) -> Result<Client, reqwest::Error> {
    let mut headers = header::HeaderMap::new();
    let x_auth_key = header::HeaderValue::from_str(auth_key).expect("invalid auth_key");
    let auth_data =
        header::HeaderValue::from_str(&format!("Bearer {}", token)).expect("invalid token");
    headers.insert(header::AUTHORIZATION, auth_data);
    headers.insert(header::HeaderName::from_static("x-auth-key"), x_auth_key);
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("cloudflare-manager"),
    );
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    Client::builder()
        .use_default_tls()
        .default_headers(headers)
        .build()
}

async fn read_file(path: &str, name: &str) -> std::io::Result<String> {
    let file_path = Path::new(path).join(name);
    let mut data = vec![];
    let mut file = File::open(file_path).await?;
    file.read_to_end(&mut data).await?;
    Ok(String::from_utf8(data).unwrap_or_else(|_| panic!("invalid {}", name)))
}

async fn handle_create_dns_record(
    data: web::Data<Arc<State>>,
    request: web::Json<Request>,
) -> std::io::Result<String> {
    let name = format!("{}.{}", request.name, data.zone_name);
    let response = data
        .client
        .post(&create_dns_record_url(&data.zone_id))
        .body(create_dns_record_body(&name, &data.content))
        .send()
        .await
        .map_err(|e| {
            log::error!("request error: {:?}", e);
            std::io::ErrorKind::Other
        })?;
    log::info!(
        "created: {}, http_status: {}, response: {:?}",
        name,
        response.status(),
        response.text().await
    );

    Ok(json!({ "status": "ok" }).to_string())
}

fn create_dns_record_url(zone_id: &str) -> String {
    format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
        zone_id
    )
}

fn create_dns_record_body(name: &str, content: &str) -> String {
    json!({
        "type": "A",
        "name": name,
        "content": content,
        "ttl": 1,
        "priority": 10,
        "proxied": true,
    })
    .to_string()
}
