//! Port of packages/opencode/test/tool/websearch.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: stable per-session provider selection with env override
//! and feature flags, provider gating, branded labels, analytics model id, and
//! the JSON-RPC/SSE MCP response parser.

use opencode_server::tools::ToolError;
use opencode_server::websearch::{
    parse_response, select_web_search_provider, web_search_enabled, web_search_model_name,
    web_search_provider_label, WebSearchFlags,
};
use std::sync::Mutex;

const SESSION_ID: &str = "ses_0196aabbccddeeff001122334455";

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn no_flags() -> WebSearchFlags {
    WebSearchFlags::default()
}

#[test]
fn selects_a_stable_provider_per_session() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    assert_eq!(
        select_web_search_provider(SESSION_ID, no_flags()),
        select_web_search_provider(SESSION_ID, no_flags())
    );
}

#[test]
fn supports_an_operational_override() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let original = std::env::var("OPENCODE_WEBSEARCH_PROVIDER").ok();

    std::env::set_var("OPENCODE_WEBSEARCH_PROVIDER", "parallel");
    assert_eq!(
        select_web_search_provider(SESSION_ID, no_flags()),
        "parallel"
    );

    std::env::set_var("OPENCODE_WEBSEARCH_PROVIDER", "exa");
    assert_eq!(select_web_search_provider(SESSION_ID, no_flags()), "exa");

    match original {
        Some(value) => std::env::set_var("OPENCODE_WEBSEARCH_PROVIDER", value),
        None => std::env::remove_var("OPENCODE_WEBSEARCH_PROVIDER"),
    }
}

#[test]
fn routes_to_exa_when_the_exa_flag_is_enabled() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    assert_eq!(
        select_web_search_provider(
            SESSION_ID,
            WebSearchFlags {
                exa: true,
                parallel: false,
            }
        ),
        "exa"
    );
}

#[test]
fn routes_to_parallel_when_the_parallel_flag_is_enabled() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    assert_eq!(
        select_web_search_provider(
            SESSION_ID,
            WebSearchFlags {
                exa: false,
                parallel: true,
            }
        ),
        "parallel"
    );
}

#[test]
fn is_enabled_for_opencode_providers_or_explicit_flags() {
    assert!(web_search_enabled("opencode", no_flags()));
    assert!(web_search_enabled("opencode-go", no_flags()));
    assert!(!web_search_enabled("openai", no_flags()));
    assert!(web_search_enabled(
        "openai",
        WebSearchFlags {
            exa: true,
            parallel: false,
        }
    ));
    assert!(web_search_enabled(
        "openai",
        WebSearchFlags {
            exa: false,
            parallel: true,
        }
    ));
}

#[test]
fn uses_branded_labels() {
    assert_eq!(
        web_search_provider_label(Some("parallel")),
        "Parallel Web Search"
    );
    assert_eq!(web_search_provider_label(Some("exa")), "Exa Web Search");
    assert_eq!(web_search_provider_label(None), "Web Search");
}

#[test]
fn uses_the_provider_api_model_id_for_analytics() {
    assert_eq!(web_search_model_name("claude-opus-4.7"), "claude-opus-4.7");
}

fn payload() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "content": [{ "type": "text", "text": "search results" }]
        }
    })
    .to_string()
}

#[test]
fn parses_plain_json_rpc_responses() -> Result<(), ToolError> {
    let result = parse_response(&payload())?;
    assert_eq!(result, "search results");
    Ok(())
}

#[test]
fn parses_sse_json_rpc_responses() -> Result<(), ToolError> {
    let result = parse_response(&format!("event: message\ndata: {}\n\n", payload()))?;
    assert_eq!(result, "search results");
    Ok(())
}

#[test]
fn ignores_non_json_sse_data_frames() -> Result<(), ToolError> {
    let result = parse_response(&format!("data: [DONE]\ndata: {}\n\n", payload()))?;
    assert_eq!(result, "search results");
    Ok(())
}
