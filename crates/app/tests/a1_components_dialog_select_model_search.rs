//! Port of packages/app/src/components/dialog-select-model-search.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::dialog_select_model_search::matches_model_search;

#[test]
fn matches_model_names_across_separators() {
    assert!(matches_model_search("gpt 5", &["GPT-5.5"]));
    assert!(matches_model_search("gpt-5", &["GPT-5.5"]));
    assert!(matches_model_search("gpt5", &["GPT-5.5"]));
}

#[test]
fn matches_any_searchable_model_field() {
    assert!(matches_model_search(
        "open ai",
        &["GPT-5.5", "gpt-5.5", "OpenAI"]
    ));
    assert!(matches_model_search(
        "gpt 5",
        &["GPT-5.5", "gpt-5.5", "OpenAI"]
    ));
}

#[test]
fn does_not_match_unrelated_searches() {
    assert!(!matches_model_search(
        "claude",
        &["GPT-5.5", "gpt-5.5", "OpenAI"]
    ));
}
