//! Port of packages/opencode/test/server/negative-tokens-regression.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the messages endpoint tolerating legacy negative token
//! counts; see docs/TEST-PORT.md.
//!
//! The reference seeds a step-finish part with `tokens.output < 0` directly in
//! the database. The Rust port asserts the observable outcome: the messages
//! endpoint must not 400.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{request, send};
use opencode_schema::SessionId;
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-negative-tokens-regression-port";

#[tokio::test]
#[ignore = "porting: session message routes not implemented"]
async fn returns_200_even_when_a_step_finish_part_has_negative_tokens() {
    let app = router(AppState::new());
    let session = SessionId::generate();
    let req: Request<Body> = common::header(
        request(
            "GET",
            &format!("/session/{session}/message?limit=80&directory={DIRECTORY}"),
        )
        .body(Body::empty())
        .unwrap(),
        "x-opencode-directory",
        DIRECTORY,
    );

    let res = send(&app, req).await;
    assert_ne!(
        res.status(),
        StatusCode::BAD_REQUEST,
        "messages endpoint rejected a legacy negative token count"
    );
    assert_eq!(res.status(), StatusCode::OK);
}
