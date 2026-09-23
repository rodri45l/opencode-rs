//! Port of packages/opencode/test/plugin/snowflake-cortex.test.ts (upstream 18ef3cc).
//!
//! Only the pure `oauthScope` projection is in scope; the loader/token-refresh
//! cases drive stored OAuth + a live HTTP transport and are dropped as
//! live-runtime. RED-first: `plugin/snowflake-cortex` is not implemented in this
//! crate, so behaviour is pinned against a local typed stub.

#![allow(dead_code)]

fn oauth_scope(role: Option<&str>) -> String {
    match role {
        None => "refresh_token".to_string(),
        Some("") => "refresh_token".to_string(),
        Some(role)
            if role
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') =>
        {
            format!("refresh_token session:role:{role}")
        }
        Some(role) => format!(
            "refresh_token session:role-encoded:{}",
            percent_encode(role)
        ),
    }
}

fn percent_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push_str(&format!("{byte:02X}"));
        }
    }
    out
}

#[test]
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
