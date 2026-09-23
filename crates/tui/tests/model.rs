//! Port of packages/tui/test/util/model.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/util/model.ts; see docs/TEST-PORT.md.

use opencode_tui::model::{parse, ParsedModel};

#[test]
fn splits_provider_from_a_nested_model_identifier() {
    assert_eq!(
        parse("provider/org/model"),
        ParsedModel {
            provider_id: "provider".to_string(),
            model_id: "org/model".to_string(),
        }
    );
    assert_eq!(
        parse("invalid"),
        ParsedModel {
            provider_id: "invalid".to_string(),
            model_id: String::new(),
        }
    );
}
