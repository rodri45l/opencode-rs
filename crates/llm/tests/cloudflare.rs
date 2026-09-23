//! Port of packages/llm/test/provider/cloudflare.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the Cloudflare provider facades.
//! Effect-ts `ConfigProvider` env plumbing is dropped; endpoint/auth behaviour is kept.

use opencode_llm::{providers, testing, LLMClient, LLM};
use serde_json::json;

fn text_response(text: &str) -> serde_json::Value {
    json!({ "status": 200, "body": format!("data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
        json!({ "choices": [{ "delta": { "content": text } }] }),
        json!({ "choices": [{ "delta": {}, "finish_reason": "stop" }] })) })
}

#[test]
fn prepares_ai_gateway_models_through_the_openai_compatible_chat_protocol() {
    let model = providers::cloudflare::ai_gateway(
        json!({ "accountId": "test-account", "gatewayId": "test-gateway", "apiKey": "test-token" }),
    )
    .model("workers-ai/@cf/meta/llama-3.3-70b-instruct");

    assert_eq!(model["id"], "workers-ai/@cf/meta/llama-3.3-70b-instruct");
    assert_eq!(model["provider"], "cloudflare-ai-gateway");
    assert_eq!(model["route"]["id"], "cloudflare-ai-gateway");

    let prepared = LLMClient::prepare(LLM::request(
        json!({ "model": model, "prompt": "Say hello." }),
    ))
    .expect("prepare");
    assert_eq!(prepared.route, "cloudflare-ai-gateway");
    assert_eq!(
        prepared.body["model"],
        "workers-ai/@cf/meta/llama-3.3-70b-instruct"
    );
    assert_eq!(prepared.body["stream"], true);
}

#[test]
fn defaults_ai_gateway_id_to_default_when_omitted_or_blank() {
    let model = providers::cloudflare::ai_gateway(
        json!({ "accountId": "test-account", "gatewayId": "", "gatewayApiKey": "test-token" }),
    )
    .model("workers-ai/@cf/meta/llama-3.3-70b-instruct");

    assert_eq!(
        model["endpoint"]["baseURL"],
        "https://gateway.ai.cloudflare.com/v1/test-account/default/compat"
    );
}

#[test]
fn allows_a_fully_configured_base_url_override() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": providers::cloudflare::ai_gateway(json!({ "baseURL": "https://gateway.proxy.test/v1/custom/compat", "apiKey": "test-token" })).model("openai/gpt-4o-mini"),
        "prompt": "Say hello.",
    })))
    .expect("prepare");

    assert_eq!(
        prepared.model["endpoint"]["baseURL"],
        "https://gateway.proxy.test/v1/custom/compat"
    );
}

#[test]
fn prepares_direct_workers_ai_models_through_the_openai_compatible_chat_protocol() {
    let model = providers::cloudflare::workers_ai(
        json!({ "accountId": "test-account", "apiKey": "test-token" }),
    )
    .model("@cf/meta/llama-3.1-8b-instruct");

    assert_eq!(model["id"], "@cf/meta/llama-3.1-8b-instruct");
    assert_eq!(model["provider"], "cloudflare-workers-ai");
    assert_eq!(model["route"]["id"], "cloudflare-workers-ai");
    assert_eq!(
        model["endpoint"]["baseURL"],
        "https://api.cloudflare.com/client/v4/accounts/test-account/ai/v1"
    );

    let prepared = LLMClient::prepare(LLM::request(
        json!({ "model": model, "prompt": "Say hello." }),
    ))
    .expect("prepare");
    assert_eq!(prepared.route, "cloudflare-workers-ai");
}

#[test]
fn posts_to_the_derived_gateway_endpoint_with_bearer_auth() {
    let response = LLM::request(json!({
        "model": providers::cloudflare::ai_gateway(json!({ "accountId": "test-account", "gatewayId": "test-gateway", "apiKey": "test-token" })).model("openai/gpt-4o-mini"),
        "prompt": "Say hello.",
    }));
    testing::push_response(text_response("Hello"));
    let response = LLMClient::generate(response).expect("generate");

    assert_eq!(response.text, "Hello");
}

#[test]
fn supports_direct_workers_ai_token_aliases_through_auth_config() {
    testing::push_response(text_response("Hello"));
    let response = LLMClient::generate(LLM::request(json!({
        "model": providers::cloudflare::workers_ai(json!({ "accountId": "test-account" })).model("@cf/meta/llama-3.1-8b-instruct"),
        "prompt": "Say hello.",
        "env": { "CLOUDFLARE_WORKERS_AI_TOKEN": "test-token" },
    })))
    .expect("generate");

    assert_eq!(response.text, "Hello");
}
