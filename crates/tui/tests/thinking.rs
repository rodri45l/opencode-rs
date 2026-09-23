//! Port of packages/tui/test/cli/tui/thinking.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/context/thinking.ts; see docs/TEST-PORT.md.

use opencode_tui::thinking::{reasoning_summary, ReasoningSummary};

fn summary(title: Option<&str>, body: &str) -> ReasoningSummary {
    ReasoningSummary {
        title: title.map(str::to_string),
        body: body.to_string(),
    }
}

#[test]
fn extracts_a_leading_summary_title_and_leaves_markdown_body() {
    assert_eq!(
        reasoning_summary("**Continuing Quality Review**\n\nDetails.\n\n**Next section**\n\nMore."),
        summary(
            Some("Continuing Quality Review"),
            "Details.\n\n**Next section**\n\nMore."
        )
    );
}

#[test]
fn extracts_a_completed_title_before_its_streamed_body_arrives() {
    assert_eq!(
        reasoning_summary("**Continuing Quality Review**"),
        summary(Some("Continuing Quality Review"), "")
    );
}

#[test]
fn preserves_markdown_significant_indentation_in_the_extracted_body() {
    assert_eq!(
        reasoning_summary("**Continuing Quality Review**\n\n    const value = true\n"),
        summary(Some("Continuing Quality Review"), "    const value = true")
    );
}

#[test]
fn does_not_consume_ordinary_leading_bold_content() {
    assert_eq!(
        reasoning_summary("**Important:** keep this in the body."),
        summary(None, "**Important:** keep this in the body.")
    );
}

#[test]
fn leaves_content_without_a_leading_title_in_its_body() {
    assert_eq!(
        reasoning_summary("Details only."),
        summary(None, "Details only.")
    );
}
