use axum::{routing::{any, get}, Router};
use std::net::SocketAddr;
use std::sync::Arc;

mod proxy;
use proxy::AppState;

#[tokio::main]
async fn main() {
    let blocked =
        proxy::blocker::load("config/blocked.json").expect("Falha ao carregar blocked.json");
    let words =
        proxy::filter::load("config/words.json").expect("Falha ao carregar words.json");

    std::fs::create_dir_all("logs").expect("Falha ao criar diretório logs/");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; SIProxy/1.0)")
        .build()
        .expect("Falha ao criar cliente HTTP");

    let state = Arc::new(AppState { blocked, words, client });

    let app = Router::new()
        .route("/", get(proxy::service::home))
        .route("/{*path}", any(proxy::service::handle))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
    println!("SI Proxy rodando em http://localhost:5000");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
