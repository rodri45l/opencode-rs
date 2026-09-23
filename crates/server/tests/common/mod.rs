#![allow(dead_code)]

//! Shared black-box helpers for the ported `packages/opencode/test/server`
//! integration tests. Each test drives the real axum router in-process.

use axum::body::Body;
use axum::http::{HeaderName, HeaderValue, Request, Response};
use axum::Router;
use futures::StreamExt;
use std::time::Duration;
use tower::ServiceExt;

/// Send a request through the router (cloned per call).
pub async fn send(app: &Router, req: Request<Body>) -> Response<Body> {
    app.clone().oneshot(req).await.expect("router call")
}

/// Start building a request with the given method and path.
pub fn request(method: &str, uri: &str) -> axum::http::request::Builder {
    Request::builder().method(method).uri(uri)
}

/// Attach a header to a request builder.
pub fn header(mut req: Request<Body>, name: &str, value: &str) -> Request<Body> {
    req.headers_mut().insert(
        HeaderName::from_bytes(name.as_bytes()).expect("header name"),
        HeaderValue::from_str(value).expect("header value"),
    );
    req
}

/// Attach a JSON body to a request builder.
pub fn json_body(req: Request<Body>, value: &serde_json::Value) -> Request<Body> {
    let body = serde_json::to_vec(value).expect("json body");
    let mut req = req;
    req.headers_mut().insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/json"),
    );
    *req.body_mut() = Body::from(body);
    req
}

/// Read a response body as bytes.
pub async fn bytes(res: Response<Body>) -> Vec<u8> {
    axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .expect("response body")
        .to_vec()
}

/// Read a response body as JSON.
pub async fn json(res: Response<Body>) -> serde_json::Value {
    serde_json::from_slice(&bytes(res).await).expect("json response")
}

/// Read a response body as UTF-8 text.
pub async fn text(res: Response<Body>) -> String {
    String::from_utf8_lossy(&bytes(res).await).into_owned()
}

/// Read the next `data:` payload from an SSE byte stream as JSON.
///
/// Frames are separated by a blank line; `data:` lines are joined with `\n`.
/// Returns `None` on timeout or end of stream.
pub async fn next_sse_data<S>(
    stream: &mut S,
    buffer: &mut String,
    timeout: Duration,
) -> Option<serde_json::Value>
where
    S: futures::Stream<Item = Result<axum::body::Bytes, axum::Error>> + Unpin,
{
    loop {
        if let Some(index) = buffer.find("\n\n") {
            let frame: String = buffer.drain(..index + 2).collect();
            let data = frame
                .lines()
                .filter_map(|line| line.strip_prefix("data:"))
                .map(|line| line.trim_start())
                .collect::<Vec<_>>()
                .join("\n");
            if !data.is_empty() {
                if let Ok(value) = serde_json::from_str(&data) {
                    return Some(value);
                }
            }
            continue;
        }
        let chunk = tokio::time::timeout(timeout, stream.next())
            .await
            .ok()??
            .ok()?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
    }
}

/// Encode a `Basic` authorization header value for the given credentials.
pub fn basic(username: &str, password: &str) -> String {
    format!("Basic {}", base64(&format!("{username}:{password}")))
}

/// Standard base64 encoding (RFC 4648, padded).
pub fn base64(input: &str) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}
