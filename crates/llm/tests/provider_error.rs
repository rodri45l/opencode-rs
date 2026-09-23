//! Port of packages/llm/test/provider-error.test.ts (upstream 18ef3cc).
//! Behaviour pinned by `isContextOverflow`.

use opencode_llm::LLM;

#[test]
fn classifies_provider_token_limit_messages_as_context_overflow() {
    let messages = [
        "tokens in request more than max tokens allowed",
        "{\"error\":{\"type\":\"request_too_large\",\"message\":\"Request exceeds the maximum size\"}}",
        "Requested token count exceeds the model's maximum context length of 131072 tokens.",
        "Input length (265330) exceeds model's maximum context length (262144).",
        "Input length 131393 exceeds the maximum allowed input length of 131040 tokens.",
        "The input (516368 tokens) is longer than the model's context length (262144 tokens).",
        "Prompt has 5,958,968 tokens, but the configured context size is 256,000 tokens",
        "Too many tokens",
        "Token limit exceeded",
    ];

    for message in messages {
        assert!(
            LLM::is_context_overflow(message).expect("classify"),
            "expected overflow: {message}"
        );
    }
}

#[test]
fn does_not_classify_rate_limits_as_context_overflow() {
    let messages = [
        "Throttling error: Too many tokens, please wait before trying again.",
        "Rate limit exceeded, please retry after 30 seconds.",
        "Too many requests. Please slow down.",
    ];

    for message in messages {
        assert!(
            !LLM::is_context_overflow(message).expect("classify"),
            "expected not overflow: {message}"
        );
    }
}
