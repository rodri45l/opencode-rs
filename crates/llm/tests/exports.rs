//! Port of packages/llm/test/exports.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the public barrel surface.

use opencode_llm::{protocols, providers, Protocol, Provider, Route};
use serde_json::json;

#[test]
fn root_exposes_app_facing_runtime_apis() {
    let _ = opencode_llm::LLM::request(json!({}));
    let _ = opencode_llm::LLMClient::service();
    let _ = opencode_llm::LLMClient::layer();
    assert_eq!(
        Provider::make(json!({ "id": "fixture" })),
        json!({ "id": "fixture" })
    );
}

#[test]
fn route_barrel_exposes_route_authoring_apis() {
    assert_eq!(
        Route::make(json!({ "id": "fixture" })),
        json!({ "id": "fixture" })
    );
    assert_eq!(
        Protocol::make(json!({ "id": "fixture" })),
        json!({ "id": "fixture" })
    );
}

#[test]
fn protocol_barrels_expose_supported_low_level_routes() {
    assert_eq!(protocols::openai_chat::id(), "openai-chat");
    assert_eq!(
        protocols::openai_compatible_chat::id(),
        "openai-compatible-chat"
    );
    assert_eq!(protocols::openai_responses::id(), "openai-responses");
    assert_eq!(
        protocols::openai_responses_web_socket_route()["id"],
        "openai-responses-websocket"
    );
    assert_eq!(protocols::anthropic_messages::id(), "anthropic-messages");
}

#[test]
fn provider_barrels_expose_user_facing_facades() {
    assert_eq!(
        providers::openai::responses("gpt-5.5")["route"]["id"],
        "openai-responses"
    );
    assert_eq!(
        providers::openai::responses_web_socket("gpt-4.1-mini")["route"]["id"],
        "openai-responses-websocket"
    );
    assert_eq!(
        providers::openai_compatible::deepseek().model("deepseek-chat")["route"]["id"],
        "openai-compatible-chat"
    );
    assert_eq!(
        providers::cloudflare::ai_gateway(json!({ "accountId": "a", "gatewayApiKey": "b" }))
            .model("m")["route"]["id"],
        "cloudflare-ai-gateway"
    );
    assert_eq!(
        providers::cloudflare::workers_ai(json!({ "accountId": "a", "apiKey": "b" })).model("m")
            ["route"]["id"],
        "cloudflare-workers-ai"
    );
    assert_eq!(
        providers::openrouter::model("openai/gpt-4o-mini")["route"]["id"],
        "openrouter"
    );
    assert_eq!(
        providers::xai::responses("grok-4.3")["route"]["id"],
        "openai-responses"
    );
    assert_eq!(
        providers::xai::configure(json!({ "apiKey": "fixture" })).chat("grok-4.3")["route"]["id"],
        "openai-compatible-chat"
    );
    assert_eq!(
        providers::github_copilot::configure(json!({ "baseURL": "https://api.githubcopilot.test", "apiKey": "fixture", "endpoint": "responses" }))
            .model("mai-code-1-flash-picker")["route"]["id"],
        "openai-responses"
    );
    assert_eq!(
        providers::github_copilot::configure(json!({ "baseURL": "https://api.githubcopilot.test", "apiKey": "fixture", "endpoint": "chat" }))
            .model("gpt-5")["route"]["id"],
        "openai-chat"
    );
}
