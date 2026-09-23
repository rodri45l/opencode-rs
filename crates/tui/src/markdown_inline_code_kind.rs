//! Inline code classification.
//!
//! Derived from `packages/session-ui/src/components/markdown-inline-code-kind.ts`
//! (upstream 18ef3cc): inline code that is a file/directory path or an http(s)
//! URL is highlighted; code expressions and other schemes stay normal.

use std::fmt;

/// Error raised by the classifier.
#[derive(Debug, PartialEq, Eq)]
pub struct NotImplemented(pub &'static str);

impl fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for NotImplemented {}

/// Result alias for the classifier.
pub type PortResult<T> = Result<T, NotImplemented>;

/// Message used by the ported tests when expecting success.
pub const NOTE: &str = "session-ui inline code kind";

/// The highlight kind for inline code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineCodeKind {
    Path,
    Url,
}

const PATH_EXTENSIONS: &[&str] = &[
    "ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs", "json", "jsonc", "json5", "md", "mdx",
    "css", "scss", "sass", "less", "html", "htm", "svelte", "vue", "astro", "graphql", "graphqls",
    "gql", "lock", "lockb", "sum", "yaml", "yml", "toml", "tf", "tfvars", "rs", "go", "py", "rb",
    "java", "kt", "kts", "c", "h", "cc", "cpp", "cxx", "hpp", "cs", "php", "sh", "bash", "zsh",
    "fish", "sql", "swift", "dart", "lua", "pl", "pm", "r", "scala", "clj", "cljs", "ex", "exs",
    "erl", "hs", "ml", "nim", "zig", "wasm", "proto", "txt", "env", "cfg", "ini", "conf", "log",
    "diff", "patch", "svg", "png", "jpg", "jpeg", "gif", "webp", "ico", "woff", "woff2", "ttf",
    "eot", "map", "d",
];

const PATH_FILE_NAMES: &[&str] = &[
    ".gitignore",
    ".gitattributes",
    ".gitmodules",
    ".dockerignore",
    ".npmrc",
    ".nvmrc",
    ".prettierrc",
    ".eslintrc",
    ".editorconfig",
    ".env",
    ".bashrc",
    ".zshrc",
    ".vimrc",
    "dockerfile",
    "makefile",
    "cargo.lock",
    "go.mod",
    "go.sum",
    "pnpm-lock.yaml",
    "package.json",
    "tsconfig.json",
    "readme",
    "license",
];

const PATH_FILE_NAME_PREFIXES: &[&str] = &[
    "dockerfile",
    "makefile",
    "license",
    "readme",
    ".env",
    ".git",
    "tsconfig",
    "cargo",
];

fn has_path_extension(text: &str) -> bool {
    let value = text.to_ascii_lowercase();
    if value.ends_with(".d.ts") {
        return true;
    }
    match value.rfind('.') {
        Some(index) => PATH_EXTENSIONS.contains(&&value[index + 1..]),
        None => false,
    }
}

fn has_path_file_name(text: &str) -> bool {
    let value = text.to_ascii_lowercase();
    if PATH_FILE_NAMES.contains(&value.as_str()) {
        return true;
    }
    match value.find('.') {
        Some(index) => PATH_FILE_NAME_PREFIXES.contains(&&value[..index]),
        None => false,
    }
}

fn is_scheme_url(text: &str) -> bool {
    let Some(index) = text.find("://") else {
        return false;
    };
    let scheme = &text[..index];
    let mut chars = scheme.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '.' || c == '-')
}

fn is_slash_command(text: &str) -> bool {
    let Some(rest) = text.strip_prefix('/') else {
        return false;
    };
    if rest.is_empty() {
        return false;
    }
    let mut chars = rest.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// Classify one inline code span.
pub fn inline_code_kind(input: &str) -> PortResult<Option<InlineCodeKind>> {
    let lower = input.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return Ok(Some(InlineCodeKind::Url));
    }
    if is_scheme_url(input) {
        return Ok(None);
    }
    if input == "/" || is_slash_command(input) {
        return Ok(None);
    }
    if input.chars().any(char::is_whitespace) {
        return Ok(None);
    }
    if input.chars().any(|c| "()[]{}*+=<>|&^\"';".contains(c)) {
        return Ok(None);
    }
    if input.contains('/')
        || input.contains('\\')
        || input.starts_with('.')
        || has_path_extension(input)
        || has_path_file_name(input)
    {
        return Ok(Some(InlineCodeKind::Path));
    }
    Ok(None)
}
