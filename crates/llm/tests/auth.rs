//! Port of packages/llm/test/auth.test.ts (upstream 18ef3cc).
//! Behaviour pinned by `Auth.apply` header rendering.

use opencode_llm::Auth;
use serde_json::json;

fn input(env: serde_json::Value) -> serde_json::Value {
    json!({ "headers": { "x-existing": "yes" }, "env": env })
}

#[test]
fn renders_a_config_credential_as_bearer_auth() {
    let headers = Auth::apply(
        Auth::config_bearer("OPENAI_API_KEY"),
        input(json!({ "OPENAI_API_KEY": "sk-test" })),
    )
    .expect("auth apply");

    assert_eq!(headers["authorization"], "Bearer sk-test");
    assert_eq!(headers["x-existing"], "yes");
}

#[test]
fn falls_back_between_credential_sources_before_rendering() {
    let auth = Auth::render_header(
        Auth::or_else(Auth::config("PRIMARY_KEY"), Auth::value("fallback-key")),
        "x-api-key",
    );
    let headers = Auth::apply(auth, input(json!({}))).expect("auth apply");

    assert_eq!(headers["x-api-key"], "fallback-key");
    assert_eq!(headers["x-existing"], "yes");
}

#[test]
fn composes_header_auth_in_sequence() {
    let auth = Auth::and_then(
        Auth::headers(json!({ "x-tenant-id": "tenant-1" })),
        Auth::bearer("gateway-token"),
    );
    let headers = Auth::apply(auth, input(json!({}))).expect("auth apply");

    assert_eq!(headers["x-tenant-id"], "tenant-1");
    assert_eq!(headers["authorization"], "Bearer gateway-token");
    assert_eq!(headers["x-existing"], "yes");
}

#[test]
fn renders_a_direct_secret_as_a_custom_header() {
    let headers =
        Auth::apply(Auth::header("api-key", "direct-key"), input(json!({}))).expect("auth apply");

    assert_eq!(headers["api-key"], "direct-key");
    assert_eq!(headers["x-existing"], "yes");
}

#[test]
fn renders_bearer_auth_into_a_custom_header() {
    let headers = Auth::apply(
        Auth::bearer_header("cf-aig-authorization", "gateway-token"),
        input(json!({})),
    )
    .expect("auth apply");

    assert_eq!(headers["cf-aig-authorization"], "Bearer gateway-token");
    assert_eq!(headers["x-existing"], "yes");
}

#[test]
fn falls_back_between_full_auth_values() {
    let auth = Auth::or_else(
        Auth::config_bearer("OPENAI_API_KEY"),
        Auth::headers(json!({ "authorization": "Bearer supplied" })),
    );
    let headers = Auth::apply(auth, input(json!({}))).expect("auth apply");

    assert_eq!(headers["authorization"], "Bearer supplied");
    assert_eq!(headers["x-existing"], "yes");
}

#[test]
fn can_intentionally_leave_auth_untouched() {
    let headers = Auth::apply(Auth::none(), input(json!({}))).expect("auth apply");

    assert!(headers.get("authorization").is_none());
    assert_eq!(headers["x-existing"], "yes");
}
