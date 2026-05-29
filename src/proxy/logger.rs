use chrono::Utc;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Serialize)]
struct LogEntry<'a> {
    timestamp: String,
    url: &'a str,
    acao: &'a str,
}

pub fn log(url: &str, acao: &str) {
    fn write(url: &str, acao: &str) -> Option<()> {
        let line = serde_json::to_string(&LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            url,
            acao,
        })
        .ok()?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/log.json")
            .ok()?;
        writeln!(file, "{}", line).ok()
    }
    write(url, acao);
}
