//! Unified-diff patch parsing and derivation.
//!
//! Ports the observable behaviour of `packages/core/src/patch.ts`: parse
//! add/update/delete hunks (optionally wrapped in a heredoc), derive fuzzy line
//! updates that preserve a BOM, and reject malformed bodies.

use crate::{CoreError, CoreResult};

/// A single change inside an update hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchChunk {
    /// Lines expected in the original file.
    pub old_lines: Vec<String>,
    /// Replacement lines.
    pub new_lines: Vec<String>,
    /// Optional `@@` section label.
    pub change_context: Option<String>,
    /// Whether the chunk anchors to end-of-file.
    pub end_of_file: Option<bool>,
}

/// One parsed hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchHunk {
    /// Create a file with `contents`.
    Add {
        /// Target path.
        path: String,
        /// File contents.
        contents: String,
    },
    /// Update a file.
    Update {
        /// Target path.
        path: String,
        /// Chunks to apply.
        chunks: Vec<PatchChunk>,
        /// Optional move destination.
        move_path: Option<String>,
    },
    /// Delete a file.
    Delete {
        /// Target path.
        path: String,
    },
}

/// Result of deriving an updated file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchUpdate {
    /// New file contents (BOM stripped).
    pub content: String,
    /// Whether the original had a BOM.
    pub bom: bool,
}

/// Patch parsing helpers.
#[derive(Debug, Default)]
pub struct Patch;

impl Patch {
    /// Parse a patch body into hunks.
    pub fn parse(input: &str) -> CoreResult<Vec<PatchHunk>> {
        let heredoc = strip_heredoc(input.trim());
        let lines: Vec<&str> = heredoc.split('\n').collect();
        let begin = lines
            .iter()
            .position(|line| line.trim() == "*** Begin Patch");
        let end = lines.iter().position(|line| line.trim() == "*** End Patch");
        let (begin, end) = match (begin, end) {
            (Some(begin), Some(end)) if begin < end => (begin, end),
            _ => {
                return Err(CoreError::Message(
                    "Invalid patch format: missing Begin/End markers".into(),
                ))
            }
        };

        let mut hunks = Vec::new();
        let mut index = begin + 1;
        while index < end {
            let line = lines[index];
            if let Some(rest) = line.strip_prefix("*** Add File:") {
                let path = rest.trim();
                if path.is_empty() {
                    return Err(CoreError::Message("Invalid add file path".into()));
                }
                let (contents, next) = parse_add(&lines, index + 1)?;
                hunks.push(PatchHunk::Add {
                    path: path.to_string(),
                    contents,
                });
                index = next;
                continue;
            }
            if let Some(rest) = line.strip_prefix("*** Delete File:") {
                let path = rest.trim();
                if path.is_empty() {
                    return Err(CoreError::Message("Invalid delete file path".into()));
                }
                hunks.push(PatchHunk::Delete {
                    path: path.to_string(),
                });
                index += 1;
                continue;
            }
            if let Some(rest) = line.strip_prefix("*** Update File:") {
                let path = rest.trim();
                if path.is_empty() {
                    return Err(CoreError::Message("Invalid update file path".into()));
                }
                let mut next = index + 1;
                let mut move_path = None;
                if let Some(move_line) = lines.get(next) {
                    if let Some(dest) = move_line.strip_prefix("*** Move to:") {
                        let dest = dest.trim();
                        if dest.is_empty() {
                            return Err(CoreError::Message("Invalid move file path".into()));
                        }
                        move_path = Some(dest.to_string());
                        next += 1;
                    }
                }
                let (chunks, parsed_next) = parse_update(&lines, next)?;
                if chunks.is_empty() {
                    return Err(CoreError::Message(format!(
                        "Invalid update hunk for {path}: expected at least one @@ chunk"
                    )));
                }
                hunks.push(PatchHunk::Update {
                    path: path.to_string(),
                    chunks,
                    move_path,
                });
                index = parsed_next;
                continue;
            }
            return Err(CoreError::Message(format!("Invalid patch line: {line}")));
        }
        Ok(hunks)
    }

    /// Derive updated contents by applying `chunks` to `old_content`.
    pub fn derive(
        path: &str,
        chunks: Vec<PatchChunk>,
        old_content: &str,
    ) -> CoreResult<PatchUpdate> {
        let (bom, text) = split_bom(old_content);
        let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
        if lines.last().map(String::as_str) == Some("") {
            lines.pop();
        }
        let replacements = compute_replacements(&lines, path, &chunks)?;
        let mut updated = lines.clone();
        for (start, remove, insert) in replacements.iter().rev() {
            updated.splice(*start..(*start + *remove), insert.iter().cloned());
        }
        if updated.last().map(String::as_str) != Some("") {
            updated.push(String::new());
        }
        let joined = updated.join("\n");
        let (next_bom, next_text) = split_bom(&joined);
        Ok(PatchUpdate {
            content: next_text.to_string(),
            bom: bom || next_bom,
        })
    }

    /// Re-attach a BOM to derived contents.
    pub fn join_bom(content: &str, bom: bool) -> CoreResult<String> {
        let (_, text) = split_bom(content);
        Ok(if bom {
            format!("\u{FEFF}{text}")
        } else {
            text.to_string()
        })
    }
}

fn parse_add(lines: &[&str], start: usize) -> CoreResult<(String, usize)> {
    let mut content = Vec::new();
    let mut index = start;
    while index < lines.len() && !lines[index].starts_with("***") {
        let line = lines[index];
        match line.strip_prefix('+') {
            Some(rest) => content.push(rest.to_string()),
            None => {
                return Err(CoreError::Message(format!("Invalid add file line: {line}")));
            }
        }
        index += 1;
    }
    Ok((content.join("\n"), index))
}

fn parse_update(lines: &[&str], start: usize) -> CoreResult<(Vec<PatchChunk>, usize)> {
    let mut chunks = Vec::new();
    let mut index = start;
    while index < lines.len() && !lines[index].starts_with("***") {
        let header = lines[index];
        let context = match header.strip_prefix("@@") {
            Some(rest) => rest.trim(),
            None => {
                return Err(CoreError::Message(format!(
                    "Invalid update file line: {header}"
                )))
            }
        };
        let change_context = if context.is_empty() {
            None
        } else {
            Some(context.to_string())
        };
        let mut old_lines = Vec::new();
        let mut new_lines = Vec::new();
        let mut end_of_file = false;
        index += 1;
        while index < lines.len() && !lines[index].starts_with("@@") {
            let line = lines[index];
            if line == "*** End of File" {
                end_of_file = true;
                index += 1;
                break;
            }
            if line.starts_with("***") {
                break;
            }
            if let Some(rest) = line.strip_prefix(' ') {
                old_lines.push(rest.to_string());
                new_lines.push(rest.to_string());
            } else if let Some(rest) = line.strip_prefix('-') {
                old_lines.push(rest.to_string());
            } else if let Some(rest) = line.strip_prefix('+') {
                new_lines.push(rest.to_string());
            } else {
                return Err(CoreError::Message(format!(
                    "Invalid update chunk line: {line}"
                )));
            }
            index += 1;
        }
        chunks.push(PatchChunk {
            old_lines,
            new_lines,
            change_context,
            end_of_file: if end_of_file { Some(true) } else { None },
        });
    }
    Ok((chunks, index))
}

type Replacement = (usize, usize, Vec<String>);

fn compute_replacements(
    lines: &[String],
    path: &str,
    chunks: &[PatchChunk],
) -> CoreResult<Vec<Replacement>> {
    let mut replacements: Vec<Replacement> = Vec::new();
    let mut line_index = 0usize;
    for chunk in chunks {
        if let Some(context) = &chunk.change_context {
            let context_lines = [context.clone()];
            let found = seek(lines, &context_lines, line_index, false);
            if found.is_none() {
                return Err(CoreError::Message(format!(
                    "Failed to find context '{context}' in {path}"
                )));
            }
            line_index = found.unwrap() + 1;
        }
        if chunk.old_lines.is_empty() {
            replacements.push((lines.len(), 0, chunk.new_lines.clone()));
            continue;
        }
        let mut old_lines = chunk.old_lines.clone();
        let mut new_lines = chunk.new_lines.clone();
        let eof = chunk.end_of_file.unwrap_or(false);
        let mut found = seek(lines, &old_lines, line_index, eof);
        if found.is_none() && old_lines.last().map(String::as_str) == Some("") {
            old_lines.pop();
            if new_lines.last().map(String::as_str) == Some("") {
                new_lines.pop();
            }
            found = seek(lines, &old_lines, line_index, eof);
        }
        let found = match found {
            Some(found) => found,
            None => {
                return Err(CoreError::Message(format!(
                    "Failed to find expected lines in {path}:\n{}",
                    chunk.old_lines.join("\n")
                )))
            }
        };
        replacements.push((found, old_lines.len(), new_lines));
        line_index = found + old_lines.len();
    }
    replacements.sort_by_key(|(start, _, _)| *start);
    Ok(replacements)
}

enum Compare {
    Exact,
    Rstrip,
    Trim,
    Normalized,
}

fn compare_values(compare: &Compare, left: &str, right: &str) -> bool {
    match compare {
        Compare::Exact => left == right,
        Compare::Rstrip => left.trim_end() == right.trim_end(),
        Compare::Trim => left.trim() == right.trim(),
        Compare::Normalized => normalize(left.trim()) == normalize(right.trim()),
    }
}

fn seek(lines: &[String], pattern: &[String], start: usize, eof: bool) -> Option<usize> {
    if pattern.is_empty() {
        return None;
    }
    let compares = [
        Compare::Exact,
        Compare::Rstrip,
        Compare::Trim,
        Compare::Normalized,
    ];
    for compare in &compares {
        if eof && lines.len() >= pattern.len() {
            let offset = lines.len() - pattern.len();
            if offset >= start && matches(lines, pattern, offset, compare) {
                return Some(offset);
            }
        }
        if lines.len() < pattern.len() {
            continue;
        }
        for offset in start..=(lines.len() - pattern.len()) {
            if matches(lines, pattern, offset, compare) {
                return Some(offset);
            }
        }
    }
    None
}

fn matches(lines: &[String], pattern: &[String], offset: usize, compare: &Compare) -> bool {
    pattern
        .iter()
        .enumerate()
        .all(|(index, line)| compare_values(compare, &lines[offset + index], line))
}

fn normalize(value: &str) -> String {
    value
        .replace(['\u{2018}', '\u{2019}', '\u{201A}', '\u{201B}'], "'")
        .replace(['\u{201C}', '\u{201D}', '\u{201E}', '\u{201F}'], "\"")
        .replace(
            [
                '\u{2010}', '\u{2011}', '\u{2012}', '\u{2013}', '\u{2014}', '\u{2015}',
            ],
            "-",
        )
        .replace('\u{2026}', "...")
        .replace('\u{00A0}', " ")
}

fn split_bom(text: &str) -> (bool, &str) {
    match text.strip_prefix('\u{FEFF}') {
        Some(rest) => (true, rest),
        None => (false, text),
    }
}

fn strip_heredoc(input: &str) -> String {
    let mut lines = input.split('\n');
    let Some(first) = lines.next() else {
        return input.to_string();
    };
    let rest: Vec<&str> = lines.collect();
    let Some(last) = rest.last() else {
        return input.to_string();
    };
    let marker = match heredoc_marker(first) {
        Some(marker) => marker,
        None => return input.to_string(),
    };
    if last.trim() != marker {
        return input.to_string();
    }
    rest[..rest.len() - 1].join("\n")
}

fn heredoc_marker(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let trimmed = trimmed.strip_prefix("cat ").unwrap_or(trimmed);
    let rest = trimmed.strip_prefix("<<")?;
    let rest = rest.trim_start();
    let quote = rest.chars().next();
    let (quote_char, body) = match quote {
        Some(ch) if ch == '\'' || ch == '"' => (Some(ch), &rest[1..]),
        _ => (None, rest),
    };
    let name: String = body
        .chars()
        .take_while(|ch| ch.is_alphanumeric() || *ch == '_')
        .collect();
    if name.is_empty() {
        return None;
    }
    let after = &body[name.len()..];
    if let Some(quote_char) = quote_char {
        if !after.trim_start().starts_with(quote_char) {
            return None;
        }
    }
    Some(name)
}
