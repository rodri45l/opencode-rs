//! Port of packages/opencode/test/server/httpapi-v2-location.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the v2 location contract; see docs/TEST-PORT.md.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, next_sse_data, request, send};
use opencode_server::{router, AppState};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct Event {
    #[allow(dead_code)]
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    location: Option<Location>,
    #[allow(dead_code)]
    #[serde(default)]
    data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct Location {
    directory: String,
    #[allow(dead_code)]
    #[serde(default)]
    project: Option<serde_json::Value>,
}

fn with_directory(req: Request<Body>, dir: &str) -> Request<Body> {
    common::header(req, "x-opencode-directory", dir)
}

#[test]
fn decodes_event_v2_location_refs_without_resolved_project_metadata() {
    let raw = r#"{
        "id": "evt_test",
        "type": "file.watcher.updated",
        "location": { "directory": "/tmp/project" },
        "data": {}
    }"#;
    let event: Event = serde_json::from_str(raw).expect("event decodes");
    assert_eq!(event.event_type, "file.watcher.updated");
    assert_eq!(event.location.expect("location").directory, "/tmp/project");
}

#[tokio::test]
#[ignore = "porting: v2 command/skill location routes not implemented"]
async fn returns_command_and_skill_snapshots_with_resolved_locations() {
    let dir = "/tmp/opencode-v2-location-port";
    let app = router(AppState::new());

    for route in ["/api/command", "/api/skill"] {
        let res = send(
            &app,
            with_directory(request("GET", route).body(Body::empty()).unwrap(), dir),
        )
        .await;
        assert_eq!(res.status(), StatusCode::OK, "route {route}");
        let body = json(res).await;
        assert!(
            body["data"].is_array(),
            "route {route}: data must be an array"
        );
        assert_eq!(body["location"]["directory"], dir, "route {route}");
        assert!(
            body["location"]["project"]["id"].is_string(),
            "route {route}"
        );
    }
}

#[tokio::test]
#[ignore = "porting: v2 event location streaming not implemented"]
async fn streams_native_event_v2_payloads_across_locations() {
    let subscriber = "/tmp/opencode-v2-location-subscriber";
    let publisher = "/tmp/opencode-v2-location-publisher";
    let app = router(AppState::new());

    let res = send(
        &app,
        with_directory(
            request("GET", "/api/event").body(Body::empty()).unwrap(),
            subscriber,
        ),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let mut stream = res.into_body().into_data_stream();
    let mut buffer = String::new();

    let connected = next_sse_data(&mut stream, &mut buffer, Duration::from_secs(5))
        .await
        .expect("timed out waiting for server.connected");
    assert_eq!(connected["type"], "server.connected");
    assert!(connected.get("location").is_none());

    let created = send(
        &app,
        with_directory(
            request("POST", "/session").body(Body::empty()).unwrap(),
            publisher,
        ),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);

    let mut found = None;
    for _ in 0..20 {
        let Some(event) = next_sse_data(&mut stream, &mut buffer, Duration::from_secs(5)).await
        else {
            break;
        };
        if event["type"] == "session.created" {
            found = Some(event);
            break;
        }
    }
    let event = found.expect("timed out waiting for session.created");
    assert_eq!(event["location"]["directory"], publisher);
    assert!(event["data"]["sessionID"].is_string());
}
