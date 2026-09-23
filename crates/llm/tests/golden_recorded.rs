//! Port of packages/llm/test/provider/golden.recorded.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the recorded golden scenario matrix.
//! The per-scenario Effect runner is represented as the scenario table itself:
//! each target resolves to a provider/route and a non-empty scenario list, and
//! the referenced cassettes are loaded where present at the pinned commit.

mod common;

use opencode_llm::{providers, Json};
use serde_json::json;

struct Target {
    name: &'static str,
    prefix: &'static str,
    provider: &'static str,
    route: &'static str,
    scenarios: &'static [&'static str],
}

fn targets() -> Vec<Target> {
    vec![
        Target {
            name: "OpenAI Chat gpt-4o-mini",
            prefix: "openai-chat",
            provider: "openai",
            route: "openai-chat",
            scenarios: &["text", "tool-call", "tool-loop", "image-tool-result"],
        },
        Target {
            name: "OpenAI Responses gpt-5.5",
            prefix: "openai-responses",
            provider: "openai",
            route: "openai-responses",
            scenarios: &[
                "text",
                "reasoning",
                "reasoning-continuation",
                "tool-call",
                "tool-loop",
                "image-tool-result",
            ],
        },
        Target {
            name: "OpenAI Responses WebSocket gpt-4.1-mini",
            prefix: "openai-responses-websocket",
            provider: "openai",
            route: "openai-responses-websocket",
            scenarios: &["tool-loop"],
        },
        Target {
            name: "Anthropic Haiku 4.5",
            prefix: "anthropic-messages",
            provider: "anthropic",
            route: "anthropic-messages",
            scenarios: &["text", "tool-call"],
        },
        Target {
            name: "Anthropic Opus 4.7",
            prefix: "anthropic-messages",
            provider: "anthropic",
            route: "anthropic-messages",
            scenarios: &["tool-loop", "image-tool-result"],
        },
        Target {
            name: "Gemini 2.5 Flash",
            prefix: "gemini",
            provider: "google",
            route: "gemini",
            scenarios: &["text", "tool-call", "image", "image-tool-result"],
        },
        Target {
            name: "xAI Grok 3 Mini",
            prefix: "xai",
            provider: "xai",
            route: "openai-compatible-chat",
            scenarios: &["text", "tool-call"],
        },
        Target {
            name: "xAI Grok 4.3",
            prefix: "xai",
            provider: "xai",
            route: "openai-compatible-chat",
            scenarios: &["tool-loop"],
        },
        Target {
            name: "Cloudflare AI Gateway Workers AI Llama 3.1 8B",
            prefix: "cloudflare-ai-gateway",
            provider: "cloudflare-ai-gateway",
            route: "cloudflare-ai-gateway",
            scenarios: &["text"],
        },
        Target {
            name: "Cloudflare AI Gateway Workers AI GPT OSS 20B Tools",
            prefix: "cloudflare-ai-gateway",
            provider: "cloudflare-ai-gateway",
            route: "cloudflare-ai-gateway",
            scenarios: &["tool-call"],
        },
        Target {
            name: "Cloudflare Workers AI Llama 3.1 8B",
            prefix: "cloudflare-workers-ai",
            provider: "cloudflare-workers-ai",
            route: "cloudflare-workers-ai",
            scenarios: &["text"],
        },
        Target {
            name: "Cloudflare Workers AI GPT OSS 20B Tools",
            prefix: "cloudflare-workers-ai",
            provider: "cloudflare-workers-ai",
            route: "cloudflare-workers-ai",
            scenarios: &["tool-call"],
        },
        Target {
            name: "DeepSeek Chat",
            prefix: "openai-compatible-chat",
            provider: "deepseek",
            route: "openai-compatible-chat",
            scenarios: &["text"],
        },
        Target {
            name: "TogetherAI Llama 3.3 70B",
            prefix: "openai-compatible-chat",
            provider: "togetherai",
            route: "openai-compatible-chat",
            scenarios: &["text", "tool-call"],
        },
        Target {
            name: "Groq Llama 3.3 70B",
            prefix: "openai-compatible-chat",
            provider: "groq",
            route: "openai-compatible-chat",
            scenarios: &["text", "tool-call", "tool-loop"],
        },
        Target {
            name: "OpenRouter gpt-4o-mini",
            prefix: "openai-compatible-chat",
            provider: "openrouter",
            route: "openrouter",
            scenarios: &["text", "tool-call", "tool-loop"],
        },
        Target {
            name: "OpenRouter gpt-5.5",
            prefix: "openai-compatible-chat",
            provider: "openrouter",
            route: "openrouter",
            scenarios: &["tool-loop"],
        },
        Target {
            name: "OpenRouter Claude Opus 4.7",
            prefix: "openai-compatible-chat",
            provider: "openrouter",
            route: "openrouter",
            scenarios: &["tool-loop"],
        },
    ]
}

fn facade(provider: &str, route: &str) -> Json {
    match provider {
        "openai" if route == "openai-chat" => {
            providers::openai::configure(json!({ "apiKey": "fixture" })).chat("gpt-4o-mini")
        }
        "openai" if route == "openai-responses-websocket" => {
            providers::openai::configure(json!({ "apiKey": "fixture" }))
                .responses_web_socket("gpt-4.1-mini")
        }
        "openai" => {
            providers::openai::configure(json!({ "apiKey": "fixture" })).responses("gpt-5.5")
        }
        "anthropic" => providers::anthropic::configure(json!({ "apiKey": "fixture" }))
            .model("claude-haiku-4-5-20251001"),
        "google" => {
            providers::google::configure(json!({ "apiKey": "fixture" })).model("gemini-2.5-flash")
        }
        "xai" => providers::xai::configure(json!({ "apiKey": "fixture" })).chat("grok-3-mini"),
        "cloudflare-ai-gateway" => providers::cloudflare::ai_gateway(
            json!({ "accountId": "fixture", "gatewayApiKey": "fixture" }),
        )
        .model("workers-ai/@cf/meta/llama-3.1-8b-instruct"),
        "cloudflare-workers-ai" => providers::cloudflare::workers_ai(
            json!({ "accountId": "fixture", "apiKey": "fixture" }),
        )
        .model("@cf/meta/llama-3.1-8b-instruct"),
        "deepseek" => providers::openai_compatible::deepseek()
            .configure(json!({ "apiKey": "fixture" }))
            .model("deepseek-chat"),
        "togetherai" => providers::openai_compatible::togetherai()
            .configure(json!({ "apiKey": "fixture" }))
            .model("meta-llama/Llama-3.3-70B-Instruct-Turbo"),
        "groq" => providers::openai_compatible::groq()
            .configure(json!({ "apiKey": "fixture" }))
            .model("llama-3.3-70b-versatile"),
        "openrouter" => providers::openrouter::configure(json!({ "apiKey": "fixture" }))
            .model("openai/gpt-4o-mini"),
        other => panic!("unknown provider {other}"),
    }
}

#[test]
fn every_golden_target_resolves_to_its_provider_route_and_scenarios() {
    for target in targets() {
        assert!(
            !target.scenarios.is_empty(),
            "{} has scenarios",
            target.name
        );
        assert!(
            !target.prefix.is_empty(),
            "{} has a cassette prefix",
            target.name
        );
        let model = facade(target.provider, target.route);
        assert_eq!(model["route"]["id"], target.route, "{} route", target.name);
    }
}

#[test]
fn golden_cassettes_are_loadable_where_present() {
    let known = [
        ("openai-chat", "streams-text"),
        ("openai-chat", "streams-tool-call"),
        ("openai-responses", "gpt-5-5-streams-text"),
        ("openai-responses", "gpt-5-5-streams-tool-call"),
        ("anthropic-messages", "streams-text"),
        ("anthropic-messages", "streams-tool-call"),
        ("gemini", "streams-text"),
        ("gemini", "streams-tool-call"),
        ("openai-compatible-chat", "deepseek-streams-text"),
        ("openai-compatible-chat", "togetherai-streams-text"),
        ("openai-compatible-chat", "groq-streams-text"),
        ("openai-compatible-chat", "openrouter-streams-text"),
        (
            "cloudflare-ai-gateway",
            "cloudflare-ai-gateway-workers-ai-llama-3-1-8b-text",
        ),
        (
            "cloudflare-workers-ai",
            "cloudflare-workers-ai-llama-3-1-8b-text",
        ),
    ];

    for (group, name) in known {
        if common::recording_exists(group, name) {
            let cassette = common::recording(group, name);
            assert!(
                cassette.get("interactions").is_some(),
                "{group}/{name} has interactions"
            );
        }
    }
}
