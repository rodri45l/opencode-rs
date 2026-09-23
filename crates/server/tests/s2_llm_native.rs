#![allow(dead_code)]

//! Port of packages/opencode/test/session/llm-native.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: native request route selection (npm package -> route id and
//! default base URL), unsupported-package fast failure, runtime enablement
//! (supported/unsupported reason) for OpenAI/Anthropic/opencode API-key models,
//! and normalized request projection (system prompt folding, generation
//! settings, tools, session parts -> native content). The recorded tool-loop
//! cases need a live HTTP runtime and are dropped here (noted in
//! PORT-STATUS.s2.json). Stubs are local per the fast-wave protocol.

use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
enum S2Error {
    NotImplemented(&'static str),
}

impl std::fmt::Display for S2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S2Error::NotImplemented(what) => write!(f, "not implemented: {what}"),
        }
    }
}

impl std::error::Error for S2Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Route {
    id: String,
    base_url: String,
}

fn native_model(_npm: &str, _url: &str) -> Result<Route, S2Error> {
    Err(S2Error::NotImplemented("LLMNative.model"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RuntimeStatus {
    Supported { api_key: String },
    Unsupported { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeModel {
    provider_id: String,
    npm: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeProvider {
    id: String,
    options_api_key: Option<String>,
    key: Option<String>,
    has_fetch_override: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Auth {
    None,
    Oauth,
    Api { key: String },
}

fn runtime_status(
    _model: &RuntimeModel,
    _provider: &RuntimeProvider,
    _auth: &Auth,
) -> Result<RuntimeStatus, S2Error> {
    Err(S2Error::NotImplemented("LLMNativeRuntime.status"))
}

#[derive(Debug, Clone, PartialEq)]
enum NativePart {
    Text {
        text: String,
        provider_metadata: Option<Value>,
    },
    Media {
        media_type: String,
        filename: String,
        data: String,
    },
    Reasoning {
        text: String,
        provider_metadata: Option<Value>,
    },
    ToolCall {
        id: String,
        name: String,
        input: Value,
        provider_metadata: Option<Value>,
    },
    ToolResult {
        id: String,
        name: String,
        result: Value,
        provider_metadata: Option<Value>,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct NativeMessage {
    role: String,
    content: Vec<NativePart>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeTool {
    name: String,
    description: String,
    input_schema: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Generation {
    temperature: Option<String>,
    top_p: Option<String>,
    top_k: Option<u64>,
    max_tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
struct RequestProjection {
    system: Vec<String>,
    generation: Generation,
    provider_options: Value,
    tool_choice: Option<String>,
    tools: Vec<NativeTool>,
    messages: Vec<NativeMessage>,
}

#[derive(Debug, Clone, PartialEq)]
struct RequestInput {
    npm: String,
    url: String,
    system: Vec<String>,
    messages: Vec<NativeMessage>,
    temperature: Option<String>,
    top_p: Option<String>,
    top_k: Option<u64>,
    max_output_tokens: Option<u64>,
    provider_options: Value,
    tools: Vec<NativeTool>,
    tool_choice: Option<String>,
}

fn native_request(_input: &RequestInput) -> Result<RequestProjection, S2Error> {
    Err(S2Error::NotImplemented("LLMNative.request"))
}

#[test]
#[ignore = "porting: LLMNative.model not implemented"]
fn selects_native_request_routes_for_provider_packages() {
    let openai = native_model("@ai-sdk/openai", "").expect("openai route");
    assert_eq!(openai.id, "openai-responses");
    assert_eq!(openai.base_url, "https://api.openai.com/v1");

    let anthropic = native_model("@ai-sdk/anthropic", "").expect("anthropic route");
    assert_eq!(anthropic.id, "anthropic-messages");
    assert_eq!(anthropic.base_url, "https://api.anthropic.com/v1");

    let google = native_model("@ai-sdk/google", "").expect("google route");
    assert_eq!(google.id, "gemini");
    assert_eq!(
        google.base_url,
        "https://generativelanguage.googleapis.com/v1beta"
    );

    let compatible = native_model("@ai-sdk/openai-compatible", "https://ai.example.test/v1")
        .expect("compatible route");
    assert_eq!(compatible.id, "openai-compatible-chat");
    assert_eq!(compatible.base_url, "https://ai.example.test/v1");

    let openrouter = native_model("@openrouter/ai-sdk-provider", "").expect("openrouter route");
    assert_eq!(openrouter.id, "openrouter");
    assert_eq!(openrouter.base_url, "https://openrouter.ai/api/v1");
}

#[test]
#[ignore = "porting: LLMNative.model not implemented"]
fn fails_fast_for_unsupported_provider_packages() {
    let error = native_model("unknown-provider", "").expect_err("unsupported");
    assert!(error
        .to_string()
        .contains("Native LLM request adapter does not support provider package unknown-provider"));
}

fn openai_model() -> RuntimeModel {
    RuntimeModel {
        provider_id: "openai".to_string(),
        npm: "@ai-sdk/openai".to_string(),
    }
}

fn openai_provider(api_key: Option<&str>) -> RuntimeProvider {
    RuntimeProvider {
        id: "openai".to_string(),
        options_api_key: api_key.map(str::to_string),
        key: None,
        has_fetch_override: false,
    }
}

#[test]
#[ignore = "porting: LLMNativeRuntime.status not implemented"]
fn enables_native_runtime_for_supported_openai_api_key_models() {
    assert_eq!(
        runtime_status(
            &openai_model(),
            &openai_provider(Some("test-openai-key")),
            &Auth::None
        )
        .expect("status"),
        RuntimeStatus::Supported {
            api_key: "test-openai-key".to_string()
        }
    );

    let opencode = RuntimeModel {
        provider_id: "opencode".to_string(),
        npm: "@ai-sdk/openai".to_string(),
    };
    let opencode_provider = RuntimeProvider {
        id: "opencode".to_string(),
        ..openai_provider(Some("test-openai-key"))
    };
    assert_eq!(
        runtime_status(&opencode, &opencode_provider, &Auth::None).expect("status"),
        RuntimeStatus::Supported {
            api_key: "test-openai-key".to_string()
        }
    );

    let compatible = RuntimeModel {
        provider_id: "opencode".to_string(),
        npm: "@ai-sdk/openai-compatible".to_string(),
    };
    assert_eq!(
        runtime_status(&compatible, &opencode_provider, &Auth::None).expect("status"),
        RuntimeStatus::Supported {
            api_key: "test-openai-key".to_string()
        }
    );
}

#[test]
#[ignore = "porting: LLMNativeRuntime.status not implemented"]
fn reports_unsupported_reasons_for_native_runtime() {
    let google = RuntimeModel {
        provider_id: "google".to_string(),
        npm: "@ai-sdk/google".to_string(),
    };
    let google_provider = RuntimeProvider {
        id: "google".to_string(),
        ..openai_provider(Some("test-openai-key"))
    };
    assert_eq!(
        runtime_status(&google, &google_provider, &Auth::None).expect("status"),
        RuntimeStatus::Unsupported {
            reason: "provider is not openai, opencode, or anthropic".to_string()
        }
    );

    assert_eq!(
        runtime_status(
            &openai_model(),
            &openai_provider(Some("test-openai-key")),
            &Auth::Oauth
        )
        .expect("status"),
        RuntimeStatus::Unsupported {
            reason: "OAuth auth requires a provider fetch override".to_string()
        }
    );

    let with_fetch = RuntimeProvider {
        has_fetch_override: true,
        ..openai_provider(Some("ignored"))
    };
    assert_eq!(
        runtime_status(&openai_model(), &with_fetch, &Auth::Oauth).expect("status"),
        RuntimeStatus::Supported {
            api_key: "opencode-oauth".to_string()
        }
    );

    let google_pkg = RuntimeModel {
        provider_id: "openai".to_string(),
        npm: "@ai-sdk/google".to_string(),
    };
    assert_eq!(
        runtime_status(&google_pkg, &openai_provider(None), &Auth::None).expect("status"),
        RuntimeStatus::Unsupported {
            reason: "provider package is not OpenAI, OpenAI-compatible, or Anthropic".to_string()
        }
    );

    assert_eq!(
        runtime_status(&openai_model(), &openai_provider(None), &Auth::None).expect("status"),
        RuntimeStatus::Unsupported {
            reason: "API key is not configured".to_string()
        }
    );
}

#[test]
#[ignore = "porting: LLMNativeRuntime.status not implemented"]
fn enables_native_runtime_for_anthropic_api_key_models() {
    let anthropic = RuntimeModel {
        provider_id: "anthropic".to_string(),
        npm: "@ai-sdk/anthropic".to_string(),
    };
    let provider = RuntimeProvider {
        id: "anthropic".to_string(),
        options_api_key: Some("test-anthropic-key".to_string()),
        key: None,
        has_fetch_override: false,
    };
    assert_eq!(
        runtime_status(&anthropic, &provider, &Auth::None).expect("status"),
        RuntimeStatus::Supported {
            api_key: "test-anthropic-key".to_string()
        }
    );
}

#[test]
#[ignore = "porting: LLMNativeRuntime.status not implemented"]
fn prefers_console_provider_api_key_over_stored_auth() {
    let provider = RuntimeProvider {
        id: "opencode".to_string(),
        options_api_key: Some("console-token".to_string()),
        key: Some("zen-token".to_string()),
        has_fetch_override: false,
    };
    assert_eq!(
        runtime_status(
            &openai_model(),
            &provider,
            &Auth::Api {
                key: "zen-token".to_string()
            }
        )
        .expect("status"),
        RuntimeStatus::Supported {
            api_key: "console-token".to_string()
        }
    );

    let provider_key = RuntimeProvider {
        id: "openai".to_string(),
        options_api_key: None,
        key: Some("provider-key".to_string()),
        has_fetch_override: false,
    };
    assert_eq!(
        runtime_status(&openai_model(), &provider_key, &Auth::None).expect("status"),
        RuntimeStatus::Supported {
            api_key: "provider-key".to_string()
        }
    );
}

#[test]
#[ignore = "porting: LLMNative.request not implemented"]
fn maps_normalized_stream_inputs_to_a_native_llm_request() {
    let input = RequestInput {
        npm: "@ai-sdk/openai".to_string(),
        url: "https://api.openai.com/v1".to_string(),
        system: vec!["agent system".to_string()],
        messages: vec![
            NativeMessage {
                role: "user".to_string(),
                content: vec![
                    NativePart::Text {
                        text: "hello".to_string(),
                        provider_metadata: Some(
                            json!({ "openai": { "cacheControl": { "type": "ephemeral" } } }),
                        ),
                    },
                    NativePart::Media {
                        media_type: "image/png".to_string(),
                        filename: "img.png".to_string(),
                        data: "data:image/png;base64,Zm9v".to_string(),
                    },
                ],
            },
            NativeMessage {
                role: "assistant".to_string(),
                content: vec![
                    NativePart::Reasoning {
                        text: "thinking".to_string(),
                        provider_metadata: Some(
                            json!({ "openai": { "encryptedContent": "secret" } }),
                        ),
                    },
                    NativePart::Text {
                        text: "I'll run it".to_string(),
                        provider_metadata: None,
                    },
                    NativePart::ToolCall {
                        id: "call-1".to_string(),
                        name: "bash".to_string(),
                        input: json!({ "command": "ls" }),
                        provider_metadata: Some(json!({ "openai": { "itemId": "item-1" } })),
                    },
                ],
            },
            NativeMessage {
                role: "tool".to_string(),
                content: vec![NativePart::ToolResult {
                    id: "call-1".to_string(),
                    name: "bash".to_string(),
                    result: json!({ "type": "text", "value": "ok" }),
                    provider_metadata: Some(json!({ "openai": { "outputId": "output-1" } })),
                }],
            },
        ],
        temperature: Some("0.2".to_string()),
        top_p: Some("0.9".to_string()),
        top_k: Some(40),
        max_output_tokens: Some(1024),
        provider_options: json!({ "openai": { "store": false } }),
        tools: vec![NativeTool {
            name: "bash".to_string(),
            description: "Run a shell command".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": { "command": { "type": "string" } },
                "required": ["command"]
            }),
        }],
        tool_choice: Some("required".to_string()),
    };

    let request = native_request(&input).expect("native request");
    assert_eq!(
        request.system,
        vec![
            "agent system".to_string(),
            "system from messages".to_string()
        ]
    );
    assert_eq!(
        request.generation,
        Generation {
            temperature: Some("0.2".to_string()),
            top_p: Some("0.9".to_string()),
            top_k: Some(40),
            max_tokens: Some(1024),
        }
    );
    assert_eq!(
        request.provider_options,
        json!({ "openai": { "store": false } })
    );
    assert_eq!(request.tool_choice.as_deref(), Some("required"));
    assert_eq!(request.tools, input.tools);
    assert_eq!(request.messages, input.messages);
}
