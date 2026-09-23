//! Port of packages/opencode/test/server/httpapi-file.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the filesystem routes; see docs/TEST-PORT.md.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, request, send};
use opencode_server::{router, AppState};

fn directory() -> String {
    let dir = std::env::temp_dir().join("opencode-file-port");
    std::fs::create_dir_all(&dir).expect("temp dir");
    std::fs::write(dir.join("hello.txt"), "hello").expect("write fixture");
    dir.to_string_lossy().into_owned()
}

fn get(uri: &str, dir: &str) -> Request<Body> {
    common::header(
        request("GET", uri).body(Body::empty()).unwrap(),
        "x-opencode-directory",
        dir,
    )
}

#[tokio::test]
#[ignore = "porting: filesystem routes not implemented"]
async fn serves_read_endpoints() {
    let dir = directory();
    let app = router(AppState::new());

    let list = send(&app, get("/file?path=.", &dir)).await;
    assert_eq!(list.status(), StatusCode::OK);
    let list = json(list).await;
    assert!(list.as_array().expect("array").iter().any(|item| {
        item["name"] == "hello.txt" && item["path"] == "hello.txt" && item["type"] == "file"
    }));

    let content = send(&app, get("/file/content?path=hello.txt", &dir)).await;
    assert_eq!(content.status(), StatusCode::OK);
    let content = json(content).await;
    assert_eq!(content["type"], "text");
    assert_eq!(content["content"], "hello");

    let status = send(&app, get("/file/status", &dir)).await;
    assert_eq!(status.status(), StatusCode::OK);
    assert_eq!(json(status).await, serde_json::json!([]));
}

#[tokio::test]
#[ignore = "porting: filesystem routes not implemented"]
async fn serves_search_endpoints() {
    let dir = directory();
    std::fs::write(std::path::Path::new(&dir).join("hello.txt"), "needle").expect("write fixture");
    let app = router(AppState::new());

    let text = send(&app, get("/find?pattern=needle", &dir)).await;
    assert_eq!(text.status(), StatusCode::OK);
    let text = json(text).await;
    assert!(text
        .as_array()
        .expect("array")
        .iter()
        .any(|item| item["line_number"] == 1));

    let files = send(&app, get("/find/file?query=hello&type=file", &dir)).await;
    assert_eq!(files.status(), StatusCode::OK);
    assert!(json(files).await.to_string().contains("hello.txt"));

    let symbols = send(&app, get("/find/symbol?query=hello", &dir)).await;
    assert_eq!(symbols.status(), StatusCode::OK);
    assert_eq!(json(symbols).await, serde_json::json!([]));
}
