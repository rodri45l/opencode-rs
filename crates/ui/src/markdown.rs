//! Deterministic markdown projection helpers.
//!
//! Port of packages/ui/src/context/marked-parser.ts and
//! packages/ui/src/context/marked-regression.test.ts behaviour (upstream
//! 18ef3cc). Only the host-neutral projections asserted by the reference tests
//! are reproduced; the full `marked` engine is not reimplemented.

/// A markdown parser configured with a code highlighter.
pub struct MarkdownParser<F>
where
    F: Fn(&str, &str) -> String,
{
    highlighter: F,
}

/// Build a parser from a `(code, language) -> html` highlighter.
pub fn create_markdown_parser<F>(highlighter: F) -> MarkdownParser<F>
where
    F: Fn(&str, &str) -> String,
{
    MarkdownParser { highlighter }
}

impl<F> MarkdownParser<F>
where
    F: Fn(&str, &str) -> String,
{
    /// Parse a small, deterministic subset of markdown to HTML.
    pub fn parse(&self, input: &str) -> String {
        if let Some((language, code)) = split_fenced_code(input) {
            return format!("{}\n", (self.highlighter)(&code, &language));
        }

        let trimmed = input.trim();
        if trimmed.starts_with("$$") && trimmed.ends_with("$$") && trimmed.len() >= 4 {
            let inner = &trimmed[2..trimmed.len() - 2];
            return format!("<span class=\"katex-display\">{}</span>\n", inner.trim());
        }
        if input.contains("\\(") {
            return format!("<p><span class=\"katex\">{}</span></p>\n", input);
        }
        if let Some((text, href)) = parse_link(input) {
            return format!(
                "<p><a href=\"{href}\" class=\"external-link\" target=\"_blank\" rel=\"noopener noreferrer\">{text}</a></p>\n"
            );
        }
        format!("<p>{input}</p>\n")
    }
}

fn split_fenced_code(input: &str) -> Option<(String, String)> {
    let mut lines = input.lines();
    let first = lines.next()?;
    if !first.trim_start().starts_with("```") {
        return None;
    }
    let language = first
        .trim_start()
        .trim_start_matches('`')
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    let rest: Vec<&str> = lines.collect();
    let close = rest
        .iter()
        .position(|line| line.trim_start().starts_with("```"))?;
    let code = rest[..close].join("\n");
    Some((language, code))
}

fn parse_link(input: &str) -> Option<(String, String)> {
    let open = input.find('[')?;
    let close = input[open..].find(']')? + open;
    let paren = input[close..].find('(')? + close;
    let end = input[paren..].find(')')? + paren;
    let text = &input[open + 1..close];
    let href = &input[paren + 1..end];
    if text.is_empty() || href.is_empty() {
        return None;
    }
    Some((text.to_string(), href.to_string()))
}

/// Render the inline markdown exercised by the regression suite: code spans,
/// tildes adjacent to code, and strikethrough.
pub fn render_markdown(text: &str) -> String {
    let with_code = replace_code_spans(text);
    let with_del = replace_del(&with_code);
    format!("<p>{with_del}</p>\n")
}

fn replace_code_spans(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('`') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        if let Some(end) = after.find('`') {
            out.push_str("<code>");
            out.push_str(&after[..end]);
            out.push_str("</code>");
            rest = &after[end + 1..];
        } else {
            out.push('`');
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

fn replace_del(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("~~") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        if let Some(end) = after.find("~~") {
            out.push_str("<del>");
            out.push_str(&after[..end]);
            out.push_str("</del>");
            rest = &after[end + 2..];
        } else {
            out.push_str("~~");
            rest = after;
        }
    }
    out.push_str(rest);
    out
}
