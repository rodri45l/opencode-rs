use axum::body::Body;
use axum::http::{Request, StatusCode};
use futures::StreamExt;
use opencode_core::EventBus;
use opencode_schema::{EventEnvelope, EventType};
use opencode_server::{router, AppState};
use std::time::Duration;
use tower::ServiceExt;

#[tokio::test]
async fn health_and_legacy_event_routes_exist() {
    let app = router(AppState::new());
    let res = app
        .oneshot(
            Request::builder()
                .uri("/event")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn event_stream_emits_published_events() {
    let state = AppState::new();
    let app = router(state.clone());

    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/event")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let content_type = res.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(
        content_type.starts_with("text/event-stream"),
        "unexpected content-type: {content_type}"
    );

    // The handler subscribed before returning, so a later publish is delivered.
    state.bus.publish(EventEnvelope::new(
        EventType::SessionCreated,
        serde_json::json!({ "sessionID": "ses_test" }),
    ));

    let mut body = res.into_body().into_data_stream();
    let mut text = String::new();
    for _ in 0..10 {
        let chunk = tokio::time::timeout(Duration::from_secs(2), body.next())
            .await
            .expect("timed out waiting for an event")
            .expect("stream ended")
            .expect("body error");
        text = String::from_utf8_lossy(&chunk).into_owned();
        if text.contains("session.created") {
            break;
        }
    }

    assert!(text.contains("event: session.created"), "chunk was: {text}");
    assert!(
        text.contains("\"type\":\"session.created\""),
        "chunk was: {text}"
    );
    assert!(text.contains("data: "), "chunk was: {text}");
}
