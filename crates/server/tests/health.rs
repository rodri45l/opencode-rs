use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use opencode_server::{router, AppState};
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_ok() {
    let app = router(AppState::new());
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body, serde_json::json!({ "healthy": true }));
}
