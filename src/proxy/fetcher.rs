use anyhow::Result;
use axum::http::{HeaderMap, Method};
use bytes::Bytes;
use url::Url;

// Headers that must not be forwarded to the origin server.
const HOP_BY_HOP: &[&str] = &[
    "host",            // reqwest sets this from the target URL
    "accept-encoding", // let reqwest negotiate compression it can actually decompress
    "transfer-encoding",
    "connection",
    "keep-alive",
    "upgrade",
    "te",
    "trailer",
    "proxy-authenticate",
    "proxy-authorization",
];

pub async fn fetch(
    client: &reqwest::Client,
    method: Method,
    url: &Url,
    req_headers: HeaderMap,
    body: Bytes,
) -> Result<reqwest::Response> {
    let mut headers = HeaderMap::new();
    for (name, value) in &req_headers {
        if !HOP_BY_HOP.contains(&name.as_str()) {
            headers.append(name, value.clone());
        }
    }

    let response = client
        .request(method, url.as_str())
        .headers(headers)
        .body(body)
        .send()
        .await?;

    Ok(response)
}
