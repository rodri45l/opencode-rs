//! Port of packages/llm/test/provider/openrouter.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the OpenRouter provider facade.

use opencode_llm::{providers, LLM, LLMClient};
use serde_json::json;

#[test]
#[ignore = "porting: openrouter provider not implemented"]
fn prepares_openrouter_models_through_the_openai_compatible_chat_route() {
    let model = providers::openrouter::configure(json!({ "apiKey": "test-key" })).model("openai/gpt-4o-mini");

    assert_eq!(model["id"], "openai/gpt-4o-mini");
    assert_eq!(model["provider"], "openrouter");
    assert_eq!(model["route"]["id"], "openrouter");
    assert_eq!(model["endpoint"]["baseURL"], "https://openrouter.ai/api/v1");

    let prepared = LLMClient::prepare(LLM::request(json!({ "model": model, "prompt": "Say hello." }))).expect("prepare");
    assert_eq!(prepared.route, "openrouter");
    assert_eq!(prepared.body["model"], "openai/gpt-4o-mini");
    assert_eq!(prepared.body["messages"], json!([{ "role": "user", "content": "Say hello." }]));
    assert_eq!(prepared.body["stream"], true);
}

#[test]
#[ignore = "porting: openrouter provider not implemented"]
fn applies_openrouter_payload_options_from_the_model_helper() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": providers::openrouter::configure(json!({
            "apiKey": "test-key",
            "providerOptions": { "openrouter": { "usage": true, "reasoning": { "effort": "high" }, "promptCacheKey": "session_123" } }
        })).model("anthropic/claude-3.7-sonnet:thinking"),
        "prompt": "Think briefly.",
    })))
    .expect("prepare");

    assert_eq!(prepared.body["usage"], json!({ "include": true }));
    assert_eq!(prepared.body["reasoning"], json!({ "effort": "high" }));
    assert_eq!(prepared.body["prompt_cache_key"], "session_123");
}
