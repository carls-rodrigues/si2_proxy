use axum::{
    Router,
    routing::{any, get},
};
use std::net::SocketAddr;
use std::sync::Arc;

mod proxy;
use proxy::AppState;

const BLOCKED_CONFIG_PATH: &str = "config/blocked.json";
const WORDS_CONFIG_PATH: &str = "config/words.json";
const LOG_DIR: &str = "logs";
const LISTEN_PORT: u16 = 5000;
const USER_AGENT: &str = "Mozilla/5.0 (compatible; SIProxy/1.0)";

#[tokio::main]
async fn main() {
    let blocked = proxy::blocker::load(BLOCKED_CONFIG_PATH).expect("Falha ao carregar blocked.json");
    let words = proxy::filter::load(WORDS_CONFIG_PATH).expect("Falha ao carregar words.json");

    std::fs::create_dir_all(LOG_DIR).expect("Falha ao criar diretório logs/");

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("Falha ao criar cliente HTTP");

    let state = Arc::new(AppState {
        blocked,
        words,
        client,
    });

    let app = Router::new()
        .route("/", get(proxy::service::home))
        .route("/{*path}", any(proxy::service::handle))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], LISTEN_PORT));
    println!("SI Proxy rodando em http://localhost:{LISTEN_PORT}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
