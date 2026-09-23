//! Port of packages/console/function/src/auth-redirect.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/function/src/auth-redirect.ts: only the
//! registered `app` client may redirect to an OpenCode callback host
//! (`opencode.ai`, `dev.opencode.ai`, `localhost:3000`, `127.0.0.1:3000`) on the
//! `/auth/callback` path; lookalike hosts and non-web schemes are rejected.

use opencode_console::auth_redirect::is_allowed_authorization_redirect;

#[test]
fn allows_registered_opencode_callbacks() {
    assert!(is_allowed_authorization_redirect(
        "app",
        "https://opencode.ai/auth/callback"
    ));
    assert!(is_allowed_authorization_redirect(
        "app",
        "https://dev.opencode.ai/auth/callback"
    ));
    assert!(is_allowed_authorization_redirect(
        "app",
        "http://localhost:3000/auth/callback"
    ));
    assert!(is_allowed_authorization_redirect(
        "app",
        "http://127.0.0.1:3000/auth/callback"
    ));
}

#[test]
fn rejects_unregistered_clients_and_external_redirects() {
    assert!(!is_allowed_authorization_redirect(
        "other",
        "https://opencode.ai/auth/callback"
    ));
    assert!(!is_allowed_authorization_redirect(
        "app",
        "https://evil.example/callback"
    ));
    assert!(!is_allowed_authorization_redirect(
        "app",
        "https://opencode.ai.evil.example/callback"
    ));
    assert!(!is_allowed_authorization_redirect(
        "app",
        "javascript:alert(1)"
    ));
}
