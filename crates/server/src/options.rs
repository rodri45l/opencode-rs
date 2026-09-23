//! Server construction options.
//!
//! `router` builds the default (unauthenticated) application. Tests that pin
//! auth and CORS behaviour build the router through [`router_with_options`] so
//! the configuration is explicit at the call site.

use crate::auth::AuthConfig;
use crate::state::AppState;
use axum::body::Body;
use axum::http::{HeaderValue, Method, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use base64::Engine;
use serde_json::json;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

/// Options that shape a server instance.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerOptions {
    /// Basic-auth configuration.
    pub auth: AuthConfig,
    /// Extra origins permitted by the CORS layer.
    pub cors: Vec<String>,
}

impl ServerOptions {
    /// Default options: auth disabled, no custom CORS origins.
    pub fn new() -> Self {
        Self::default()
    }

    /// Options with basic auth enabled.
    pub fn with_auth(auth: AuthConfig) -> Self {
        Self {
            auth,
            ..Self::default()
        }
    }

    /// Options with custom CORS origins.
    pub fn with_cors(origins: Vec<String>) -> Self {
        Self {
            cors: origins,
            ..Self::default()
        }
    }
}

/// Build the application router honouring `options`.
pub fn router_with_options(state: AppState, options: ServerOptions) -> Router {
    let auth = options.auth.clone();
    let cors = options.clone();
    let app = crate::routes::base_router()
        .layer(CompressionLayer::new())
        .layer(middleware::from_fn(head_compression_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(move |request, next| {
            let auth = auth.clone();
            async move { auth_middleware(request, next, auth).await }
        }))
        .layer(middleware::from_fn(move |request, next| {
            let cors = cors.clone();
            async move { cors_middleware(request, next, cors).await }
        }));
    app.with_state(state)
}

async fn head_compression_middleware(request: Request<Body>, next: Next) -> Response {
    let is_head = request.method() == Method::HEAD;
    let mut response = next.run(request).await;
    if is_head {
        response.headers_mut().remove("content-encoding");
        response.headers_mut().remove("content-length");
    }
    response
}

fn decode_basic(value: &str) -> Option<(String, String)> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(value)
        .ok()?;
    let decoded = String::from_utf8(decoded).ok()?;
    let (username, password) = decoded.split_once(':')?;
    Some((username.to_string(), password.to_string()))
}

fn authorized(request: &Request<Body>, auth: &AuthConfig) -> bool {
    if let Some(query) = request.uri().query() {
        for pair in query.split('&') {
            if let Some(value) = pair.strip_prefix("auth_token=") {
                return decode_basic(value)
                    .map(|(username, password)| auth.authorized(&username, &password))
                    .unwrap_or(false);
            }
        }
    }
    let Some(header) = request
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let Some(encoded) = header.strip_prefix("Basic ") else {
        return false;
    };
    decode_basic(encoded)
        .map(|(username, password)| auth.authorized(&username, &password))
        .unwrap_or(false)
}

async fn auth_middleware(request: Request<Body>, next: Next, auth: AuthConfig) -> Response {
    if request.method() == Method::OPTIONS || !auth.required() || authorized(&request, &auth) {
        return next.run(request).await;
    }
    let mut response = if request.uri().path().starts_with("/api/") {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "_tag": "UnauthorizedError", "message": "Authentication required" })),
        )
            .into_response()
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    };
    response.headers_mut().insert(
        "www-authenticate",
        HeaderValue::from_static("Basic realm=\"Secure Area\""),
    );
    response
}

async fn cors_middleware(request: Request<Body>, next: Next, options: ServerOptions) -> Response {
    let origin = request
        .headers()
        .get("origin")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let allowed = origin
        .as_deref()
        .map(|origin| options.cors.is_empty() || options.cors.iter().any(|item| item == origin))
        .unwrap_or(false);
    let allow_origin = if allowed {
        origin.clone().unwrap_or_else(|| "*".to_string())
    } else {
        "null".to_string()
    };
    let is_preflight = request.method() == Method::OPTIONS
        && request
            .headers()
            .contains_key("access-control-request-method");
    if is_preflight {
        let requested_headers = request
            .headers()
            .get("access-control-request-headers")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("authorization")
            .to_string();
        let mut response = StatusCode::NO_CONTENT.into_response();
        let headers = response.headers_mut();
        if let Ok(value) = HeaderValue::from_str(&allow_origin) {
            headers.insert("access-control-allow-origin", value);
        }
        headers.insert(
            "access-control-allow-methods",
            HeaderValue::from_static("GET, POST, PUT, PATCH, DELETE, OPTIONS"),
        );
        if let Ok(value) = HeaderValue::from_str(&requested_headers) {
            headers.insert("access-control-allow-headers", value);
        }
        headers.insert(
            "vary",
            HeaderValue::from_static(
                "origin, access-control-request-method, access-control-request-headers",
            ),
        );
        return response;
    }
    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&allow_origin) {
        response
            .headers_mut()
            .insert("access-control-allow-origin", value);
    }
    response
        .headers_mut()
        .append("vary", HeaderValue::from_static("origin"));
    response
}
