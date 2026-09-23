//! Port of packages/app/src/utils/server.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use opencode_test_support as ts;

#[derive(Clone, Debug, PartialEq)]
struct Credentials {
    username: String,
    password: String,
}

// Local stubs (fast wave): real module lands later.
fn auth_from_token(_token: &str) -> Option<Credentials> {
    None
}

fn auth_token_from_credentials(_credentials: &Credentials) -> String {
    String::new()
}

#[test]
#[ignore = "porting: utils/server not implemented"]
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
#[ignore = "porting: utils/server not implemented"]
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
#[ignore = "porting: utils/server not implemented"]
fn ignores_malformed_tokens() {
    assert_eq!(auth_from_token("not base64"), None);
    assert_eq!(
        auth_from_token(&ts::encode_b64url(b"missing-separator")),
        None
    );
}

#[test]
#[ignore = "porting: utils/server not implemented"]
fn encodes_credentials_with_the_default_username() {
    assert_eq!(
        auth_token_from_credentials(&Credentials {
            username: "opencode".into(),
            password: "secret".into()
        }),
        ts::encode_b64url(b"opencode:secret")
    );
}
