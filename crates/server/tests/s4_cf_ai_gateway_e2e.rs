//! Port of packages/opencode/test/provider/cf-ai-gateway-e2e.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/provider/provider.ts and src/provider/transform.ts;
//! see docs/TEST-PORT.md.
//!
//! Ported: the pure routing classification — `openai/*` rides the native OpenAI
//! passthrough on the Responses API, `anthropic/*` rides the native Anthropic
//! passthrough on the Messages API (dotted models.dev ids become dashed slugs),
//! and everything else stays on the unified `/compat` route.
//! Dropped: the end-to-end `generateText` cases that capture the Cloudflare
//! envelope body/headers through a stubbed fetch (reasoning effort on the wire,
//! token scoping regressions). Those need the ai-gateway-provider chain and a
//! network boundary; the projections they assert are covered by
//! `tests/s4_transform.rs`.

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

#[allow(dead_code)]
fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn gateway_route(api_id: &str) -> Result<Value, NotImplemented> {
    let (provider, endpoint, model) = if let Some(rest) = api_id.strip_prefix("openai/") {
        ("openai", "v1/responses", rest.to_string())
    } else if let Some(rest) = api_id.strip_prefix("anthropic/") {
        ("anthropic", "v1/messages", native_slug(rest)?)
    } else {
        ("compat", "chat/completions", api_id.to_string())
    };
    Ok(json!({
        "provider": provider,
        "endpoint": endpoint,
        "upstreamModel": model,
    }))
}

fn native_slug(api_id: &str) -> Result<String, NotImplemented> {
    Ok(api_id.replace('.', "-"))
}

#[test]
fn openai_rides_the_native_openai_passthrough_on_the_responses_api() {
    let route = gateway_route("openai/gpt-5.4").unwrap();
    assert_eq!(route["provider"], json!("openai"));
    assert_eq!(route["endpoint"], json!("v1/responses"));
    assert_eq!(route["upstreamModel"], json!("gpt-5.4"));
}

#[test]
fn anthropic_rides_the_native_anthropic_passthrough_on_the_messages_api() {
    let route = gateway_route("anthropic/claude-sonnet-4-6").unwrap();
    assert_eq!(route["provider"], json!("anthropic"));
    assert_eq!(route["endpoint"], json!("v1/messages"));
    assert_eq!(route["upstreamModel"], json!("claude-sonnet-4-6"));
}

#[test]
fn anthropic_dotted_models_dev_id_reaches_anthropic_as_a_dashed_native_slug() {
    let route = gateway_route("anthropic/claude-haiku-4.5").unwrap();
    assert_eq!(route["provider"], json!("anthropic"));
    assert_eq!(route["endpoint"], json!("v1/messages"));
    assert_eq!(route["upstreamModel"], json!("claude-haiku-4-5"));
    assert_eq!(native_slug("claude-haiku-4.5").unwrap(), "claude-haiku-4-5");
}

#[test]
fn workers_ai_models_stay_on_the_unified_compat_route() {
    let route = gateway_route("workers-ai/@cf/moonshotai/kimi-k2.6").unwrap();
    assert_eq!(route["provider"], json!("compat"));
    assert_eq!(route["endpoint"], json!("chat/completions"));
    assert_eq!(
        route["upstreamModel"],
        json!("workers-ai/@cf/moonshotai/kimi-k2.6")
    );
}
