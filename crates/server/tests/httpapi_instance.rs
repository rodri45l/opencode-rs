//! Port of packages/opencode/test/server/httpapi-instance.test.ts (upstream 18ef3cc).
//!
//! Subset: the OpenAPI document, malformed permission/question ids, typed
//! not-found permission/question bodies, and typed not-found project bodies.
//! The fence-header and VCS/path cases from the reference file depend on the
//! experimental-workspace flag and a live git worktree; those are covered by
//! the workspace phase instead.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_schema::{PermissionId, QuestionId};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-instance-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn post_json(uri: &str, value: &serde_json::Value) -> Request<Body> {
    with_directory(json_body(
        request("POST", uri).body(Body::empty()).unwrap(),
        value,
    ))
}

#[tokio::test]
#[ignore = "porting: OpenAPI document route not implemented"]
async fn serves_the_openapi_document() {
    let app = router(AppState::new());
    let res = send(
        &app,
        with_directory(request("GET", "/doc").body(Body::empty()).unwrap()),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers()["content-type"]
        .to_str()
        .unwrap_or_default()
        .contains("application/json"));
    let body = json(res).await;
    assert!(body["openapi"].is_string());
    assert!(body["info"].is_object());
    assert!(body["paths"]["/global/health"].is_object());
    assert!(body["paths"]["/session"].is_object());
}

#[tokio::test]
#[ignore = "porting: permission/question routes not implemented"]
async fn rejects_malformed_permission_and_question_request_ids() {
    let app = router(AppState::new());

    let permission = send(
        &app,
        post_json(
            "/permission/invalid-permission-id/reply",
            &serde_json::json!({ "reply": "once" }),
        ),
    )
    .await;
    assert_eq!(permission.status(), StatusCode::BAD_REQUEST);

    let question_reply = send(
        &app,
        post_json(
            "/question/invalid-question-id/reply",
            &serde_json::json!({ "answers": [["Yes"]] }),
        ),
    )
    .await;
    assert_eq!(question_reply.status(), StatusCode::BAD_REQUEST);

    let question_reject = send(
        &app,
        post_json(
            "/question/invalid-question-id/reject",
            &serde_json::json!({}),
        ),
    )
    .await;
    assert_eq!(question_reject.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "porting: permission/question routes not implemented"]
async fn returns_typed_not_found_bodies_for_missing_permission_and_question_requests() {
    let app = router(AppState::new());
    let permission_id = PermissionId::generate();
    let question_reply_id = QuestionId::generate();
    let question_reject_id = QuestionId::generate();

    let permission = send(
        &app,
        post_json(
            &format!("/permission/{permission_id}/reply"),
            &serde_json::json!({ "reply": "once" }),
        ),
    )
    .await;
    assert_eq!(permission.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        json(permission).await,
        serde_json::json!({
            "_tag": "PermissionNotFoundError",
            "requestID": permission_id.to_string(),
            "message": format!("Permission request not found: {permission_id}"),
        })
    );

    let question_reply = send(
        &app,
        post_json(
            &format!("/question/{question_reply_id}/reply"),
            &serde_json::json!({ "answers": [["Yes"]] }),
        ),
    )
    .await;
    assert_eq!(question_reply.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        json(question_reply).await,
        serde_json::json!({
            "_tag": "QuestionNotFoundError",
            "requestID": question_reply_id.to_string(),
            "message": format!("Question request not found: {question_reply_id}"),
        })
    );

    let question_reject = send(
        &app,
        post_json(
            &format!("/question/{question_reject_id}/reject"),
            &serde_json::json!({}),
        ),
    )
    .await;
    assert_eq!(question_reject.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        json(question_reject).await,
        serde_json::json!({
            "_tag": "QuestionNotFoundError",
            "requestID": question_reject_id.to_string(),
            "message": format!("Question request not found: {question_reject_id}"),
        })
    );
}

#[tokio::test]
#[ignore = "porting: project routes not implemented"]
async fn returns_typed_not_found_bodies_for_missing_projects() {
    let app = router(AppState::new());
    let req = with_directory(json_body(
        request("PATCH", "/project/project_missing")
            .body(Body::empty())
            .unwrap(),
        &serde_json::json!({ "name": "Missing" }),
    ));

    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        json(res).await,
        serde_json::json!({
            "_tag": "ProjectNotFoundError",
            "projectID": "project_missing",
            "message": "Project not found: project_missing",
        })
    );
}
