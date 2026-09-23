//! Provider error classification (context overflow detection).

/// Regex-free context-overflow patterns, mirroring the reference list.
const PATTERNS: &[&str] = &[
    "prompt is too long",
    "request_too_large",
    "input is too long for requested model",
    "exceeds the context window",
    "maximum context length",
    "input token count",
    "tokens in request more than max tokens allowed",
    "maximum prompt length is",
    "reduce the length of the messages",
    "maximum context length is",
    "maximum allowed input length of",
    "is longer than the model",
    "exceeds the limit of",
    "exceeds the available context size",
    "greater than the context length",
    "context window exceeds limit",
    "exceeded model token limit",
    "context_length_exceeded",
    "context length_exceeded",
    "request entity too large",
    "context length is only",
    "prompt too long",
    "too large for model with",
    "prompt has",
    "model_context_window_exceeded",
    "too many tokens",
    "token limit exceeded",
];

const EXCLUSIONS: &[&str] = &[
    "throttling error",
    "service unavailable",
    "rate limit",
    "too many requests",
];

fn contains(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// Whether a provider message describes a context-overflow failure.
pub fn is_context_overflow(message: &str) -> bool {
    if EXCLUSIONS.iter().any(|pattern| contains(message, pattern)) {
        return false;
    }
    if PATTERNS.iter().any(|pattern| contains(message, pattern)) {
        return true;
    }
    // Fallback: `4xx (no body)` style transport errors.
    let lower = message.to_lowercase();
    lower.starts_with("400") || lower.starts_with("413")
}
