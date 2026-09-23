//! Port of packages/tui/test/util/tool-display.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/util/tool-display.ts; see docs/TEST-PORT.md.

use opencode_tui::tool_display::{tool_display_metadata, web_search_provider_label};
use serde_json::json;

#[test]
fn labels_known_providers() {
    assert_eq!(
        web_search_provider_label(Some(&json!("parallel"))),
        "Parallel Web Search"
    );
    assert_eq!(
        web_search_provider_label(Some(&json!("exa"))),
        "Exa Web Search"
    );
}

#[test]
fn uses_the_generic_label_for_other_values() {
    for provider in [json!(null), json!({}), json!([]), json!(1), json!("other")] {
        assert_eq!(web_search_provider_label(Some(&provider)), "Web Search");
    }
    assert_eq!(web_search_provider_label(None), "Web Search");
}

#[test]
fn returns_structured_metadata_for_non_pending_states() {
    let structured = json!({ "provider": "parallel", "numResults": 3 });

    assert_eq!(
        tool_display_metadata(Some(
            &json!({ "status": "running", "structured": structured })
        )),
        structured
    );
    assert_eq!(
        tool_display_metadata(Some(
            &json!({ "status": "completed", "structured": structured })
        )),
        structured
    );
    assert_eq!(
        tool_display_metadata(Some(
            &json!({ "status": "error", "structured": structured })
        )),
        structured
    );
}

#[test]
fn does_not_expose_pending_or_malformed_metadata() {
    let empty = json!({});
    assert_eq!(
        tool_display_metadata(Some(
            &json!({ "status": "pending", "structured": { "provider": "exa" } })
        )),
        empty
    );
    assert_eq!(
        tool_display_metadata(Some(&json!({ "status": "completed" }))),
        empty
    );
    assert_eq!(
        tool_display_metadata(Some(&json!({ "status": "completed", "structured": null }))),
        empty
    );
    assert_eq!(
        tool_display_metadata(Some(&json!({ "status": "completed", "structured": [] }))),
        empty
    );
    assert_eq!(tool_display_metadata(None), empty);
}
