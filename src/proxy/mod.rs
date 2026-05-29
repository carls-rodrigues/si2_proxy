pub mod blocker;
pub mod fetcher;
pub mod filter;
pub mod logger;
pub mod service;
pub mod utils;

use std::collections::HashSet;

use filter::WordFilter;

pub struct AppState {
    pub blocked: HashSet<String>,
    pub words: WordFilter,
    pub client: reqwest::Client,
}
