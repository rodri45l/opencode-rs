//! Shared protocol helpers (token arithmetic, JSON encoding, media limits).

use serde_json::Value;

/// Helpers shared by every protocol adapter.
pub struct ProviderShared;

impl ProviderShared {
    /// Largest base64-encoded media payload accepted before rejecting input.
    pub const MAX_MEDIA_ENCODED_BYTES: usize = 5 * 1024 * 1024;

    /// `a - b`, clamped at zero; `None` when `a` is absent.
    pub fn subtract_tokens(a: Option<i64>, b: Option<i64>) -> Option<i64> {
        match (a, b) {
            (None, _) => None,
            (Some(a), None) => Some(a),
            (Some(a), Some(b)) => Some((a - b).max(0)),
        }
    }

    /// Sum of every present value; `None` when all inputs are absent.
    pub fn sum_tokens(values: &[Option<i64>]) -> Option<i64> {
        let present: Vec<i64> = values.iter().filter_map(|v| *v).collect();
        if present.is_empty() {
            None
        } else {
            Some(present.iter().sum())
        }
    }

    /// Compact JSON encoding used for tool payloads and wire bodies.
    pub fn encode_json(value: &Value) -> String {
        serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
    }
}
