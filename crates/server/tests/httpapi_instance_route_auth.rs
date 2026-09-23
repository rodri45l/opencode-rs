//! Port of packages/opencode/test/server/httpapi-instance-route-auth.test.ts (upstream 18ef3cc).
//! Behaviour pinned by instance-route authorization; see docs/TEST-PORT.md.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{basic, header, request, send};
use opencode_server::{router_with_options, AppState, AuthConfig, ServerOptions};

fn authed_app() -> axum::Router {
    router_with_options(
        AppState::new(),
        ServerOptions::with_auth(AuthConfig::with_password("secret")),
    )
}

fn get(uri: &str, headers: &[(&str, &str)]) -> Request<Body> {
    let mut req = request("GET", uri).body(Body::empty()).unwrap();
    for (name, value) in headers {
        req = header(req, name, value);
    }
    req
}

#[tokio::test]
#[ignore = "porting: instance route authorization middleware not implemented"]
async fn requires_configured_auth_before_opening_the_instance_event_stream() {
    let app = authed_app();
    let directory = "/tmp/opencode-instance-route-auth";

    let missing = send(&app, get("/event", &[("x-opencode-directory", directory)])).await;
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);

    let authed = send(
        &app,
        get(
            "/event",
            &[
                ("x-opencode-directory", directory),
                ("authorization", &basic("opencode", "secret")),
            ],
        ),
    )
    .await;
    assert_eq!(authed.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "porting: instance route authorization middleware not implemented"]
async fn requires_configured_auth_before_resolving_the_pty_websocket_route() {
    let app = authed_app();
    let directory = "/tmp/opencode-instance-route-auth";
    let route = "/pty/pty_test/connect";

    let missing = send(&app, get(route, &[("x-opencode-directory", directory)])).await;
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);

    let authed = send(
        &app,
        get(
            route,
            &[
                ("x-opencode-directory", directory),
                ("authorization", &basic("opencode", "secret")),
            ],
        ),
    )
    .await;
    assert_eq!(authed.status(), StatusCode::NOT_FOUND);
}
