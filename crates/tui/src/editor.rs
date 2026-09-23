//! External editor prompt normalization.
//!
//! Port of packages/tui/src/editor.ts `normalizePromptContent` (upstream 18ef3cc).
//! Launching an external editor is process/runtime behaviour and is
//! human-verified.

/// Normalize a single trailing editor newline for one-line prompts.
pub fn normalize_prompt_content(content: &str) -> String {
    if let Some(body) = content.strip_suffix("\r\n") {
        return if !body.contains('\n') && !body.contains('\r') {
            body.to_string()
        } else {
            content.to_string()
        };
    }
    if let Some(body) = content.strip_suffix('\n') {
        return if !body.contains('\n') && !body.contains('\r') {
            body.to_string()
        } else {
            content.to_string()
        };
    }
    content.to_string()
}
