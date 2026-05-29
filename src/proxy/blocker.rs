use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
struct BlockedConfig {
    bloqueados: Vec<String>,
}

pub fn load(path: &str) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    let config: BlockedConfig = serde_json::from_str(&content)?;
    Ok(config.bloqueados)
}

pub fn is_blocked(domain: &str, list: &[String]) -> bool {
    list.iter().any(|b| b == domain)
}
