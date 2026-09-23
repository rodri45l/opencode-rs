//! Port of packages/opencode/test/plugin/snowflake-cortex.test.ts (upstream 18ef3cc).
//!
//! Only the pure `oauthScope` projection is in scope; the loader/token-refresh
//! cases drive stored OAuth + a live HTTP transport and are dropped as
//! live-runtime. RED-first: `plugin/snowflake-cortex` is not implemented in this
//! crate, so behaviour is pinned against a local typed stub.

#![allow(dead_code)]

fn oauth_scope(_role: Option<&str>) -> String {
    String::new()
}

#[test]
#[ignore = "porting: plugin snowflake-cortex oauthScope not implemented"]
fn oauth_scope_uses_snowflake_compatible_scope_values() {
    assert_eq!(oauth_scope(None), "refresh_token");
    assert_eq!(
        oauth_scope(Some("PUBLIC")),
        "refresh_token session:role:PUBLIC"
    );
    assert_eq!(
        oauth_scope(Some("AUTH SNOWFLAKE")),
        "refresh_token session:role-encoded:AUTH%20SNOWFLAKE"
    );
}
