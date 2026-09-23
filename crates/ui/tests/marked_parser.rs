//! Port of packages/ui/src/context/marked-parser.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/ui/src/context/marked-parser.ts; see docs/TEST-PORT.md.

use opencode_ui::markdown::create_markdown_parser;

fn parser() -> impl Fn(&str) -> String {
    let inner = create_markdown_parser(|code: &str, language: &str| {
        format!("<pre data-language=\"{language}\">{code}</pre>")
    });
    move |input: &str| inner.parse(input)
}

#[test]
fn renders_links_with_application_attributes() {
    let parse = parser();
    assert_eq!(
        parse("[OpenCode](https://opencode.ai)"),
        "<p><a href=\"https://opencode.ai\" class=\"external-link\" target=\"_blank\" rel=\"noopener noreferrer\">OpenCode</a></p>\n"
    );
}

#[test]
fn renders_inline_and_block_math() {
    let parse = parser();
    assert!(parse("\\(x^2\\)").contains("<span class=\"katex\">"));
    assert!(parse("$$\nx^2\n$$\n").contains("<span class=\"katex-display\">"));
}

#[test]
fn uses_the_configured_code_highlighter() {
    let parse = parser();
    assert_eq!(
        parse("```ts\nconst value = 1\n```\n"),
        "<pre data-language=\"ts\">const value = 1</pre>\n"
    );
}
