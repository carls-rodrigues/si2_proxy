use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

pub fn load(path: &str) -> Result<HashMap<String, String>> {
    let content = std::fs::read_to_string(path)?;
    let words: HashMap<String, String> = serde_json::from_str(&content)?;
    Ok(words)
}

/// Replaces all blacklisted words in `html` (case-insensitive).
/// Returns the modified string and whether any replacement was made.
pub fn apply(html: &str, words: &HashMap<String, String>) -> (String, bool) {
    let mut result = html.to_string();
    let mut filtered = false;

    for (word, replacement) in words {
        let pattern = format!("(?i){}", regex::escape(word));
        if let Ok(re) = Regex::new(&pattern) {
            let new = re.replace_all(&result, replacement.as_str()).into_owned();
            if new != result {
                filtered = true;
                result = new;
            }
        }
    }

    (result, filtered)
}
