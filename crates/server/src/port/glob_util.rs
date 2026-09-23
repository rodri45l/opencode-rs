//! Glob pattern matching and filesystem scanning.
//!
//! Re-derived from the observable behaviour pinned by
//! `packages/opencode/test/util/glob.test.ts` (upstream 18ef3cc): `match`,
//! `scan`/`scanSync` with `cwd`, `absolute`, `include`, `symlink`, and `dot`
//! options. The glob is compiled to a regex; brace alternatives are expanded
//! structurally so no shell is involved.

use std::fs;
use std::path::{Path, PathBuf};

/// Options accepted by [`scan`] and [`scan_sync`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanOptions {
    /// Directory the pattern is resolved against.
    pub cwd: Option<String>,
    /// Return absolute paths instead of paths relative to `cwd`.
    pub absolute: bool,
    /// `"file"` (default) excludes directories; `"all"` includes them.
    pub include: Option<String>,
    /// Follow directory symlinks while walking.
    pub symlink: bool,
    /// Include dotfiles and dot-directories.
    pub dot: bool,
}

/// Return `true` when `path` matches the glob `pattern`.
///
/// `*` and `?` do not cross `/`; `**` does. Brace groups such as `{js,ts}`
/// expand to alternatives. Unlike a POSIX shell, a leading dot is matched by
/// `*` (the reference matcher is used for already-enumerated paths).
pub fn glob_match(pattern: &str, path: &str) -> bool {
    compile(pattern).is_match(path)
}

/// Scan `cwd` recursively for paths matching `pattern` (blocking).
pub fn scan_sync(pattern: &str, options: &ScanOptions) -> Vec<String> {
    let cwd = options
        .cwd
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let include_all = options.include.as_deref() == Some("all");
    let mut results = Vec::new();
    if cwd.is_dir() {
        walk(&cwd, &cwd, pattern, options, include_all, &mut results);
    }
    results.sort();
    results
}

/// Scan `cwd` recursively for paths matching `pattern`.
///
/// The reference API is async; the ported behaviour is pure filesystem work, so
/// this is a synchronous wrapper over [`scan_sync`].
pub fn scan(pattern: &str, options: &ScanOptions) -> Vec<String> {
    scan_sync(pattern, options)
}

fn walk(
    dir: &Path,
    cwd: &Path,
    pattern: &str,
    options: &ScanOptions,
    include_all: bool,
    out: &mut Vec<String>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !options.dot && file_name.starts_with('.') {
            continue;
        }
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        let is_symlink = meta.file_type().is_symlink();
        let is_dir = if is_symlink {
            options.symlink && fs::metadata(&path).map(|m| m.is_dir()).unwrap_or(false)
        } else {
            meta.is_dir()
        };
        let rel = path
            .strip_prefix(cwd)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if is_dir {
            if include_all && glob_match(pattern, &rel) {
                out.push(render(&path, &rel, options.absolute));
            }
            if !is_symlink || options.symlink {
                walk(&path, cwd, pattern, options, include_all, out);
            }
        } else if glob_match(pattern, &rel) {
            out.push(render(&path, &rel, options.absolute));
        }
    }
}

fn render(path: &Path, rel: &str, absolute: bool) -> String {
    if absolute {
        path.to_string_lossy().to_string()
    } else {
        rel.to_string()
    }
}

fn compile(pattern: &str) -> regex::Regex {
    let mut out = String::from("^");
    build(pattern, &mut out);
    out.push('$');
    regex::Regex::new(&out).unwrap_or_else(|_| regex::Regex::new("$^").expect("fallback regex"))
}

fn build(pattern: &str, out: &mut String) {
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' => {
                if i + 1 < chars.len() && chars[i + 1] == '*' {
                    i += 2;
                    if i < chars.len() && chars[i] == '/' {
                        out.push_str("(?:.*/)?");
                        i += 1;
                    } else {
                        out.push_str(".*");
                    }
                } else {
                    out.push_str("[^/]*");
                    i += 1;
                }
            }
            '?' => {
                out.push_str("[^/]");
                i += 1;
            }
            '[' => {
                out.push('[');
                i += 1;
                if i < chars.len() && (chars[i] == '!' || chars[i] == '^') {
                    out.push('^');
                    i += 1;
                }
                while i < chars.len() && chars[i] != ']' {
                    out.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() {
                    out.push(']');
                    i += 1;
                }
            }
            '{' => {
                let start = i + 1;
                let mut depth = 1usize;
                let mut j = start;
                while j < chars.len() {
                    match chars[j] {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                let inner: String = chars[start..j.min(chars.len())].iter().collect();
                let alternatives = split_top_level(&inner);
                out.push_str("(?:");
                for (index, alt) in alternatives.iter().enumerate() {
                    if index > 0 {
                        out.push('|');
                    }
                    build(alt, out);
                }
                out.push(')');
                i = (j + 1).min(chars.len());
            }
            c => {
                out.push_str(&regex::escape(&c.to_string()));
                i += 1;
            }
        }
    }
}

fn split_top_level(inner: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    for c in inner.chars() {
        match c {
            '{' => {
                depth += 1;
                current.push(c);
            }
            '}' => {
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => {
                parts.push(std::mem::take(&mut current));
            }
            _ => current.push(c),
        }
    }
    parts.push(current);
    parts
}
