//! Port of packages/core/test/provider-groq.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: Groq forwards an unknown `reasoningEffort` provider option
//! verbatim as `reasoning_effort`. Re-derived: the `@ai-sdk/groq` model and mock
//! fetch are replaced by a direct body lowering.

use opencode_core::groq::GroqPlugin;
use serde_json::json;

const NOTE: &str = "porting: groq provider lowering not implemented";

#[test]
#[ignore = "porting: groq provider lowering not implemented"]
fn passes_through_unknown_reasoning_effort() {
    let mut body = json!({});
    GroqPlugin::apply_body(
        &json!({ "groq": { "reasoningEffort": "custom" } }),
        &mut body,
    )
    .expect(NOTE);
    assert_eq!(body["reasoning_effort"], json!("custom"));
}
