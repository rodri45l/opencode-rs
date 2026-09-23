//! Server auth helpers (port of packages/app/src/utils/server.ts).

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

#[derive(Clone, Debug, PartialEq)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

pub fn auth_from_token(token: &str) -> Option<Credentials> {
    let bytes = URL_SAFE_NO_PAD.decode(token).ok()?;
    let decoded = String::from_utf8(bytes).ok()?;
    let separator = decoded.find(':')?;
    let username = &decoded[..separator];
    let password = &decoded[separator + 1..];
    Some(Credentials {
        username: if username.is_empty() {
            "opencode".to_string()
        } else {
            username.to_string()
        },
        password: password.to_string(),
    })
}

pub fn auth_token_from_credentials(credentials: &Credentials) -> String {
    URL_SAFE_NO_PAD.encode(format!("{}:{}", credentials.username, credentials.password))
}
