//! Session diff projection.
//!
//! Derived from the observable behaviour pinned by
//! `packages/session-ui/src/components/session-diff.ts` (upstream 18ef3cc):
//! whole-file unified/VCS patches render as complete diffs, ordinary tool
//! patches stay partial, separated hunks keep their collapse gap, headerless and
//! legacy patches work, and malformed persisted patches are ignored. The Solid
//! `View` wrapper is represented as a plain value.

use std::fmt;

/// Error raised by the diff projection.
#[derive(Debug, PartialEq, Eq)]
pub struct NotImplemented(pub &'static str);

impl fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for NotImplemented {}

/// Result alias for the diff projection.
pub type PortResult<T> = Result<T, NotImplemented>;

/// Message used by the ported tests when expecting success.
pub const NOTE: &str = "session-ui session diff projection";

/// A persisted diff input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffInput {
    pub file: String,
    pub patch: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub additions: i64,
    pub deletions: i64,
    pub status: String,
}

/// A file/patch pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiffInput {
    pub file: String,
    pub patch: String,
}

/// A collapsed hunk gap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    pub collapsed_before: i64,
}

/// A resolved file diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiff {
    pub name: String,
    pub is_partial: bool,
    pub deletion_lines: Vec<String>,
    pub addition_lines: Vec<String>,
    pub hunks: Vec<Hunk>,
}

/// A normalized diff view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub file_diff: FileDiff,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedLine {
    kind: char,
    text: String,
    newline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedHunk {
    old_start: i64,
    old_count: i64,
    new_start: i64,
    new_count: i64,
    lines: Vec<ParsedLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedPatch {
    has_header: bool,
    diff_git: bool,
    hunks: Vec<ParsedHunk>,
}

fn parse_hunk_header(line: &str) -> Option<(i64, i64, i64, i64)> {
    let inner = line.strip_prefix("@@")?;
    let end = inner.find("@@")?;
    let ranges = &inner[..end];
    let mut parts = ranges.split_whitespace();
    let old = parts.next()?.strip_prefix('-')?;
    let new = parts.next()?.strip_prefix('+')?;
    let parse = |value: &str| -> Option<(i64, i64)> {
        match value.split_once(',') {
            Some((start, count)) => Some((start.parse().ok()?, count.parse().ok()?)),
            None => Some((value.parse().ok()?, 1)),
        }
    };
    let (old_start, old_count) = parse(old)?;
    let (new_start, new_count) = parse(new)?;
    Some((old_start, old_count, new_start, new_count))
}

fn parse_patch(patch: &str) -> Option<ParsedPatch> {
    let mut lines: Vec<&str> = patch.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let mut parsed = ParsedPatch {
        has_header: false,
        diff_git: false,
        hunks: Vec::new(),
    };
    let mut index = 0usize;
    while index < lines.len() {
        let line = lines[index];
        if line.starts_with("diff --git ") {
            parsed.has_header = true;
            parsed.diff_git = true;
            index += 1;
            continue;
        }
        if line.starts_with("Index: ") || line.starts_with("--- ") || line.starts_with("+++ ") {
            parsed.has_header = true;
            index += 1;
            continue;
        }
        if line.starts_with("@@") {
            let (old_start, old_count, new_start, new_count) = parse_hunk_header(line)?;
            index += 1;
            let mut hunk_lines: Vec<ParsedLine> = Vec::new();
            while index < lines.len() {
                let current = lines[index];
                if current.starts_with("@@")
                    || current.starts_with("diff --git ")
                    || current.starts_with("Index: ")
                    || current.starts_with("--- ")
                    || current.starts_with("+++ ")
                {
                    break;
                }
                let kind = current.chars().next().unwrap_or('\0');
                if kind != ' ' && kind != '+' && kind != '-' && kind != '\\' {
                    return None;
                }
                let text = current[1..].to_string();
                if kind == '\\' {
                    if let Some(previous) = hunk_lines.last_mut() {
                        previous.newline = false;
                    }
                } else {
                    hunk_lines.push(ParsedLine {
                        kind,
                        text,
                        newline: true,
                    });
                }
                index += 1;
            }
            let old_actual = hunk_lines
                .iter()
                .filter(|line| line.kind == ' ' || line.kind == '-')
                .count() as i64;
            let new_actual = hunk_lines
                .iter()
                .filter(|line| line.kind == ' ' || line.kind == '+')
                .count() as i64;
            if old_actual != old_count || new_actual != new_count {
                return None;
            }
            parsed.hunks.push(ParsedHunk {
                old_start,
                old_count,
                new_start,
                new_count,
                lines: hunk_lines,
            });
            continue;
        }
        index += 1;
    }
    Some(parsed)
}

fn has_tab_header(patch: &str) -> bool {
    let lines: Vec<&str> = patch.split('\n').collect();
    for (index, line) in lines.iter().enumerate() {
        if line.starts_with("--- ") && line.contains('\t') {
            if let Some(next) = lines.get(index + 1) {
                if next.starts_with("+++ ") && next.contains('\t') {
                    return true;
                }
            }
        }
    }
    false
}

fn lines_to_strings(lines: &[ParsedLine]) -> Vec<String> {
    lines
        .iter()
        .map(|line| {
            if line.newline {
                format!("{}\n", line.text)
            } else {
                line.text.clone()
            }
        })
        .collect()
}

fn complete_patch_contents(patch: &str) -> Option<(Vec<ParsedLine>, Vec<ParsedLine>)> {
    let parsed = parse_patch(patch)?;
    if !parsed.has_header {
        return None;
    }
    if !parsed.diff_git && !has_tab_header(patch) {
        return None;
    }
    if parsed.hunks.len() != 1 {
        return None;
    }
    let hunk = &parsed.hunks[0];
    if hunk.old_start > 1 || hunk.new_start > 1 {
        return None;
    }
    let mut before: Vec<ParsedLine> = Vec::new();
    let mut after: Vec<ParsedLine> = Vec::new();
    for line in &hunk.lines {
        match line.kind {
            '-' => before.push(line.clone()),
            '+' => after.push(line.clone()),
            ' ' => {
                before.push(line.clone());
                after.push(line.clone());
            }
            _ => return None,
        }
    }
    Some((before, after))
}

fn patch_input(file: &str, patch: &str) -> Option<String> {
    let parsed = parse_patch(patch)?;
    if parsed.has_header {
        return Some(patch.to_string());
    }
    if parsed.hunks.is_empty() {
        return None;
    }
    Some(format!(
        "Index: {file}\n===================================================================\n--- {file}\t\n+++ {file}\t\n{patch}"
    ))
}

fn empty_diff(file: &str) -> FileDiff {
    FileDiff {
        name: file.to_string(),
        is_partial: false,
        deletion_lines: Vec::new(),
        addition_lines: Vec::new(),
        hunks: Vec::new(),
    }
}

fn file_diff_from_content(file: &str, before: &str, after: &str) -> FileDiff {
    if before.is_empty() && after.is_empty() {
        return empty_diff(file);
    }
    let split = |text: &str| -> Vec<ParsedLine> {
        text.split_inclusive('\n')
            .map(|part| {
                let newline = part.ends_with('\n');
                let content = part.strip_suffix('\n').unwrap_or(part);
                ParsedLine {
                    kind: ' ',
                    text: content.to_string(),
                    newline,
                }
            })
            .collect()
    };
    FileDiff {
        name: file.to_string(),
        is_partial: false,
        deletion_lines: lines_to_strings(&split(before)),
        addition_lines: lines_to_strings(&split(after)),
        hunks: Vec::new(),
    }
}

fn file_diff_from_patch(file: &str, patch: &str) -> FileDiff {
    if let Some((before, after)) = complete_patch_contents(patch) {
        return FileDiff {
            name: file.to_string(),
            is_partial: false,
            deletion_lines: lines_to_strings(&before),
            addition_lines: lines_to_strings(&after),
            hunks: Vec::new(),
        };
    }
    let Some(input) = patch_input(file, patch) else {
        return empty_diff(file);
    };
    let Some(parsed) = parse_patch(&input) else {
        return empty_diff(file);
    };
    let mut deletion_lines: Vec<String> = Vec::new();
    let mut addition_lines: Vec<String> = Vec::new();
    let mut hunks: Vec<Hunk> = Vec::new();
    let mut previous_end: Option<i64> = None;
    for hunk in &parsed.hunks {
        let collapsed_before = match previous_end {
            Some(end) => (hunk.old_start - end).max(0),
            None => (hunk.old_start - 1).max(0),
        };
        hunks.push(Hunk { collapsed_before });
        previous_end = Some(hunk.old_start + hunk.old_count);
        for line in &hunk.lines {
            let rendered = if line.newline {
                format!("{}\n", line.text)
            } else {
                line.text.clone()
            };
            if line.kind == ' ' || line.kind == '-' {
                deletion_lines.push(rendered.clone());
            }
            if line.kind == ' ' || line.kind == '+' {
                addition_lines.push(rendered);
            }
        }
    }
    FileDiff {
        name: file.to_string(),
        is_partial: true,
        deletion_lines,
        addition_lines,
        hunks,
    }
}

/// Resolve a file diff from a patch or from before/after content.
pub fn resolve_file_diff(input: &FileDiffInput) -> PortResult<FileDiff> {
    Ok(file_diff_from_patch(&input.file, &input.patch))
}

pub fn resolve_source(
    file: &str,
    patch: Option<&str>,
    before: Option<&str>,
    after: Option<&str>,
) -> FileDiff {
    match patch {
        Some(patch) => file_diff_from_patch(file, patch),
        None => file_diff_from_content(file, before.unwrap_or(""), after.unwrap_or("")),
    }
}

/// Normalize a persisted diff into a view.
pub fn normalize(diff: &DiffInput) -> PortResult<View> {
    Ok(View {
        file_diff: resolve_source(
            &diff.file,
            diff.patch.as_deref(),
            diff.before.as_deref(),
            diff.after.as_deref(),
        ),
    })
}

/// Render one side of a diff view.
pub fn text(view: &View, side: &str) -> PortResult<String> {
    let lines = if side == "deletions" {
        &view.file_diff.deletion_lines
    } else {
        &view.file_diff.addition_lines
    };
    Ok(lines.join(""))
}
