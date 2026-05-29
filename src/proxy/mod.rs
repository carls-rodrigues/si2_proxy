pub mod blocker;
pub mod fetcher;
pub mod filter;
pub mod logger;
pub mod service;
pub mod utils;

use std::collections::HashMap;

pub struct AppState {
    pub blocked: Vec<String>,
    pub words: HashMap<String, String>,
    pub client: reqwest::Client,
}
