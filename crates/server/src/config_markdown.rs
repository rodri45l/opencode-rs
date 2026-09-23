//! Markdown `@path` reference extraction.
//!
//! Ports the observable behaviour of `packages/opencode/src/config/markdown.ts`:
//! `ConfigMarkdown.files` returns `(full_match, path)` for every `@path`
//! reference not preceded by a word character or a backtick.

use regex::Regex;

/// Extract `@path` references from a markdown template.
pub fn files(template: &str) -> Vec<(String, String)> {
    let pattern = Regex::new(r"^\.?[^\s`,.]*(?:\.[^\s`,.]+)*").expect("valid path pattern");
    let bytes = template.as_bytes();
    let mut results = Vec::new();
    for (index, ch) in template.char_indices() {
        if ch != '@' {
            continue;
        }
        if index > 0 {
            let previous = bytes[index - 1] as char;
            if previous.is_alphanumeric() || previous == '_' || previous == '`' {
                continue;
            }
        }
        let start = index + 1;
        let Some(found) = pattern.find(&template[start..]) else {
            continue;
        };
        let path = found.as_str().to_string();
        results.push((format!("@{path}"), path));
    }
    results
}
