//! Port of packages/app/src/utils/server.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::server_auth::{auth_from_token, auth_token_from_credentials, Credentials};
use opencode_test_support as ts;

#[test]
fn decodes_basic_auth_credentials_from_auth_token() {
    let token = ts::encode_b64url(b"kit:secret");
    assert_eq!(
        auth_from_token(&token),
        Some(Credentials {
            username: "kit".into(),
            password: "secret".into()
        })
    );
}

#[test]
fn defaults_blank_username_to_opencode() {
    let token = ts::encode_b64url(b":secret");
    assert_eq!(
        auth_from_token(&token),
        Some(Credentials {
            username: "opencode".into(),
            password: "secret".into()
        })
    );
}

#[test]
fn ignores_malformed_tokens() {
    assert_eq!(auth_from_token("not base64"), None);
    assert_eq!(
        auth_from_token(&ts::encode_b64url(b"missing-separator")),
        None
    );
}

#[test]
fn encodes_credentials_with_the_default_username() {
    assert_eq!(
        auth_token_from_credentials(&Credentials {
            username: "opencode".into(),
            password: "secret".into()
        }),
        ts::encode_b64url(b"opencode:secret")
    );
}
