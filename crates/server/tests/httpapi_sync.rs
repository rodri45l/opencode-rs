//! Port of packages/opencode/test/server/httpapi-sync.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the sync routes; see docs/TEST-PORT.md.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-sync-port";
const SESSION_ID: &str = "ses_sync";

fn post(uri: &str, body: &serde_json::Value) -> Request<Body> {
    let req = json_body(request("POST", uri).body(Body::empty()).unwrap(), body);
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn post_empty(uri: &str) -> Request<Body> {
    common::header(
        request("POST", uri).body(Body::empty()).unwrap(),
        "x-opencode-directory",
        DIRECTORY,
    )
}

#[tokio::test]
#[ignore = "porting: sync routes not implemented"]
async fn serves_sync_routes() {
    let app = router(AppState::new());

    let started = send(&app, post_empty("/sync/start")).await;
    assert_eq!(started.status(), StatusCode::OK);
    assert_eq!(json(started).await, serde_json::json!(true));

    let history = send(&app, post("/sync/history", &serde_json::json!({}))).await;
    assert_eq!(history.status(), StatusCode::OK);
    let rows = json(history).await;
    let rows = rows.as_array().expect("history rows");
    assert!(rows.iter().any(|row| row["aggregate_id"] == SESSION_ID));

    let events: Vec<serde_json::Value> = rows
        .iter()
        .filter(|row| row["aggregate_id"] == SESSION_ID)
        .map(|row| {
            serde_json::json!({
                "id": row["id"],
                "aggregateID": row["aggregate_id"],
                "seq": row["seq"],
                "type": row["type"],
                "data": row["data"],
            })
        })
        .collect();

    let replayed = send(
        &app,
        post(
            "/sync/replay",
            &serde_json::json!({ "directory": DIRECTORY, "events": events }),
        ),
    )
    .await;
    assert_eq!(replayed.status(), StatusCode::OK);
    assert_eq!(
        json(replayed).await,
        serde_json::json!({ "sessionID": SESSION_ID })
    );
}

#[tokio::test]
#[ignore = "porting: sync routes not implemented"]
async fn validates_seq_values() {
    let app = router(AppState::new());

    let cases = [
        serde_json::json!({ "aggregate": -1 }),
        serde_json::json!({ "aggregate": 1.5 }),
    ];
    for body in cases {
        let res = send(&app, post("/sync/history", &body)).await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST, "history {body}");
    }

    let replay_seqs = [-1.0_f64, 1.5, 0.0];
    for seq in replay_seqs {
        let body = serde_json::json!({
            "directory": DIRECTORY,
            "events": [{
                "id": "event",
                "aggregateID": "session",
                "seq": seq,
                "type": "session.created",
                "data": {},
            }],
        });
        let res = send(&app, post("/sync/replay", &body)).await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST, "replay {body}");
    }
}
