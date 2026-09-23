//! Port of packages/llm/test/route.test.ts (upstream 18ef3cc).
//! Behaviour pinned by `Route.with`.

use opencode_llm::{Auth, Route};
use serde_json::json;

#[test]
fn merges_endpoint_query_and_header_defaults_while_replacing_auth_and_id() {
    let auth = Auth::headers(json!({ "x-auth": "new" }));
    let base = json!({
        "id": "base-chat",
        "endpoint": { "baseURL": "https://api.example.test/v1", "query": { "keep": "base", "base": "1" } },
        "headers": { "x-base": "base", "x-override": "base" },
        "auth": Auth::headers(json!({ "x-auth": "old" })),
    });
    let patched = json!({
        "id": "patched-chat",
        "endpoint": { "query": { "keep": "patch", "patch": "1" } },
        "headers": { "x-override": "patch", "x-patch": "patch" },
        "auth": auth,
    });

    let route = Route::with(base, patched).expect("route with");

    assert_eq!(route["id"], "patched-chat");
    assert_eq!(route["auth"], Auth::headers(json!({ "x-auth": "new" })));
    assert_eq!(route["endpoint"]["baseURL"], "https://api.example.test/v1");
    assert_eq!(
        route["endpoint"]["query"],
        json!({ "keep": "patch", "base": "1", "patch": "1" })
    );
    assert_eq!(
        route["defaults"]["headers"],
        json!({ "x-base": "base", "x-override": "patch", "x-patch": "patch" })
    );
}
