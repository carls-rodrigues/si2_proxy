use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// A compiled, case-insensitive word filter: the pattern to match and its replacement.
pub type WordFilter = Vec<(Regex, String)>;

pub fn load(path: &str) -> Result<WordFilter> {
    let content = std::fs::read_to_string(path)?;
    let words: HashMap<String, String> = serde_json::from_str(&content)?;

    let mut filters = Vec::with_capacity(words.len());
    for (word, replacement) in words {
        let re = Regex::new(&format!("(?i){}", regex::escape(&word)))?; // ?i case-insensitive
        filters.push((re, replacement));
    }
    Ok(filters)
}

pub fn apply(html: &str, words: &WordFilter) -> (String, bool) {
    let mut result = html.to_string();
    let mut filtered = false;

    for (re, replacement) in words {
        let new = re.replace_all(&result, replacement.as_str()).into_owned();
        if new != result {
            filtered = true;
            result = new;
        }
    }

    (result, filtered)
}
