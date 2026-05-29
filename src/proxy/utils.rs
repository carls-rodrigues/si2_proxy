use anyhow::{Result, anyhow};
use url::Url;

pub fn extract_target(path: &str, query: Option<&str>) -> Result<Url> {
    let raw = path.trim_start_matches('/');

    let normalized = if raw.starts_with("http:/") && !raw.starts_with("http://") {
        raw.replacen("http:/", "http://", 1)
    } else if raw.starts_with("https:/") && !raw.starts_with("https://") {
        raw.replacen("https:/", "https://", 1)
    } else {
        raw.to_string()
    };

    let with_query = match query {
        Some(q) if !q.is_empty() => format!("{}?{}", normalized, q),
        _ => normalized,
    };

    Url::parse(&with_query).map_err(|e| anyhow!("URL inválida '{}': {}", with_query, e))
}

pub fn get_domain(url: &Url) -> String {
    url.host_str().unwrap_or("").to_string()
}
