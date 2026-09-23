//! Freeform `apply_patch` parsing and application.
//!
//! Ports the observable behaviour of `packages/opencode/src/patch/index.ts` and
//! `packages/opencode/src/tool/apply_patch.ts`: `*** Begin Patch`/`*** End
//! Patch` framing, add/update/delete/move hunks, context seeking with exact,
//! right-trim, trim and Unicode-normalised passes, and permission metadata.

use crate::port::tools::unified_diff;
use crate::tools::{PermissionRequest, ToolContext, ToolError, ToolResult};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
enum RawHunk {
    Add {
        path: String,
        contents: String,
    },
    Delete {
        path: String,
    },
    Update {
        path: String,
        move_path: Option<String>,
        chunks: Vec<UpdateChunk>,
    },
}

#[derive(Debug, Clone)]
struct UpdateChunk {
    old_lines: Vec<String>,
    new_lines: Vec<String>,
    change_context: Option<String>,
    is_end_of_file: bool,
}

/// Parse and apply a freeform patch against the instance directory.
pub fn apply(patch_text: &str, ctx: &mut ToolContext) -> Result<ToolResult, ToolError> {
    if patch_text.is_empty() {
        return Err(ToolError::Message("patchText is required".to_string()));
    }

    let hunks = parse_raw_patch(patch_text)
        .map_err(|error| ToolError::Message(format!("apply_patch verification failed: {error}")))?;

    if hunks.is_empty() {
        let normalized = patch_text.replace("\r\n", "\n").replace('\r', "\n");
        if normalized.trim() == "*** Begin Patch\n*** End Patch" {
            return Err(ToolError::Message(
                "patch rejected: empty patch".to_string(),
            ));
        }
        return Err(ToolError::Message(
            "apply_patch verification failed: no hunks found".to_string(),
        ));
    }

    struct FileChange {
        file_path: PathBuf,
        relative_path: String,
        kind: &'static str,
        move_path: Option<PathBuf>,
        new_content: String,
        bom: bool,
        diff: String,
        additions: usize,
        deletions: usize,
    }

    let mut changes: Vec<FileChange> = Vec::new();
    let mut total_diff = String::new();

    for hunk in hunks {
        match hunk {
            RawHunk::Add { path, contents } => {
                let file_path = resolve(&ctx.directory, &path);
                crate::tools::assert_external_directory(
                    ctx,
                    Some(&file_path.to_string_lossy()),
                    &Default::default(),
                )?;
                let (bom, text) = split_bom(&contents);
                let new_content = if text.is_empty() || text.ends_with('\n') {
                    text
                } else {
                    format!("{text}\n")
                };
                let diff = unified_diff(&file_path.to_string_lossy(), "", &new_content);
                changes.push(FileChange {
                    relative_path: relative(&ctx.directory, &file_path),
                    file_path,
                    kind: "add",
                    move_path: None,
                    additions: new_content.lines().count(),
                    deletions: 0,
                    new_content,
                    bom,
                    diff,
                });
            }
            RawHunk::Delete { path } => {
                let file_path = resolve(&ctx.directory, &path);
                crate::tools::assert_external_directory(
                    ctx,
                    Some(&file_path.to_string_lossy()),
                    &Default::default(),
                )?;
                let raw = fs::read_to_string(&file_path).map_err(|error| {
                    ToolError::Message(format!("apply_patch verification failed: {error}"))
                })?;
                let (bom, text) = split_bom(&raw);
                let diff = unified_diff(&file_path.to_string_lossy(), &text, "");
                changes.push(FileChange {
                    relative_path: relative(&ctx.directory, &file_path),
                    file_path,
                    kind: "delete",
                    move_path: None,
                    additions: 0,
                    deletions: text.split('\n').count(),
                    new_content: String::new(),
                    bom,
                    diff,
                });
            }
            RawHunk::Update {
                path,
                move_path,
                chunks,
            } => {
                let file_path = resolve(&ctx.directory, &path);
                crate::tools::assert_external_directory(
                    ctx,
                    Some(&file_path.to_string_lossy()),
                    &Default::default(),
                )?;
                if !file_path.is_file() {
                    return Err(ToolError::Message(format!(
                        "apply_patch verification failed: Failed to read file to update: {}",
                        file_path.to_string_lossy()
                    )));
                }
                let raw = fs::read_to_string(&file_path).map_err(|error| {
                    ToolError::Message(format!("apply_patch verification failed: {error}"))
                })?;
                let (source_bom, old_text) = split_bom(&raw);
                let new_content =
                    derive_new_contents(&file_path, &chunks, &old_text).map_err(|error| {
                        ToolError::Message(format!("apply_patch verification failed: {error}"))
                    })?;
                let diff = unified_diff(&file_path.to_string_lossy(), &old_text, &new_content);
                let move_path = move_path.map(|target| resolve(&ctx.directory, &target));
                if let Some(target) = &move_path {
                    crate::tools::assert_external_directory(
                        ctx,
                        Some(&target.to_string_lossy()),
                        &Default::default(),
                    )?;
                }
                let relative_path =
                    relative(&ctx.directory, move_path.as_ref().unwrap_or(&file_path));
                changes.push(FileChange {
                    relative_path,
                    file_path,
                    kind: if move_path.is_some() {
                        "move"
                    } else {
                        "update"
                    },
                    move_path,
                    additions: diff
                        .lines()
                        .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
                        .count(),
                    deletions: diff
                        .lines()
                        .filter(|line| line.starts_with('-') && !line.starts_with("---"))
                        .count(),
                    new_content,
                    bom: source_bom,
                    diff,
                });
            }
        }
    }

    for change in &changes {
        total_diff.push_str(&change.diff);
        total_diff.push('\n');
    }

    let files: Vec<serde_json::Value> = changes
        .iter()
        .map(|change| {
            let mut value = json!({
                "filePath": change.file_path.to_string_lossy(),
                "relativePath": change.relative_path,
                "type": change.kind,
                "patch": change.diff,
                "additions": change.additions,
                "deletions": change.deletions,
            });
            if let Some(target) = &change.move_path {
                value["movePath"] = json!(target.to_string_lossy());
            }
            value
        })
        .collect();

    let relative_paths: Vec<String> = changes
        .iter()
        .map(|change| relative(&ctx.directory, &change.file_path))
        .collect();
    ctx.ask(PermissionRequest {
        permission: "edit".to_string(),
        patterns: relative_paths.clone(),
        always: vec!["*".to_string()],
        metadata: json!({
            "filepath": relative_paths.join(", "),
            "diff": total_diff,
            "files": files,
        }),
    });

    for change in &changes {
        match change.kind {
            "delete" => {
                fs::remove_file(&change.file_path)
                    .map_err(|error| ToolError::Message(error.to_string()))?;
            }
            "move" => {
                let target = change.move_path.as_ref().expect("move target");
                write_with_dirs(target, &join_bom(&change.new_content, change.bom))?;
                fs::remove_file(&change.file_path)
                    .map_err(|error| ToolError::Message(error.to_string()))?;
            }
            _ => {
                write_with_dirs(
                    &change.file_path,
                    &join_bom(&change.new_content, change.bom),
                )?;
            }
        }
    }

    let summary_lines: Vec<String> = changes
        .iter()
        .map(|change| match change.kind {
            "add" => format!("A {}", change.relative_path),
            "delete" => format!("D {}", change.relative_path),
            _ => format!("M {}", change.relative_path),
        })
        .collect();
    let output = format!(
        "Success. Updated the following files:\n{}",
        summary_lines.join("\n")
    );

    Ok(ToolResult {
        title: output.clone(),
        output,
        metadata: json!({ "diff": total_diff, "files": files }),
        attachments: None,
    })
}

fn parse_raw_patch(patch_text: &str) -> Result<Vec<RawHunk>, String> {
    let cleaned = patch_text.trim().to_string();
    let lines: Vec<&str> = cleaned.split('\n').collect();
    let begin = lines
        .iter()
        .position(|line| line.trim() == "*** Begin Patch");
    let end = lines.iter().position(|line| line.trim() == "*** End Patch");
    let (Some(begin), Some(end)) = (begin, end) else {
        return Err("Invalid patch format: missing Begin/End markers".to_string());
    };
    if begin >= end {
        return Err("Invalid patch format: missing Begin/End markers".to_string());
    }

    let mut hunks = Vec::new();
    let mut i = begin + 1;
    while i < end {
        let line = lines[i];
        if let Some(rest) = line.strip_prefix("*** Add File:") {
            let path = rest.trim().to_string();
            if path.is_empty() {
                i += 1;
                continue;
            }
            let (contents, next) = parse_add_content(&lines, i + 1);
            hunks.push(RawHunk::Add { path, contents });
            i = next;
        } else if let Some(rest) = line.strip_prefix("*** Delete File:") {
            let path = rest.trim().to_string();
            hunks.push(RawHunk::Delete { path });
            i += 1;
        } else if let Some(rest) = line.strip_prefix("*** Update File:") {
            let path = rest.trim().to_string();
            let mut move_path = None;
            let mut next = i + 1;
            if next < end && lines[next].starts_with("*** Move to:") {
                move_path = lines[next]
                    .strip_prefix("*** Move to:")
                    .map(|rest| rest.trim().to_string());
                next += 1;
            }
            let (chunks, after) = parse_update_chunks(&lines, next);
            hunks.push(RawHunk::Update {
                path,
                move_path,
                chunks,
            });
            i = after;
        } else {
            i += 1;
        }
    }
    Ok(hunks)
}

fn parse_add_content(lines: &[&str], start: usize) -> (String, usize) {
    let mut content = String::new();
    let mut i = start;
    while i < lines.len() && !lines[i].starts_with("***") {
        if let Some(rest) = lines[i].strip_prefix('+') {
            content.push_str(rest);
            content.push('\n');
        }
        i += 1;
    }
    if content.ends_with('\n') {
        content.pop();
    }
    (content, i)
}

fn parse_update_chunks(lines: &[&str], start: usize) -> (Vec<UpdateChunk>, usize) {
    let mut chunks = Vec::new();
    let mut i = start;
    while i < lines.len() && !lines[i].starts_with("***") {
        if lines[i].starts_with("@@") {
            let context = lines[i][2..].trim().to_string();
            i += 1;
            let mut old_lines = Vec::new();
            let mut new_lines = Vec::new();
            let mut is_end_of_file = false;
            while i < lines.len() && !lines[i].starts_with("@@") && !lines[i].starts_with("***") {
                let change = lines[i];
                if change == "*** End of File" {
                    is_end_of_file = true;
                    i += 1;
                    break;
                }
                if let Some(rest) = change.strip_prefix(' ') {
                    old_lines.push(rest.to_string());
                    new_lines.push(rest.to_string());
                } else if let Some(rest) = change.strip_prefix('-') {
                    old_lines.push(rest.to_string());
                } else if let Some(rest) = change.strip_prefix('+') {
                    new_lines.push(rest.to_string());
                }
                i += 1;
            }
            chunks.push(UpdateChunk {
                old_lines,
                new_lines,
                change_context: if context.is_empty() {
                    None
                } else {
                    Some(context)
                },
                is_end_of_file,
            });
        } else {
            i += 1;
        }
    }
    (chunks, i)
}

fn derive_new_contents(
    file_path: &Path,
    chunks: &[UpdateChunk],
    old_text: &str,
) -> Result<String, String> {
    let mut original_lines: Vec<String> =
        old_text.split('\n').map(|line| line.to_string()).collect();
    if original_lines.last().map(String::is_empty).unwrap_or(false) {
        original_lines.pop();
    }

    let replacements = compute_replacements(&original_lines, file_path, chunks)?;
    let mut new_lines = apply_replacements(&original_lines, &replacements);
    if new_lines
        .last()
        .map(|line| !line.is_empty())
        .unwrap_or(true)
    {
        new_lines.push(String::new());
    }
    Ok(new_lines.join("\n"))
}

type Replacement = (usize, usize, Vec<String>);

fn compute_replacements(
    original_lines: &[String],
    file_path: &Path,
    chunks: &[UpdateChunk],
) -> Result<Vec<Replacement>, String> {
    let mut replacements: Vec<Replacement> = Vec::new();
    let mut line_index = 0usize;

    for chunk in chunks {
        if let Some(context) = &chunk.change_context {
            let context_index = seek_sequence(
                original_lines,
                std::slice::from_ref(context),
                line_index,
                false,
            )
            .ok_or_else(|| {
                format!(
                    "Failed to find context '{context}' in {}",
                    file_path.display()
                )
            })?;
            line_index = context_index + 1;
        }

        if chunk.old_lines.is_empty() {
            let insertion = if original_lines.last().map(String::is_empty).unwrap_or(false) {
                original_lines.len() - 1
            } else {
                original_lines.len()
            };
            replacements.push((insertion, 0, chunk.new_lines.clone()));
            continue;
        }

        let mut pattern = chunk.old_lines.clone();
        let mut new_slice = chunk.new_lines.clone();
        let mut found = seek_sequence(original_lines, &pattern, line_index, chunk.is_end_of_file);

        if found.is_none() && pattern.last().map(String::is_empty).unwrap_or(false) {
            pattern.pop();
            if new_slice.last().map(String::is_empty).unwrap_or(false) {
                new_slice.pop();
            }
            found = seek_sequence(original_lines, &pattern, line_index, chunk.is_end_of_file);
        }

        match found {
            Some(index) => {
                replacements.push((index, pattern.len(), new_slice));
                line_index = index + pattern.len();
            }
            None => {
                return Err(format!(
                    "Failed to find expected lines in {}:\n{}",
                    file_path.display(),
                    chunk.old_lines.join("\n")
                ));
            }
        }
    }

    replacements.sort_by_key(|(index, _, _)| *index);
    Ok(replacements)
}

fn apply_replacements(lines: &[String], replacements: &[Replacement]) -> Vec<String> {
    let mut result = lines.to_vec();
    for (start, old_len, new_segment) in replacements.iter().rev() {
        result.splice(*start..*start + *old_len, new_segment.iter().cloned());
    }
    result
}

fn seek_sequence(lines: &[String], pattern: &[String], start: usize, eof: bool) -> Option<usize> {
    if pattern.is_empty() {
        return None;
    }
    if let Some(index) = try_match(lines, pattern, start, eof, |a, b| a == b) {
        return Some(index);
    }
    if let Some(index) = try_match(lines, pattern, start, eof, |a, b| {
        a.trim_end() == b.trim_end()
    }) {
        return Some(index);
    }
    if let Some(index) = try_match(lines, pattern, start, eof, |a, b| a.trim() == b.trim()) {
        return Some(index);
    }
    try_match(lines, pattern, start, eof, |a, b| {
        normalize_unicode(a.trim()) == normalize_unicode(b.trim())
    })
}

fn try_match<F>(
    lines: &[String],
    pattern: &[String],
    start: usize,
    eof: bool,
    compare: F,
) -> Option<usize>
where
    F: Fn(&str, &str) -> bool,
{
    if eof && lines.len() >= pattern.len() {
        let from_end = lines.len() - pattern.len();
        if from_end >= start && pattern_matches(lines, pattern, from_end, &compare) {
            return Some(from_end);
        }
    }
    if pattern.len() > lines.len() {
        return None;
    }
    (start..=lines.len() - pattern.len()).find(|&i| pattern_matches(lines, pattern, i, &compare))
}

fn pattern_matches<F>(lines: &[String], pattern: &[String], index: usize, compare: &F) -> bool
where
    F: Fn(&str, &str) -> bool,
{
    pattern
        .iter()
        .enumerate()
        .all(|(offset, expected)| compare(&lines[index + offset], expected))
}

fn normalize_unicode(input: &str) -> String {
    let expanded = input.replace('\u{2026}', "...");
    expanded
        .chars()
        .map(|ch| match ch {
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => '\'',
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => '"',
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2015}' => '-',
            '\u{00A0}' => ' ',
            other => other,
        })
        .collect()
}

fn resolve(directory: &Path, path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        directory.join(candidate)
    }
}

fn relative(directory: &Path, path: &Path) -> String {
    path.strip_prefix(directory)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn split_bom(text: &str) -> (bool, String) {
    match text.strip_prefix('\u{feff}') {
        Some(rest) => (true, rest.to_string()),
        None => (false, text.to_string()),
    }
}

fn join_bom(text: &str, bom: bool) -> String {
    if bom {
        format!("\u{feff}{text}")
    } else {
        text.to_string()
    }
}

fn write_with_dirs(path: &Path, content: &str) -> Result<(), ToolError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| ToolError::Message(error.to_string()))?;
    }
    fs::write(path, content).map_err(|error| ToolError::Message(error.to_string()))
}

/// A parsed patch hunk, without the internal chunk detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hunk {
    /// Add a file with the given contents.
    Add {
        /// Target path.
        path: String,
        /// File contents.
        contents: String,
    },
    /// Delete a file.
    Delete {
        /// Target path.
        path: String,
    },
    /// Update (optionally move) a file.
    Update {
        /// Source path.
        path: String,
        /// Destination path when moving.
        move_path: Option<String>,
    },
}

/// The result of parsing a patch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPatch {
    /// Parsed hunks.
    pub hunks: Vec<Hunk>,
}

/// The result of recognising an `apply_patch` invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaybeApplyPatch {
    /// The invocation carries a patch body.
    Body {
        /// The raw patch text.
        patch: String,
        /// Parsed hunks.
        hunks: Vec<Hunk>,
    },
    /// The invocation is not an `apply_patch` call.
    NotApplyPatch,
}

/// Parse a patch into its hunks.
pub fn parse_patch(text: &str) -> Result<ParsedPatch, String> {
    let raw = parse_raw_patch(text)?;
    let hunks = raw
        .into_iter()
        .map(|hunk| match hunk {
            RawHunk::Add { path, contents } => Hunk::Add { path, contents },
            RawHunk::Delete { path } => Hunk::Delete { path },
            RawHunk::Update {
                path, move_path, ..
            } => Hunk::Update { path, move_path },
        })
        .collect();
    Ok(ParsedPatch { hunks })
}

/// Recognise a direct, `applypatch`, or bash-heredoc `apply_patch` invocation.
pub fn maybe_parse_apply_patch(argv: &[&str]) -> Result<MaybeApplyPatch, String> {
    const COMMANDS: [&str; 2] = ["apply_patch", "applypatch"];

    if argv.len() == 2 && COMMANDS.contains(&argv[0]) {
        let parsed = parse_patch(argv[1])?;
        return Ok(MaybeApplyPatch::Body {
            patch: argv[1].to_string(),
            hunks: parsed.hunks,
        });
    }

    if argv.len() == 3 && argv[0] == "bash" && argv[1] == "-lc" {
        if let Some(content) = extract_heredoc(argv[2]) {
            let parsed = parse_patch(content)?;
            return Ok(MaybeApplyPatch::Body {
                patch: content.to_string(),
                hunks: parsed.hunks,
            });
        }
    }

    Ok(MaybeApplyPatch::NotApplyPatch)
}

fn extract_heredoc(script: &str) -> Option<&str> {
    let marker = script.find("apply_patch")?;
    let after = &script[marker + "apply_patch".len()..];
    let shift = after.find("<<")?;
    let after = after[shift + 2..].trim_start();
    let (quote, after) = match after.chars().next() {
        Some(ch @ ('\'' | '"')) => (Some(ch), &after[ch.len_utf8()..]),
        _ => (None, after),
    };
    let delimiter_end = after.find(|ch: char| {
        if Some(ch) == quote {
            return true;
        }
        ch.is_whitespace()
    })?;
    let delimiter = &after[..delimiter_end];
    if delimiter.is_empty() {
        return None;
    }
    let after = &after[delimiter_end..];
    let body_start = after.find('\n')? + 1;
    let body = &after[body_start..];
    let terminator = format!("\n{delimiter}");
    let end = body.rfind(&terminator)?;
    Some(&body[..end])
}
