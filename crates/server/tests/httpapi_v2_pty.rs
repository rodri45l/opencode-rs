//! Port of packages/opencode/test/server/httpapi-v2-pty.test.ts (upstream 18ef3cc).
//!
//! Subset: the location-wrapped PTY CRUD surface and the connect-token CSRF
//! gate. The WebSocket echo and plugin shell-environment cases need a live PTY
//! bridge and are left to the PTY phase.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_schema::PtyId;
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-httpapi-v2-pty-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn req(method: &str, uri: &str) -> Request<Body> {
    with_directory(request(method, uri).body(Body::empty()).unwrap())
}

#[tokio::test]
#[ignore = "porting: v2 pty routes not implemented"]
async fn serves_location_wrapped_pty_routes_and_retains_exited_sessions() {
    let app = router(AppState::new());

    let empty = send(&app, req("GET", "/api/pty")).await;
    assert_eq!(empty.status(), StatusCode::OK);
    let empty = json(empty).await;
    assert_eq!(empty["data"], serde_json::json!([]));
    assert_eq!(empty["location"]["directory"], DIRECTORY);

    let created = send(
        &app,
        with_directory(json_body(
            request("POST", "/api/pty").body(Body::empty()).unwrap(),
            &serde_json::json!({
                "command": "/usr/bin/env",
                "args": ["sh", "-c", "exit 4"],
                "title": "v2",
            }),
        )),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let created = json(created).await;
    assert_eq!(created["location"]["directory"], DIRECTORY);
    assert_eq!(created["data"]["title"], "v2");
    let id = created["data"]["id"].as_str().expect("pty id").to_string();

    let found = send(&app, req("GET", &format!("/api/pty/{id}"))).await;
    assert_eq!(found.status(), StatusCode::OK);
    let found = json(found).await;
    assert_eq!(found["data"]["status"], "exited");
    assert_eq!(found["data"]["exitCode"], 4);

    let removed = send(&app, req("DELETE", &format!("/api/pty/{id}"))).await;
    assert_eq!(removed.status(), StatusCode::NO_CONTENT);

    let missing = send(&app, req("GET", &format!("/api/pty/{id}"))).await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        json(missing).await,
        serde_json::json!({
            "_tag": "PtyNotFoundError",
            "ptyID": id,
            "message": format!("PTY session not found: {id}"),
        })
    );
}

#[tokio::test]
#[ignore = "porting: v2 pty routes not implemented"]
async fn rejects_connect_tokens_without_the_csrf_header_and_connects_with_a_valid_ticket() {
    let app = router(AppState::new());
    let info = PtyId::generate();

    let forbidden = send(&app, req("POST", &format!("/api/pty/{info}/connect-token"))).await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        json(forbidden).await,
        serde_json::json!({ "_tag": "ForbiddenError" })
    );

    let token = send(
        &app,
        common::header(
            req("POST", &format!("/api/pty/{info}/connect-token")),
            "x-opencode-ticket",
            "1",
        ),
    )
    .await;
    assert_eq!(token.status(), StatusCode::OK);
    let ticket = json(token).await;
    assert!(ticket["data"]["ticket"].is_string());

    let invalid = send(
        &app,
        req(
            "GET",
            &format!("/api/pty/{info}/connect?ticket=not-a-ticket"),
        ),
    )
    .await;
    assert_eq!(invalid.status(), StatusCode::FORBIDDEN);
}
