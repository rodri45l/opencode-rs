//! Port of packages/opencode/test/server/httpapi-event.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the SSE event stream; see docs/TEST-PORT.md.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{next_sse_data, request, send};
use opencode_server::{router, AppState};
use std::time::Duration;

fn event_stream() -> Request<Body> {
    request("GET", "/api/event").body(Body::empty()).unwrap()
}

#[tokio::test]
async fn serves_event_stream() {
    let app = router(AppState::new());
    let res = send(&app, event_stream()).await;

    assert_eq!(res.status(), StatusCode::OK);
    let headers = res.headers();
    assert!(
        headers["content-type"]
            .to_str()
            .unwrap_or_default()
            .contains("text/event-stream"),
        "unexpected content-type: {:?}",
        headers["content-type"]
    );
    assert_eq!(headers["cache-control"], "no-cache, no-transform");
    assert_eq!(headers["x-accel-buffering"], "no");
    assert_eq!(headers["x-content-type-options"], "nosniff");

    let mut stream = res.into_body().into_data_stream();
    let mut buffer = String::new();
    let event = next_sse_data(&mut stream, &mut buffer, Duration::from_secs(5))
        .await
        .expect("timed out waiting for server.connected");
    assert_eq!(event["type"], "server.connected");
    assert_eq!(event["properties"], serde_json::json!({}));
}

#[tokio::test]
async fn keeps_the_event_stream_open_after_the_initial_event() {
    let app = router(AppState::new());
    let res = send(&app, event_stream()).await;

    let mut stream = res.into_body().into_data_stream();
    let mut buffer = String::new();
    let first = next_sse_data(&mut stream, &mut buffer, Duration::from_secs(5))
        .await
        .expect("timed out waiting for server.connected");
    assert_eq!(first["type"], "server.connected");
    assert_eq!(first["properties"], serde_json::json!({}));

    // If no second event arrives within 250ms, the stream is still open.
    let second = next_sse_data(&mut stream, &mut buffer, Duration::from_millis(250)).await;
    assert!(second.is_none(), "unexpected second event: {second:?}");
}

#[tokio::test]
async fn delivers_instance_events_after_the_initial_event() {
    let app = router(AppState::new());
    let res = send(&app, event_stream()).await;

    let mut stream = res.into_body().into_data_stream();
    let mut buffer = String::new();
    let first = next_sse_data(&mut stream, &mut buffer, Duration::from_secs(5))
        .await
        .expect("timed out waiting for server.connected");
    assert_eq!(first["type"], "server.connected");

    let created = send(
        &app,
        common::header(
            request("POST", "/session").body(Body::empty()).unwrap(),
            "x-opencode-directory",
            "/tmp/opencode-event-port",
        ),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);

    let event = next_sse_data(&mut stream, &mut buffer, Duration::from_secs(5))
        .await
        .expect("timed out waiting for session.created");
    assert_eq!(event["type"], "session.created");
}
