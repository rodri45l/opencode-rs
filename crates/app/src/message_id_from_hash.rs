//! Session hash parsing (port of
//! packages/app/src/pages/session/message-id-from-hash.ts).

pub fn message_id_from_hash(hash: &str) -> Option<String> {
    let value = hash.strip_prefix('#').unwrap_or(hash);
    value
        .strip_prefix("message-")
        .map(|value| value.to_string())
}
