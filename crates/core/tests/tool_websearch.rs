//! Port of packages/core/test/tool-websearch.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: numeric controls are bounded, provider selection is stable
//! per session with explicit flags (`parallel` before `exa`) and an operational
//! override, and JSON-RPC responses parse from either a plain body or an SSE
//! stream. Re-derived: the `ToolRegistry`/`HttpClient`/`PermissionV2` wiring and
//! transport assertions are dropped.

use opencode_core::tool_websearch::{
    SearchConfig, WebSearchTool, MAX_CONTEXT_CHARACTERS, MAX_NUM_RESULTS,
};
use serde_json::json;

const NOTE: &str = "porting: web search tool not implemented";

fn payload(text: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": { "content": [{ "type": "text", "text": text }] }
    })
    .to_string()
}

#[test]
#[ignore = "porting: web search tool not implemented"]
fn rejects_out_of_range_numeric_controls() {
    assert!(WebSearchTool::parse_input(&json!({ "query": "x", "numResults": 0 })).is_err());
    assert!(WebSearchTool::parse_input(
        &json!({ "query": "x", "numResults": MAX_NUM_RESULTS + 1 })
    )
    .is_err());
    assert!(WebSearchTool::parse_input(
        &json!({ "query": "x", "contextMaxCharacters": MAX_CONTEXT_CHARACTERS + 1 })
    )
    .is_err());
}

#[test]
#[ignore = "porting: web search tool not implemented"]
fn selects_a_stable_provider_per_session() {
    let config = SearchConfig::default();
    assert_eq!(
        WebSearchTool::select_provider("ses_one", &config, None).expect(NOTE),
        WebSearchTool::select_provider("ses_one", &config, None).expect(NOTE)
    );
}

#[test]
#[ignore = "porting: web search tool not implemented"]
fn honors_explicit_operational_overrides_and_flags() {
    let none = SearchConfig::default();
    assert_eq!(
        WebSearchTool::select_provider("ses_one", &none, Some("parallel")).expect(NOTE),
        "parallel"
    );
    assert_eq!(
        WebSearchTool::select_provider("ses_one", &none, Some("exa")).expect(NOTE),
        "exa"
    );
    assert_eq!(
        WebSearchTool::select_provider(
            "ses_one",
            &SearchConfig {
                enable_exa: true,
                enable_parallel: true,
            },
            None,
        )
        .expect(NOTE),
        "parallel"
    );
    assert_eq!(
        WebSearchTool::select_provider(
            "ses_one",
            &SearchConfig {
                enable_exa: true,
                enable_parallel: false,
            },
            None,
        )
        .expect(NOTE),
        "exa"
    );
}

#[test]
#[ignore = "porting: web search tool not implemented"]
fn parses_plain_and_sse_json_rpc_responses() {
    assert_eq!(
        WebSearchTool::parse_response(&payload("search results")).expect(NOTE),
        "search results"
    );
    let sse = format!(
        "data: [DONE]\nevent: message\ndata: {}\n\n",
        payload("search results")
    );
    assert_eq!(
        WebSearchTool::parse_response(&sse).expect(NOTE),
        "search results"
    );
}
