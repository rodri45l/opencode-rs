//! Port of packages/console/function/src/auth-redirect.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/function/src/auth-redirect.ts: only the
//! registered `app` client may redirect to an OpenCode callback host
//! (`opencode.ai`, `dev.opencode.ai`, `localhost:3000`, `127.0.0.1:3000`) on the
//! `/auth/callback` path; lookalike hosts and non-web schemes are rejected.

#[allow(dead_code)]
mod auth_redirect {
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

    pub const NOTE: &str = "porting: console authorization redirect validation not implemented";

    pub fn is_allowed_authorization_redirect(_client: &str, _url: &str) -> PortResult<bool> {
        Err(NotImplemented(NOTE))
    }
}

use auth_redirect::{is_allowed_authorization_redirect, NOTE};

#[test]
#[ignore = "porting: console authorization redirect validation not implemented"]
fn allows_registered_opencode_callbacks() {
    assert!(
        is_allowed_authorization_redirect("app", "https://opencode.ai/auth/callback").expect(NOTE)
    );
    assert!(
        is_allowed_authorization_redirect("app", "https://dev.opencode.ai/auth/callback")
            .expect(NOTE)
    );
    assert!(
        is_allowed_authorization_redirect("app", "http://localhost:3000/auth/callback")
            .expect(NOTE)
    );
    assert!(
        is_allowed_authorization_redirect("app", "http://127.0.0.1:3000/auth/callback")
            .expect(NOTE)
    );
}

#[test]
#[ignore = "porting: console authorization redirect validation not implemented"]
fn rejects_unregistered_clients_and_external_redirects() {
    assert!(
        !is_allowed_authorization_redirect("other", "https://opencode.ai/auth/callback")
            .expect(NOTE)
    );
    assert!(
        !is_allowed_authorization_redirect("app", "https://evil.example/callback").expect(NOTE)
    );
    assert!(
        !is_allowed_authorization_redirect("app", "https://opencode.ai.evil.example/callback")
            .expect(NOTE)
    );
    assert!(!is_allowed_authorization_redirect("app", "javascript:alert(1)").expect(NOTE));
}
