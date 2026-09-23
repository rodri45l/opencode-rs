//! Port of packages/app/src/components/prompt-input/placeholder.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::prompt_placeholder::{
    prompt_design_placeholder, prompt_placeholder, Mode, PlaceholderInput,
};

fn t(key: &str, params: &[(&str, &str)]) -> String {
    if key == "ui.promptInput.placeholder.normal" {
        let slash = params
            .iter()
            .find(|(k, _)| *k == "slash")
            .map(|(_, v)| *v)
            .unwrap_or("");
        let at = params
            .iter()
            .find(|(k, _)| *k == "at")
            .map(|(_, v)| *v)
            .unwrap_or("");
        return format!("Ask anything, {slash} for commands, {at} for context...");
    }
    match params.iter().find(|(k, _)| *k == "example") {
        Some((_, example)) => format!("{key}:{example}"),
        None => key.to_string(),
    }
}

#[test]
fn returns_shell_placeholder_in_shell_mode() {
    assert_eq!(
        prompt_placeholder(PlaceholderInput {
            mode: Mode::Shell,
            comment_count: 0,
            example: "example".into(),
            suggest: true,
        }),
        t("prompt.placeholder.shell", &[("example", "example")])
    );
}

#[test]
fn returns_summarize_placeholders_for_comment_context() {
    assert_eq!(
        prompt_placeholder(PlaceholderInput {
            mode: Mode::Normal,
            comment_count: 1,
            example: "example".into(),
            suggest: true,
        }),
        t(
            "prompt.placeholder.summarizeComment",
            &[("example", "example")]
        )
    );
    assert_eq!(
        prompt_placeholder(PlaceholderInput {
            mode: Mode::Normal,
            comment_count: 2,
            example: "example".into(),
            suggest: true,
        }),
        t(
            "prompt.placeholder.summarizeComments",
            &[("example", "example")]
        )
    );
}

#[test]
fn returns_default_placeholder_with_example_when_suggestions_enabled() {
    assert_eq!(
        prompt_placeholder(PlaceholderInput {
            mode: Mode::Normal,
            comment_count: 0,
            example: "translated-example".into(),
            suggest: true,
        }),
        t(
            "prompt.placeholder.normal",
            &[("example", "translated-example")]
        )
    );
}

#[test]
fn returns_simple_placeholder_when_suggestions_disabled() {
    assert_eq!(
        prompt_placeholder(PlaceholderInput {
            mode: Mode::Normal,
            comment_count: 0,
            example: "translated-example".into(),
            suggest: false,
        }),
        t("prompt.placeholder.simple", &[])
    );
}

#[test]
fn composes_the_design_placeholder_from_localized_fragments() {
    assert_eq!(
        prompt_design_placeholder(Mode::Normal, "fallback"),
        t(
            "ui.promptInput.placeholder.normal",
            &[("slash", "/"), ("at", "@")]
        )
    );
}

#[test]
fn preserves_the_shell_placeholder() {
    assert_eq!(
        prompt_design_placeholder(Mode::Shell, "Enter shell command..."),
        "Enter shell command..."
    );
}
