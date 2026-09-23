//! Glob-style wildcard matching for permission patterns.
//!
//! Ports the observable behaviour of `packages/opencode/src/util/wildcard.ts`:
//! `?` matches a single character, `*` matches any run, a trailing ` *` makes
//! the argument tail optional, and rules are evaluated most-specific-first with
//! the last match winning. Backslashes are normalised to `/` so patterns are
//! portable across platforms.

use regex::Regex;

/// Match `value` against a single glob `pattern`.
pub fn match_pattern(value: &str, pattern: &str) -> bool {
    let value = value.replace('\\', "/");
    let pattern = pattern.replace('\\', "/");
    let mut escaped = String::new();
    for ch in pattern.chars() {
        match ch {
            '.' | '+' | '^' | '$' | '{' | '}' | '(' | ')' | '|' | '[' | ']' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            '*' => escaped.push_str(".*"),
            '?' => escaped.push('.'),
            other => escaped.push(other),
        }
    }
    if let Some(base) = escaped.strip_suffix(" .*") {
        escaped = format!("{base}( .*)?");
    }
    let flags = if cfg!(windows) { "(?is)" } else { "(?s)" };
    Regex::new(&format!("^{flags}{escaped}$"))
        .expect("valid wildcard pattern")
        .is_match(&value)
}

/// Return the action of the most specific pattern matching `value`.
pub fn all(value: &str, rules: &[(&str, &str)]) -> Option<String> {
    let mut sorted: Vec<&(&str, &str)> = rules.iter().collect();
    sorted.sort_by(|a, b| a.0.len().cmp(&b.0.len()).then_with(|| a.0.cmp(b.0)));
    let mut result = None;
    for (pattern, action) in sorted {
        if match_pattern(value, pattern) {
            result = Some((*action).to_string());
        }
    }
    result
}

/// Return the action of the most specific structured command rule matching
/// `head` plus its argument `tail`.
pub fn all_structured(head: &str, tail: &[&str], rules: &[(&str, &str)]) -> Option<String> {
    let mut sorted: Vec<&(&str, &str)> = rules.iter().collect();
    sorted.sort_by(|a, b| a.0.len().cmp(&b.0.len()).then_with(|| a.0.cmp(b.0)));
    let mut result = None;
    for (pattern, action) in sorted {
        let parts: Vec<&str> = pattern.split_whitespace().collect();
        let Some((first, rest)) = parts.split_first() else {
            continue;
        };
        if !match_pattern(head, first) {
            continue;
        }
        if rest.is_empty() || match_sequence(tail, rest) {
            result = Some((*action).to_string());
        }
    }
    result
}

fn match_sequence(items: &[&str], patterns: &[&str]) -> bool {
    let Some((pattern, rest)) = patterns.split_first() else {
        return true;
    };
    if *pattern == "*" {
        return match_sequence(items, rest);
    }
    for index in 0..items.len() {
        if match_pattern(items[index], pattern) && match_sequence(&items[index + 1..], rest) {
            return true;
        }
    }
    false
}
