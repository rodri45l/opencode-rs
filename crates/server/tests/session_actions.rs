//! Port of packages/opencode/test/server/session-actions.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the session create/update/get/fork/abort routes; see docs/TEST-PORT.md.
//!
//! The reference sets up an Effect instance store; the Rust port drives the
//! same routes through the router.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-session-actions-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn post_json(uri: &str, value: &serde_json::Value) -> Request<Body> {
    with_directory(json_body(
        request("POST", uri).body(Body::empty()).unwrap(),
        value,
    ))
}

fn patch_json(uri: &str, value: &serde_json::Value) -> Request<Body> {
    with_directory(json_body(
        request("PATCH", uri).body(Body::empty()).unwrap(),
        value,
    ))
}

#[tokio::test]
#[ignore = "porting: session routes not implemented"]
async fn session_routes_expose_metadata_on_create_update_get_and_fork() {
    let app = router(AppState::new());

    let created = send(
        &app,
        post_json(
            "/session",
            &serde_json::json!({
                "title": "meta-session",
                "metadata": { "source": "sdk", "trace": { "id": "abc" } },
            }),
        ),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let session = json(created).await;
    assert_eq!(
        session["metadata"],
        serde_json::json!({ "source": "sdk", "trace": { "id": "abc" } })
    );
    let session_id = session["id"].as_str().expect("id").to_string();

    let updated = send(
        &app,
        patch_json(
            &format!("/session/{session_id}"),
            &serde_json::json!({
                "metadata": { "source": "sdk", "trace": { "id": "def" }, "tags": ["one"] },
            }),
        ),
    )
    .await;
    assert_eq!(updated.status(), StatusCode::OK);
    let next = json(updated).await;
    assert_eq!(
        next["metadata"],
        serde_json::json!({ "source": "sdk", "trace": { "id": "def" }, "tags": ["one"] })
    );

    let fetched = send(
        &app,
        with_directory(
            request("GET", &format!("/session/{session_id}"))
                .body(Body::empty())
                .unwrap(),
        ),
    )
    .await;
    assert_eq!(fetched.status(), StatusCode::OK);
    assert_eq!(json(fetched).await["metadata"], next["metadata"]);

    let forked = send(
        &app,
        post_json(
            &format!("/session/{session_id}/fork"),
            &serde_json::json!({}),
        ),
    )
    .await;
    assert_eq!(forked.status(), StatusCode::OK);
    assert_eq!(json(forked).await["metadata"], next["metadata"]);

    let reset = send(
        &app,
        patch_json(
            &format!("/session/{session_id}"),
            &serde_json::json!({ "metadata": {} }),
        ),
    )
    .await;
    assert_eq!(reset.status(), StatusCode::OK);
    assert_eq!(json(reset).await["metadata"], serde_json::json!({}));
}

#[tokio::test]
#[ignore = "porting: session routes not implemented"]
async fn abort_route_returns_success() {
    let app = router(AppState::new());
    let res = send(
        &app,
        with_directory(
            request("POST", "/session/ses_abort/abort")
                .body(Body::empty())
                .unwrap(),
        ),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(json(res).await, serde_json::json!(true));
}

#[tokio::test]
#[ignore = "porting: experimental session routes not implemented"]
async fn experimental_background_route_is_a_noop_without_synchronous_subagents() {
    let app = router(AppState::new());
    let res = send(
        &app,
        with_directory(
            request("POST", "/experimental/session/ses_bg/background")
                .body(Body::empty())
                .unwrap(),
        ),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(json(res).await, serde_json::json!(false));
}
