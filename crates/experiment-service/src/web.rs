//! Explicit embedded local assets; no path-to-filesystem translation.
use axum::{body::Body, http::header, response::Response};

pub(super) fn asset(path: &str, method: &str) -> Option<Response> {
    let (mime, text) = match path {
        "/" => (
            "text/html; charset=utf-8",
            include_str!("../web/index.html"),
        ),
        "/assets/dashboard.mjs" => (
            "text/javascript; charset=utf-8",
            include_str!("../web/dashboard.mjs"),
        ),
        "/assets/data.mjs" => (
            "text/javascript; charset=utf-8",
            include_str!("../web/data.mjs"),
        ),
        "/assets/parser-worker.mjs" => (
            "text/javascript; charset=utf-8",
            include_str!("../web/parser-worker.mjs"),
        ),
        "/assets/dashboard.css" => (
            "text/css; charset=utf-8",
            include_str!("../web/dashboard.css"),
        ),
        _ => return None,
    };
    let mut response = Response::new(Body::from(if method == "GET" { text } else { "" }));
    *response.status_mut() = if method == "GET" {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::METHOD_NOT_ALLOWED
    };
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, mime.parse().unwrap());
    headers.insert(
        header::CONTENT_LENGTH,
        (if method == "GET" { text.len() } else { 0 })
            .to_string()
            .parse()
            .unwrap(),
    );
    headers.insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'none'; script-src 'self'; style-src 'self'; worker-src 'self'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"
            .parse()
            .unwrap(),
    );
    if method != "GET" {
        headers.insert(header::ALLOW, "GET".parse().unwrap());
    }
    Some(response)
}
