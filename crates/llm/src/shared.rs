//! Shared protocol helpers (token arithmetic, JSON encoding, media limits).

use serde_json::Value;

/// Helpers shared by every protocol adapter.
pub struct ProviderShared;

impl ProviderShared {
    /// Largest base64-encoded media payload accepted before rejecting input.
    pub const MAX_MEDIA_ENCODED_BYTES: usize = 28 * 1024 * 1024;

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

    /// Provider total when supplied, otherwise `input + output` when either is known.
    pub fn total_tokens(
        input: Option<i64>,
        output: Option<i64>,
        total: Option<i64>,
    ) -> Option<i64> {
        if total.is_some() {
            return total;
        }
        if input.is_none() && output.is_none() {
            return None;
        }
        Some(input.unwrap_or(0) + output.unwrap_or(0))
    }

    /// Compact JSON encoding used for tool payloads and wire bodies.
    pub fn encode_json(value: &Value) -> String {
        serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
    }

    /// Join the `text` field of a list of parts with newlines.
    pub fn join_text(parts: &[Value]) -> String {
        parts
            .iter()
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// XML-escape a chronological system update's text.
    pub fn escape_system_update_text(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    /// Wrap chronological system-update text in the lower-authority user wrapper.
    pub fn wrap_system_update(parts: &[Value]) -> String {
        format!(
            "<system-update>\n{}\n</system-update>",
            Self::escape_system_update_text(&Self::join_text(parts))
        )
    }

    /// Read an optional i64 field.
    pub fn int(value: &Value, key: &str) -> Option<i64> {
        value.get(key).and_then(Value::as_i64)
    }

    /// Whether `value` is a JSON object (not array/null).
    pub fn is_record(value: &Value) -> bool {
        value.is_object()
    }

    /// Render a tool-result part's textual projection.
    pub fn tool_result_text(part: &Value) -> String {
        let result = part.get("result").cloned().unwrap_or(Value::Null);
        let kind = result.get("type").and_then(Value::as_str).unwrap_or("json");
        let value = result.get("value").cloned().unwrap_or(Value::Null);
        match kind {
            "text" => value
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| value.to_string()),
            "error" => match &value {
                Value::String(s) => s.clone(),
                other => Self::encode_json(other),
            },
            _ => Self::encode_json(&value),
        }
    }
}
