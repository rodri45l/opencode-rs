//! Data URL decoding.
//!
//! Ports the observable behaviour of `packages/opencode/src/util/data-url.ts`.

use base64::Engine;

/// Decode a `data:` URL into its UTF-8 body, or the empty string when malformed.
pub fn decode_data_url(url: &str) -> String {
    let Some(index) = url.find(',') else {
        return String::new();
    };
    let head = &url[..index];
    let body = &url[index + 1..];
    if head.contains(";base64") {
        return base64::engine::general_purpose::STANDARD
            .decode(body)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_default();
    }
    percent_decode(body)
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = &input[index + 1..index + 3];
            if let Ok(value) = u8::from_str_radix(hex, 16) {
                out.push(value);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
