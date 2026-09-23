//! Port of packages/console/app/test/serverAction.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/app/src/lib/server-action.ts: a server
//! action referer is preserved when it is same-origin, replaced with the request
//! origin when unsafe or absent, and requests to other routes are untouched.
//! Re-derived: the request/Response pair is represented as a sanitized value with
//! a `changed` flag instead of object identity.

use opencode_console::server_action::sanitize_server_action_request;

#[test]
fn preserves_same_origin_return_locations() {
    let request = sanitize_server_action_request(
        "https://dev.opencode.ai/_server?id=action",
        Some("https://dev.opencode.ai/auth?next=%2Fconsole"),
    );

    assert!(!request.changed);
    assert_eq!(
        request.referer,
        "https://dev.opencode.ai/auth?next=%2Fconsole"
    );
}

#[test]
fn replaces_unsafe_return_locations_with_the_request_origin() {
    let referers = [
        Some("https://evil.example/phishing-login"),
        Some("not a url"),
        None,
    ];

    let origins: Vec<String> = referers
        .iter()
        .map(|referer| {
            sanitize_server_action_request("https://dev.opencode.ai/_server?id=action", *referer)
                .referer
        })
        .collect();

    assert_eq!(
        origins,
        vec![
            "https://dev.opencode.ai".to_string(),
            "https://dev.opencode.ai".to_string(),
            "https://dev.opencode.ai".to_string(),
        ]
    );
}

#[test]
fn does_not_change_other_routes() {
    let request = sanitize_server_action_request(
        "https://dev.opencode.ai/auth",
        Some("https://evil.example/phishing-login"),
    );

    assert!(!request.changed);
    assert_eq!(request.referer, "https://evil.example/phishing-login");
}
