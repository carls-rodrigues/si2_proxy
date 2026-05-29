use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
};
use std::sync::Arc;

use super::{AppState, blocker, fetcher, filter, logger, utils};

const BLOCKED_HTML: &str = include_str!("../../templates/blocked.html");
const INDEX_HTML: &str = include_str!("../../templates/index.html");

/// Maximum request body we buffer before forwarding to the origin (10 MiB).
const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

// Response headers that must not be forwarded back to the client.
const HOP_BY_HOP: &[&str] = &[
    "transfer-encoding",
    "content-encoding",
    "content-length",
    "connection",
    "keep-alive",
    "upgrade",
    "te",
    "trailer",
    "proxy-authenticate",
    "proxy-authorization",
];

fn forward_headers(src: &HeaderMap) -> Vec<(String, HeaderValue)> {
    src.iter()
        .filter(|(name, _)| !HOP_BY_HOP.contains(&name.as_str()))
        .map(|(name, value)| (name.to_string(), value.clone()))
        .collect()
}

/// Inject a `<base href="...">` so the browser resolves the page's relative and
/// root-relative assets (CSS, fonts, images, scripts) against the real origin
/// instead of against the proxy's own address. Without this, links like
/// `/static/app.css` would be requested from `http://localhost:5000/static/app.css`
/// and fail to parse as a target URL — leaving the page unstyled.
fn inject_base(html: &str, target_url: &str) -> String {
    let tag = format!("<base href=\"{}\">", escape_html(target_url));

    // Insert right after the opening <head ...> tag, case-insensitively.
    let lower = html.to_lowercase();
    if let Some(start) = lower.find("<head") {
        if let Some(rel_end) = lower[start..].find('>') {
            let insert_at = start + rel_end + 1;
            let mut out = String::with_capacity(html.len() + tag.len());
            out.push_str(&html[..insert_at]);
            out.push_str(&tag);
            out.push_str(&html[insert_at..]);
            return out;
        }
    }

    // No <head> found: prepend the tag so assets still resolve.
    format!("{tag}{html}")
}

/// Escape text destined for HTML so reflected user input cannot inject markup.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Render a simple error page with the given status, escaping the dynamic detail.
fn error_page(status: StatusCode, title: &str, detail: &str) -> Response {
    let html = format!(
        "<h1>{}</h1><p>{}</p><a href='/'>← Voltar</a>",
        escape_html(title),
        escape_html(detail)
    );
    (status, Html(html)).into_response()
}

/// Build a proxied response, forwarding origin headers (minus hop-by-hop ones).
/// When `content_type` is set it overrides any origin `content-type` header.
fn build_response(
    status: StatusCode,
    origin_headers: &HeaderMap,
    body: Body,
    content_type: Option<&str>,
) -> Response {
    let mut builder = Response::builder().status(status);
    for (name, value) in forward_headers(origin_headers) {
        if content_type.is_some() && name == "content-type" {
            continue;
        }
        builder = builder.header(name, value);
    }
    if let Some(ct) = content_type {
        builder = builder.header("content-type", ct);
    }
    builder.body(body).unwrap().into_response()
}

pub async fn home() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub async fn handle(State(state): State<Arc<AppState>>, request: Request) -> Response {
    let path = request.uri().path().to_string();
    let query = request.uri().query().map(|s| s.to_string());
    let method = request.method().clone();
    let request_headers = request.headers().clone();

    let body = axum::body::to_bytes(request.into_body(), MAX_BODY_BYTES)
        .await
        .unwrap_or_default();

    let target_url = match utils::extract_target(&path, query.as_deref()) {
        Ok(url) => url,
        Err(e) => return error_page(StatusCode::BAD_REQUEST, "URL inválida", &e.to_string()),
    };

    let domain = utils::get_domain(&target_url);
    let url_str = target_url.to_string();

    if blocker::is_blocked(&domain, &state.blocked) {
        logger::log(&url_str, "bloqueado");
        let page = BLOCKED_HTML.replace("{{domain}}", &escape_html(&domain));
        return Html(page).into_response();
    }

    let response =
        match fetcher::fetch(&state.client, method, &target_url, request_headers, body).await {
            Ok(r) => r,
            Err(e) => {
                return error_page(
                    StatusCode::BAD_GATEWAY,
                    "Erro ao buscar conteúdo",
                    &e.to_string(),
                );
            }
        };

    let status = StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::OK);
    let origin_headers = response.headers().clone();
    let content_type = origin_headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let body_bytes = match response.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return error_page(StatusCode::BAD_GATEWAY, "Erro ao ler resposta", &e.to_string());
        }
    };

    if content_type.contains("text/html") {
        let html = String::from_utf8_lossy(&body_bytes).into_owned();
        let (filtered_html, was_filtered) = filter::apply(&html, &state.words);

        let acao = if was_filtered { "filtrado" } else { "permitido" };
        logger::log(&url_str, acao);

        let final_html = inject_base(&filtered_html, &url_str);

        return build_response(
            status,
            &origin_headers,
            Body::from(final_html),
            Some("text/html; charset=utf-8"),
        );
    }

    logger::log(&url_str, "permitido");
    build_response(status, &origin_headers, Body::from(body_bytes), None)
}
