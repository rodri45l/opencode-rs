//! Port of packages/tui/test/cli/cmd/tui/model-options.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/component/dialog-model.tsx; see docs/TEST-PORT.md.

use opencode_tui::model_options::{sort_model_options, ModelOption};

fn option(title: &str, release_date: &str, footer: Option<&str>) -> ModelOption {
    ModelOption {
        title: title.to_string(),
        release_date: release_date.to_string(),
        footer: footer.map(str::to_string),
    }
}

fn titles(options: Vec<ModelOption>) -> Vec<String> {
    options.into_iter().map(|option| option.title).collect()
}

#[test]
fn orders_provider_scoped_model_choices_by_newest_release_first() {
    let sorted = sort_model_options(
        vec![
            option("GPT 5.2", "2025-12-11", None),
            option("GPT 5.4", "2026-03-05", None),
            option("GPT 5.1", "2025-11-13", None),
        ],
        true,
    );

    assert_eq!(titles(sorted), vec!["GPT 5.4", "GPT 5.2", "GPT 5.1"]);
}

#[test]
fn orders_regular_model_choices_free_first_and_then_newest_first() {
    let sorted = sort_model_options(
        vec![
            option("GLM 5", "2025-07-28", None),
            option("GLM 5.1", "2025-12-09", None),
            option("GLM 5.2", "2026-02-16", None),
            option("Free old", "2024-01-01", Some("Free")),
            option("Free new", "2025-01-01", Some("Free")),
        ],
        false,
    );

    assert_eq!(
        titles(sorted),
        vec!["Free new", "Free old", "GLM 5.2", "GLM 5.1", "GLM 5"]
    );
}
