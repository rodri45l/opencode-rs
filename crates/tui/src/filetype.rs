//! Presentation language lookup.
//!
//! Port of packages/tui/src/util/filetype.ts behaviour (upstream 18ef3cc).

/// The presentation language for a filename, or `None` when unknown.
///
/// Missing or empty filenames report `"none"`; script/JSX languages collapse to
/// `"typescript"` exactly as the reference does.
pub fn filetype(input: Option<&str>) -> Option<&'static str> {
    let input = match input {
        Some(value) if !value.is_empty() => value,
        _ => return Some("none"),
    };
    let extension = std::path::Path::new(input)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let language = match extension.as_str() {
        "tsx" | "jsx" | "ts" | "mts" | "cts" | "js" | "mjs" | "cjs" => "typescript",
        "py" => "python",
        "rs" => "rust",
        "go" => "go",
        "md" | "markdown" => "markdown",
        "json" => "json",
        "css" => "css",
        "html" | "htm" => "html",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "sh" | "bash" | "zsh" => "shellscript",
        "sql" => "sql",
        "svelte" => "svelte",
        "vue" => "vue",
        _ => return None,
    };
    Some(language)
}
