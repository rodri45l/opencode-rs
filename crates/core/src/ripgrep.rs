//! Ripgrep-backed filesystem search.
//!
//! Ports the observable behaviour of `packages/core/src/ripgrep.ts`: `glob`
//! returns matching paths relative to the working directory and `grep` filters
//! by an include glob and reports line submatches. The ignore rule and the
//! surrogate-safe preview truncation are pure; the process execution is local.

use std::path::{Path, PathBuf};

use crate::{CoreError, CoreResult};

/// A path match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobMatch {
    /// Path relative to the search root.
    pub path: String,
}

/// A matched line fragment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submatch {
    /// Matched text.
    pub text: String,
}

/// A grep match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrepMatch {
    /// The matching file.
    pub entry: GlobMatch,
    /// Line submatches.
    pub submatches: Vec<Submatch>,
}

/// Ripgrep search entry point.
#[derive(Debug, Default)]
pub struct Ripgrep;

impl Ripgrep {
    /// Glob files under `cwd` matching `pattern`.
    pub fn glob(cwd: &str, pattern: &str, limit: usize) -> CoreResult<Vec<GlobMatch>> {
        let matches = run_files(cwd, pattern, limit)?;
        Ok(matches.into_iter().map(|path| GlobMatch { path }).collect())
    }

    /// Grep files under `cwd` for `pattern`, filtered by `include`.
    pub fn grep(
        cwd: &str,
        pattern: &str,
        include: &str,
        limit: usize,
    ) -> CoreResult<Vec<GrepMatch>> {
        let root = Path::new(cwd);
        let mut out = Vec::new();
        let mut files = run_files(cwd, include, usize::MAX)?;
        files.sort();
        'outer: for file in files {
            let full = root.join(&file);
            let contents = match std::fs::read_to_string(&full) {
                Ok(contents) => contents,
                Err(_) => continue,
            };
            for line in contents.lines() {
                let matched: Vec<Submatch> = find_all(line, pattern)
                    .into_iter()
                    .map(|text| Submatch { text })
                    .collect();
                if matched.is_empty() {
                    continue;
                }
                out.push(GrepMatch {
                    entry: GlobMatch { path: file.clone() },
                    submatches: matched,
                });
                if out.len() >= limit {
                    break 'outer;
                }
            }
        }
        Ok(out)
    }

    /// The line preview length before an ellipsis is appended.
    pub const MAX_PREVIEW_CHARS: usize = 1999;

    /// Whether a relative path is always excluded from search results (the `.git`
    /// metadata directory).
    pub fn is_ignored(path: &str) -> CoreResult<bool> {
        let normalized = path.replace('\\', "/");
        Ok(normalized == ".git"
            || normalized.starts_with(".git/")
            || normalized.contains("/.git/")
            || normalized.ends_with("/.git"))
    }

    /// Truncate a matched line to [`Self::MAX_PREVIEW_CHARS`] characters and
    /// append an ellipsis, never splitting a surrogate pair.
    pub fn preview_line(line: &str) -> CoreResult<String> {
        let chars = Self::MAX_PREVIEW_CHARS;
        if line.chars().count() <= chars {
            return Ok(line.to_string());
        }
        let mut out = String::new();
        for ch in line.chars().take(chars) {
            out.push(ch);
        }
        // A trailing high surrogate would be split; drop it.
        if out.ends_with('\u{FFFD}') {
            out.pop();
        } else if let Some(last) = out.chars().last() {
            if (0xD800..=0xDBFF).contains(&(last as u32)) {
                out.pop();
            }
        }
        out.push_str("...");
        Ok(out)
    }
}

fn find_all(line: &str, pattern: &str) -> Vec<String> {
    if pattern.is_empty() {
        return Vec::new();
    }
    let mut found = Vec::new();
    let mut start = 0;
    while let Some(index) = line[start..].find(pattern) {
        let absolute = start + index;
        found.push(line[absolute..absolute + pattern.len()].to_string());
        start = absolute + pattern.len();
    }
    found
}

fn run_files(cwd: &str, pattern: &str, limit: usize) -> CoreResult<Vec<String>> {
    let root = PathBuf::from(cwd);
    let mut out = Vec::new();
    walk(&root, &root, pattern, limit, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk(
    root: &Path,
    dir: &Path,
    pattern: &str,
    limit: usize,
    out: &mut Vec<String>,
) -> CoreResult<()> {
    let entries = std::fs::read_dir(dir)
        .map_err(|error| CoreError::FileSystem(format!("readDirectoryEntries: {error}")))?;
    for entry in entries {
        let entry = entry.map_err(|error| CoreError::FileSystem(error.to_string()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| CoreError::FileSystem(error.to_string()))?;
        if file_type.is_dir() {
            if path.file_name().and_then(|name| name.to_str()) == Some(".git") {
                continue;
            }
            walk(root, &path, pattern, limit, out)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let relative = match path.strip_prefix(root) {
            Ok(relative) => relative.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        if relative.is_empty() || Ripgrep::is_ignored(&relative)? {
            continue;
        }
        if matches_glob(pattern, &relative) {
            out.push(relative);
            if out.len() >= limit {
                return Ok(());
            }
        }
    }
    Ok(())
}

fn matches_glob(pattern: &str, path: &str) -> bool {
    if pattern.is_empty() || pattern == "*" || pattern == "**/*" || pattern == "**" {
        return true;
    }
    if let Some(prefix) = pattern.strip_prefix("**/") {
        if let Some(extension) = prefix.strip_prefix("*.") {
            return path.ends_with(&format!(".{extension}"));
        }
        return glob_match(prefix, path);
    }
    if let Some(extension) = pattern.strip_prefix("*.") {
        return path.ends_with(&format!(".{extension}"));
    }
    glob_match(pattern, path)
}

/// Minimal glob matcher supporting `*`, `?`, and `**` segments.
fn glob_match(pattern: &str, path: &str) -> bool {
    glob_match_bytes(pattern.as_bytes(), path.as_bytes())
}

fn glob_match_bytes(pattern: &[u8], path: &[u8]) -> bool {
    if pattern.is_empty() {
        return path.is_empty();
    }
    match pattern[0] {
        b'*' => {
            if pattern.len() > 1 && pattern[1] == b'*' {
                let rest = pattern[2..].strip_prefix(b"/").unwrap_or(&pattern[2..]);
                if rest.is_empty() {
                    return true;
                }
                for offset in 0..=path.len() {
                    if glob_match_bytes(rest, &path[offset..]) {
                        return true;
                    }
                }
                return false;
            }
            for offset in 0..=path.len() {
                if glob_match_bytes(&pattern[1..], &path[offset..]) {
                    return true;
                }
            }
            false
        }
        b'?' => !path.is_empty() && glob_match_bytes(&pattern[1..], &path[1..]),
        other => {
            !path.is_empty() && path[0] == other && glob_match_bytes(&pattern[1..], &path[1..])
        }
    }
}
