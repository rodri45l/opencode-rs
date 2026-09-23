//! Port of packages/llm/test/endpoint.test.ts (upstream 18ef3cc).
//! Behaviour pinned by `Endpoint.render`.

use opencode_llm::Endpoint;
use serde_json::json;

fn request() -> serde_json::Value {
    json!({
        "model": { "id": "model-1", "provider": "test", "route": { "id": "openai-chat" } },
        "prompt": "hello",
    })
}

#[test]
fn appends_a_static_path_to_the_models_base_url() {
    let url = Endpoint::render(
        Endpoint::path(
            "/chat",
            json!({ "baseURL": "https://api.example.test/v1/" }),
        ),
        json!({ "request": request(), "body": {} }),
    )
    .expect("endpoint render");

    assert_eq!(url, "https://api.example.test/v1/chat");
}

#[test]
fn endpoint_query_params_are_appended_to_the_rendered_url() {
    let url = Endpoint::render(
        Endpoint::path(
            "/chat?alt=sse",
            json!({
                "baseURL": "https://custom.example.test/root/",
                "query": { "api-version": "2026-01-01", "alt": "json" }
            }),
        ),
        json!({ "request": request(), "body": {} }),
    )
    .expect("endpoint render");

    assert_eq!(
        url,
        "https://custom.example.test/root/chat?alt=json&api-version=2026-01-01"
    );
}

#[test]
fn path_may_be_a_function_of_the_validated_body() {
    let url = Endpoint::render(
        Endpoint::path(
            "/model/us.amazon.nova-micro-v1%3A0/converse-stream",
            json!({ "baseURL": "https://bedrock-runtime.us-east-1.amazonaws.com" }),
        ),
        json!({ "request": request(), "body": { "modelId": "us.amazon.nova-micro-v1:0" } }),
    )
    .expect("endpoint render");

    assert_eq!(
        url,
        "https://bedrock-runtime.us-east-1.amazonaws.com/model/us.amazon.nova-micro-v1%3A0/converse-stream"
    );
}
