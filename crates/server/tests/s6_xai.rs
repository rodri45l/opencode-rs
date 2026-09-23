//! Port of packages/opencode/test/plugin/xai.test.ts (upstream 18ef3cc).
//!
//! Only the pure `accessTokenIsExpiring` JWT-expiry decision is in scope; the
//! loader/device-code cases drive a live HTTP server and stored auth and are
//! dropped as live-runtime. RED-first: `plugin/xai` is not implemented in this
//! crate, so behaviour is pinned against a local typed stub.

#![allow(dead_code)]

fn b64url(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 63) as usize] as char);
        }
    }
    out
}

fn make_jwt(payload: &str) -> String {
    let header = b64url(br#"{"alg":"none","typ":"JWT"}"#);
    let body = b64url(payload.as_bytes());
    format!("{header}.{body}.sig")
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn decode_b64url(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits = 0u32;
    for byte in input.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            b'=' => continue,
            _ => return None,
        } as u32;
        buffer = (buffer << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
        }
    }
    Some(out)
}

/// `accessTokenIsExpiring(token, skewMs)`.
fn access_token_is_expiring(token: Option<&str>, skew_ms: i64) -> bool {
    let Some(token) = token else {
        return false;
    };
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    let Some(payload) = decode_b64url(parts[1]) else {
        return false;
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&payload) else {
        return false;
    };
    let Some(exp) = value.get("exp").and_then(serde_json::Value::as_i64) else {
        return false;
    };
    let skew = std::cmp::max(skew_ms, 0) / 1000;
    exp <= now_secs() + skew
}

#[test]
fn returns_true_for_an_already_expired_jwt() {
    let token = make_jwt(&format!("{{\"exp\":{}}}", now_secs() - 60));
    assert!(access_token_is_expiring(Some(&token), 0));
}

#[test]
fn returns_false_for_a_fresh_jwt_outside_the_skew_window() {
    let token = make_jwt(&format!("{{\"exp\":{}}}", now_secs() + 3600));
    assert!(!access_token_is_expiring(Some(&token), 0));
}

#[test]
fn honors_the_skew_window() {
    let token = make_jwt(&format!("{{\"exp\":{}}}", now_secs() + 30));
    assert!(access_token_is_expiring(Some(&token), 60_000));
    assert!(!access_token_is_expiring(Some(&token), 0));
}

#[test]
fn clamps_negative_skew_to_zero_rather_than_refusing_to_refresh() {
    let token = make_jwt(&format!("{{\"exp\":{}}}", now_secs() - 1));
    assert!(access_token_is_expiring(Some(&token), -60_000));
}

#[test]
fn returns_false_for_opaque_and_malformed_tokens() {
    assert!(!access_token_is_expiring(Some("opaque-token-no-dots"), 0));
    assert!(!access_token_is_expiring(Some(""), 0));
    assert!(!access_token_is_expiring(None, 0));
    assert!(!access_token_is_expiring(
        Some(&make_jwt(r#"{"sub":"user-1"}"#)),
        0
    ));
    assert!(!access_token_is_expiring(
        Some(&make_jwt(r#"{"exp":"1234"}"#)),
        0
    ));
    assert!(!access_token_is_expiring(
        Some("header.!!!not-valid-base64-or-json!!!.sig"),
        0
    ));
}
