//! Port of packages/opencode/test/plugin/github-copilot.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the `chat.headers` hook stamps `X-Interaction-Id` from
//! the session id for Copilot providers (including enterprise) across both
//! Copilot npm adapters, and leaves other providers untouched.

use opencode_server::port::plugin::apply_copilot_interaction_headers;
use std::collections::BTreeMap;

fn headers_with_existing() -> BTreeMap<String, String> {
    let mut headers = BTreeMap::new();
    headers.insert("x-existing".to_string(), "preserved".to_string());
    headers
}

#[test]
fn uses_the_session_id_for_interaction_headers() {
    for provider_id in ["github-copilot", "github-copilot-enterprise"] {
        for _npm in ["@ai-sdk/github-copilot", "@ai-sdk/anthropic"] {
            for session_id in ["ses_one", "ses_one", "ses_two"] {
                let mut headers = headers_with_existing();
                apply_copilot_interaction_headers(provider_id, session_id, &mut headers);
                assert_eq!(
                    headers.get("X-Interaction-Id").map(String::as_str),
                    Some(session_id)
                );
                assert_eq!(
                    headers.get("x-existing").map(String::as_str),
                    Some("preserved")
                );
            }
        }
    }
}

#[test]
fn does_not_add_interaction_headers_to_other_providers() {
    let mut headers = headers_with_existing();
    apply_copilot_interaction_headers("openai", "ses_one", &mut headers);
    assert_eq!(headers.len(), 1);
    assert_eq!(
        headers.get("x-existing").map(String::as_str),
        Some("preserved")
    );
}
