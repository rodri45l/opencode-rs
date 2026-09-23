//! Port of packages/opencode/test/server/httpapi-listen.test.ts (upstream 18ef3cc).
//!
//! Subset: the listener's HTTP surface — the authenticated PTY shells route and
//! the PTY ticket mint/connect safety checks — asserted through the real router.
//! The listener lifecycle cases (graceful/forced stop, port 0 preferring 4096,
//! websocket upgrade, plugin-client reuse) need `Server.listen` and stay red
//! until the listen phase lands.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, request, send};
use opencode_server::{router_with_options, AppState, AuthConfig, ServerOptions};

const DIRECTORY: &str = "/tmp/opencode-httpapi-listen-port";
const PASSWORD: &str = "listen-secret";

fn authed(method: &str, uri: &str) -> Request<Body> {
    let req = request(method, uri).body(Body::empty()).unwrap();
    let req = common::header(req, "x-opencode-directory", DIRECTORY);
    common::header(req, "authorization", &common::basic("opencode", PASSWORD))
}

fn app() -> axum::Router {
    router_with_options(
        AppState::new(),
        ServerOptions::with_auth(AuthConfig::with_credentials("opencode", PASSWORD)),
    )
}

#[tokio::test]
#[ignore = "porting: Server.listen and PTY routes not implemented"]
async fn serves_http_routes_through_the_listener() {
    let app = app();
    let res = send(&app, authed("GET", "/pty/shells")).await;

    assert_eq!(res.status(), StatusCode::OK);
    let body = json(res).await;
    assert!(body.as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["path"].is_string() && item["name"].is_string() && item["acceptable"].is_boolean()
        })
    }));
}

#[tokio::test]
#[ignore = "porting: Server.listen and PTY routes not implemented"]
async fn rejects_unsafe_pty_ticket_mint_and_connect_requests() {
    let app = app();

    // Minting without the ticket header is refused.
    let no_header = send(&app, authed("POST", "/pty/pty_1/connect-token")).await;
    assert_eq!(no_header.status(), StatusCode::FORBIDDEN);

    // A cross-origin mint is refused.
    let cross_origin = send(
        &app,
        common::header(
            authed("POST", "/pty/pty_1/connect-token"),
            "origin",
            "https://evil.example",
        ),
    )
    .await;
    assert_eq!(cross_origin.status(), StatusCode::FORBIDDEN);

    // Minting without a directory uses the server cwd and cannot find a
    // directory-scoped PTY.
    let ambiguous = send(
        &app,
        common::header(
            authed("POST", "/pty/pty_1/connect-token"),
            "x-opencode-ticket",
            "1",
        ),
    )
    .await;
    assert_eq!(ambiguous.status(), StatusCode::NOT_FOUND);

    // A directory-scoped mint succeeds and returns a ticket with a TTL.
    let scoped = send(
        &app,
        common::header(
            authed(
                "POST",
                "/pty/pty_1/connect-token?directory=%2Ftmp%2Fopencode-httpapi-listen-port",
            ),
            "x-opencode-ticket",
            "1",
        ),
    )
    .await;
    assert_eq!(scoped.status(), StatusCode::OK);
    let scoped = json(scoped).await;
    assert!(scoped["ticket"].is_string());
    assert!(scoped["expires_in"].as_u64().unwrap_or_default() > 0);
}

#[tokio::test]
#[ignore = "porting: Server.listen and PTY routes not implemented"]
async fn requires_auth_for_pty_routes() {
    let app = app();
    let res = send(
        &app,
        request("GET", "/pty/shells").body(Body::empty()).unwrap(),
    )
    .await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers()["www-authenticate"],
        "Basic realm=\"Secure Area\""
    );
}
