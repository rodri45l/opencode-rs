//! Port of packages/console/app/test/serverAction.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/app/src/lib/server-action.ts: a server
//! action referer is preserved when it is same-origin, replaced with the request
//! origin when unsafe or absent, and requests to other routes are untouched.
//! Re-derived: the request/Response pair is represented as a sanitized value with
//! a `changed` flag instead of object identity.

#[allow(dead_code)]
mod server_action {
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: console server action sanitization not implemented";

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SanitizedRequest {
        pub url: String,
        pub referer: String,
        pub changed: bool,
    }

    pub fn sanitize_server_action_request(
        _url: &str,
        _referer: Option<&str>,
    ) -> PortResult<SanitizedRequest> {
        Err(NotImplemented(NOTE))
    }
}

use server_action::{sanitize_server_action_request, NOTE};

#[test]
#[ignore = "porting: console server action sanitization not implemented"]
fn preserves_same_origin_return_locations() {
    let request = sanitize_server_action_request(
        "https://dev.opencode.ai/_server?id=action",
        Some("https://dev.opencode.ai/auth?next=%2Fconsole"),
    )
    .expect(NOTE);

    assert!(!request.changed);
    assert_eq!(
        request.referer,
        "https://dev.opencode.ai/auth?next=%2Fconsole"
    );
}

#[test]
#[ignore = "porting: console server action sanitization not implemented"]
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
                .expect(NOTE)
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
#[ignore = "porting: console server action sanitization not implemented"]
fn does_not_change_other_routes() {
    let request = sanitize_server_action_request(
        "https://dev.opencode.ai/auth",
        Some("https://evil.example/phishing-login"),
    )
    .expect(NOTE);

    assert!(!request.changed);
    assert_eq!(request.referer, "https://evil.example/phishing-login");
}
