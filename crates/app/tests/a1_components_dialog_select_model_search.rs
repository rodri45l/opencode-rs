//! Port of packages/app/src/components/dialog-select-model-search.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

// Local stub (fast wave): real module lands later.
fn matches_model_search(_search: &str, _fields: &[&str]) -> bool {
    false
}

#[test]
#[ignore = "porting: components/dialog-select-model-search not implemented"]
fn matches_model_names_across_separators() {
    assert!(matches_model_search("gpt 5", &["GPT-5.5"]));
    assert!(matches_model_search("gpt-5", &["GPT-5.5"]));
    assert!(matches_model_search("gpt5", &["GPT-5.5"]));
}

#[test]
#[ignore = "porting: components/dialog-select-model-search not implemented"]
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
#[ignore = "porting: components/dialog-select-model-search not implemented"]
fn does_not_match_unrelated_searches() {
    assert!(!matches_model_search(
        "claude",
        &["GPT-5.5", "gpt-5.5", "OpenAI"]
    ));
}
