use anyhow::Result;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize)]
struct BlockedConfig {
    bloqueados: Vec<String>,
}

pub fn load(path: &str) -> Result<HashSet<String>> {
    let content = std::fs::read_to_string(path)?;
    let config: BlockedConfig = serde_json::from_str(&content)?;
    Ok(config.bloqueados.into_iter().collect())
}

pub fn is_blocked(domain: &str, list: &HashSet<String>) -> bool {
    list.contains(domain)
}
