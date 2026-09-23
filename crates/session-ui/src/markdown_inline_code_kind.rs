//! Inline-code classification for the markdown renderer.
//!
//! Port of packages/session-ui/src/components/markdown-inline-code-kind.ts
//! behaviour (upstream 18ef3cc).

/// The kind of an inline code span, if it is special.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineCodeKind {
    Path,
    Url,
}

impl InlineCodeKind {
    /// The wire string used by the reference (`"path"` / `"url"`).
    pub fn as_str(self) -> &'static str {
        match self {
            InlineCodeKind::Path => "path",
            InlineCodeKind::Url => "url",
        }
    }
}

const KNOWN_FILENAMES: &[&str] = &[
    "Dockerfile",
    ".gitignore",
    "Cargo.lock",
    "go.sum",
    "bun.lockb",
    "terraform.tfvars",
    "pnpm-lock.yaml",
];

const KNOWN_EXTENSIONS: &[&str] = &[
    "ts", "tsx", "mts", "cts", "mjs", "cjs", "js", "jsx", "svelte", "graphql", "gql", "json",
    "jsonc", "md", "mdx", "css", "scss", "html", "py", "rb", "go", "rs", "java", "kt", "swift",
    "c", "h", "cc", "cpp", "hpp", "sh", "bash", "zsh", "fish", "ps1", "sql", "toml", "yaml", "yml",
    "tf", "tfvars", "lock", "sum", "lockb", "txt", "env",
];

/// Classify an inline code span as a path or URL, or `None` for normal code.
pub fn inline_code_kind(code: &str) -> Option<InlineCodeKind> {
    if code.chars().any(char::is_whitespace) {
        return None;
    }
    if code.contains("://") {
        return if code.starts_with("http://") || code.starts_with("https://") {
            Some(InlineCodeKind::Url)
        } else {
            None
        };
    }
    if code.contains('/') {
        return Some(InlineCodeKind::Path);
    }
    if code == "Dockerfile" || code.starts_with("Dockerfile.") {
        return Some(InlineCodeKind::Path);
    }
    if KNOWN_FILENAMES.contains(&code) {
        return Some(InlineCodeKind::Path);
    }
    if let Some(extension) = std::path::Path::new(code)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        if KNOWN_EXTENSIONS
            .iter()
            .any(|known| known.eq_ignore_ascii_case(extension))
        {
            return Some(InlineCodeKind::Path);
        }
    }
    None
}
