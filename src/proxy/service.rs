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

pub async fn home() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub async fn handle(State(state): State<Arc<AppState>>, request: Request) -> Response {
    let path = request.uri().path().to_string();
    let query = request.uri().query().map(|s| s.to_string());
    let method = request.method().clone();
    let request_headers = request.headers().clone();

    let body = axum::body::to_bytes(request.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap_or_default();

    let target_url = match utils::extract_target(&path, query.as_deref()) {
        Ok(url) => url,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Html(format!(
                    "<h1>URL inválida</h1><p>{}</p><a href='/'>← Voltar</a>",
                    e
                )),
            )
                .into_response();
        }
    };

    let domain = utils::get_domain(&target_url);
    let url_str = target_url.to_string();

    if blocker::is_blocked(&domain, &state.blocked) {
        logger::log(&url_str, "bloqueado");
        let page = BLOCKED_HTML.replace("{{domain}}", &domain);
        return Html(page).into_response();
    }

    let response =
        match fetcher::fetch(&state.client, method, &target_url, request_headers, body).await {
            Ok(r) => r,
            Err(e) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Html(format!(
                        "<h1>Erro ao buscar conteúdo</h1><p>{}</p><a href='/'>← Voltar</a>",
                        e
                    )),
                )
                    .into_response();
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
            return (
                StatusCode::BAD_GATEWAY,
                Html(format!("<h1>Erro ao ler resposta</h1><p>{}</p>", e)),
            )
                .into_response();
        }
    };

    if content_type.contains("text/html") {
        let html = String::from_utf8_lossy(&body_bytes).into_owned();
        let (filtered_html, was_filtered) = filter::apply(&html, &state.words);

        let acao = if was_filtered {
            "filtrado"
        } else {
            "permitido"
        };
        logger::log(&url_str, acao);

        let mut builder = Response::builder().status(status);
        for (name, value) in forward_headers(&origin_headers) {
            if name != "content-type" {
                builder = builder.header(name, value);
            }
        }
        return builder
            .header("content-type", "text/html; charset=utf-8")
            .body(Body::from(filtered_html))
            .unwrap()
            .into_response();
    }

    logger::log(&url_str, "permitido");

    let mut builder = Response::builder().status(status);
    for (name, value) in forward_headers(&origin_headers) {
        builder = builder.header(name, value);
    }
    builder
        .body(Body::from(body_bytes))
        .unwrap()
        .into_response()
}
