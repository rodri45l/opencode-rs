//! Session diff projection (Phase 7).
//!
//! Port of packages/session-ui/src/components/session-diff.ts behaviour
//! (upstream 18ef3cc). The reference delegates parsing to `@pierre/diffs` and
//! `diff`; this module reproduces the observable projections those libraries
//! produce for the cases pinned by the reference tests. Only the host-neutral
//! data shape is ported; the rendered diff UI is human-verified.

/// The source of a diff, matching the reference `DiffSource` shape.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiffSource {
    pub file: String,
    pub patch: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub additions: i64,
    pub deletions: i64,
    pub status: Option<String>,
}

/// One collapsed gap between hunks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    pub old_start: i64,
    pub new_start: i64,
    pub collapsed_before: i64,
}

/// A resolved file diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiff {
    pub name: String,
    pub is_partial: bool,
    pub addition_lines: Vec<String>,
    pub deletion_lines: Vec<String>,
    pub hunks: Vec<Hunk>,
}

/// A normalized diff view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewDiff {
    pub file: String,
    pub additions: i64,
    pub deletions: i64,
    pub status: Option<String>,
    pub file_diff: FileDiff,
}

/// Resolve a diff source to a file diff.
pub fn resolve_file_diff(diff: &DiffSource) -> FileDiff {
    if let Some(patch) = &diff.patch {
        return file_diff_from_patch(&diff.file, patch);
    }
    file_diff_from_content(
        &diff.file,
        diff.before.as_deref().unwrap_or(""),
        diff.after.as_deref().unwrap_or(""),
    )
}

/// Normalize a diff source into a view.
pub fn normalize(diff: &DiffSource) -> ViewDiff {
    ViewDiff {
        file: diff.file.clone(),
        additions: diff.additions,
        deletions: diff.deletions,
        status: diff.status.clone(),
        file_diff: resolve_file_diff(diff),
    }
}

/// The full text of one side of a resolved diff.
pub fn text(diff: &ViewDiff, side: DiffSide) -> String {
    match side {
        DiffSide::Deletions => diff.file_diff.deletion_lines.join(""),
        DiffSide::Additions => diff.file_diff.addition_lines.join(""),
    }
}

/// Which side of a diff to read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffSide {
    Deletions,
    Additions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineKind {
    Context,
    Delete,
    Add,
    NoNewline,
}

#[derive(Debug, Clone)]
struct ParsedHunk {
    old_start: i64,
    new_start: i64,
    old_count: i64,
    lines: Vec<(LineKind, String)>,
}

#[derive(Debug, Clone, Default)]
struct ParsedPatch {
    has_index: bool,
    old_file: Option<String>,
    new_file: Option<String>,
    hunks: Vec<ParsedHunk>,
    malformed: bool,
}

fn file_diff_from_patch(file: &str, patch: &str) -> FileDiff {
    if let Some((before, after)) = complete_patch_contents(patch) {
        return file_diff_from_content(file, &before, &after);
    }
    let input = patch_input(file, patch);
    match input {
        Some(input) => partial_file_diff(file, &input),
        None => empty_file_diff(file),
    }
}

fn complete_patch_contents(patch: &str) -> Option<(String, String)> {
    let parsed = parse_patch(patch);
    if parsed.malformed {
        return None;
    }
    if !(parsed.has_index || parsed.old_file.is_some() || parsed.new_file.is_some()) {
        return None;
    }
    // Snapshot and VCS producers request full context; tool patches use shorter
    // context and must stay partial.
    if !patch.starts_with("diff --git ") && !has_tab_file_headers(patch) {
        return None;
    }
    if parsed.hunks.len() != 1 {
        return None;
    }
    let hunk = parsed.hunks.first()?;
    if hunk.old_start > 1 || hunk.new_start > 1 {
        return None;
    }

    let mut before: Vec<(String, bool)> = Vec::new();
    let mut after: Vec<(String, bool)> = Vec::new();
    let mut previous: Option<LineKind> = None;
    for (kind, text) in &hunk.lines {
        match kind {
            LineKind::NoNewline => {
                if matches!(previous, Some(LineKind::Delete) | Some(LineKind::Context)) {
                    if let Some(last) = before.last_mut() {
                        last.1 = false;
                    }
                }
                if matches!(previous, Some(LineKind::Add) | Some(LineKind::Context)) {
                    if let Some(last) = after.last_mut() {
                        last.1 = false;
                    }
                }
            }
            LineKind::Delete => {
                before.push((text.clone(), true));
                previous = Some(LineKind::Delete);
            }
            LineKind::Add => {
                after.push((text.clone(), true));
                previous = Some(LineKind::Add);
            }
            LineKind::Context => {
                before.push((text.clone(), true));
                after.push((text.clone(), true));
                previous = Some(LineKind::Context);
            }
        }
    }
    Some((join_lines(&before), join_lines(&after)))
}

fn join_lines(lines: &[(String, bool)]) -> String {
    lines
        .iter()
        .map(|(text, newline)| {
            if *newline {
                format!("{text}\n")
            } else {
                text.clone()
            }
        })
        .collect()
}

fn patch_input(file: &str, patch: &str) -> Option<String> {
    let parsed = parse_patch(patch);
    if parsed.has_index || parsed.old_file.is_some() || parsed.new_file.is_some() {
        return Some(patch.to_string());
    }
    if parsed.hunks.is_empty() {
        return None;
    }
    Some(format!(
        "Index: {file}\n===================================================================\n--- {file}\t\n+++ {file}\t\n{patch}"
    ))
}

fn file_diff_from_content(file: &str, before: &str, after: &str) -> FileDiff {
    FileDiff {
        name: file.to_string(),
        is_partial: false,
        addition_lines: split_lines(after),
        deletion_lines: split_lines(before),
        hunks: Vec::new(),
    }
}

fn empty_file_diff(file: &str) -> FileDiff {
    file_diff_from_content(file, "", "")
}

fn split_lines(value: &str) -> Vec<String> {
    if value.is_empty() {
        return Vec::new();
    }
    value.split_inclusive('\n').map(String::from).collect()
}

fn partial_file_diff(file: &str, patch: &str) -> FileDiff {
    let parsed = parse_patch(patch);
    if parsed.malformed {
        return FileDiff {
            name: file.to_string(),
            is_partial: true,
            addition_lines: Vec::new(),
            deletion_lines: Vec::new(),
            hunks: Vec::new(),
        };
    }
    let mut addition_lines = Vec::new();
    let mut deletion_lines = Vec::new();
    let mut hunks = Vec::new();
    let mut consumed_old = 0i64;
    for hunk in &parsed.hunks {
        hunks.push(Hunk {
            old_start: hunk.old_start,
            new_start: hunk.new_start,
            collapsed_before: hunk.old_start - 1 - consumed_old,
        });
        consumed_old += hunk.old_count;
        let mut previous: Option<LineKind> = None;
        for (kind, text) in &hunk.lines {
            match kind {
                LineKind::NoNewline => {
                    if matches!(previous, Some(LineKind::Delete) | Some(LineKind::Context)) {
                        strip_last_newline(&mut deletion_lines);
                    }
                    if matches!(previous, Some(LineKind::Add) | Some(LineKind::Context)) {
                        strip_last_newline(&mut addition_lines);
                    }
                }
                LineKind::Delete => {
                    deletion_lines.push(format!("{text}\n"));
                    previous = Some(LineKind::Delete);
                }
                LineKind::Add => {
                    addition_lines.push(format!("{text}\n"));
                    previous = Some(LineKind::Add);
                }
                LineKind::Context => {
                    deletion_lines.push(format!("{text}\n"));
                    addition_lines.push(format!("{text}\n"));
                    previous = Some(LineKind::Context);
                }
            }
        }
    }
    FileDiff {
        name: file.to_string(),
        is_partial: true,
        addition_lines,
        deletion_lines,
        hunks,
    }
}

fn strip_last_newline(lines: &mut [String]) {
    if let Some(last) = lines.last_mut() {
        if let Some(stripped) = last.strip_suffix('\n') {
            *last = stripped.to_string();
        }
    }
}

fn has_tab_file_headers(patch: &str) -> bool {
    let lines: Vec<&str> = patch.split('\n').collect();
    let mut index = 0;
    while index + 1 < lines.len() {
        if lines[index].starts_with("--- ")
            && lines[index].ends_with('\t')
            && lines[index + 1].starts_with("+++ ")
            && lines[index + 1].ends_with('\t')
        {
            return true;
        }
        index += 1;
    }
    false
}

fn parse_patch(patch: &str) -> ParsedPatch {
    let mut lines: Vec<&str> = patch.split('\n').collect();
    if patch.ends_with('\n') {
        lines.pop();
    }
    let mut parsed = ParsedPatch::default();
    if lines.iter().any(|line| line.ends_with('\r')) {
        parsed.malformed = true;
        return parsed;
    }

    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        if let Some((old_start, new_start, old_count)) = parse_hunk_header(line) {
            index += 1;
            let mut hunk = ParsedHunk {
                old_start,
                new_start,
                old_count,
                lines: Vec::new(),
            };
            while index < lines.len() {
                let body = lines[index];
                if parse_hunk_header(body).is_some() || is_header_line(body) {
                    break;
                }
                let kind = match body.chars().next() {
                    Some(' ') => LineKind::Context,
                    Some('-') => LineKind::Delete,
                    Some('+') => LineKind::Add,
                    Some('\\') => LineKind::NoNewline,
                    _ => {
                        parsed.malformed = true;
                        return parsed;
                    }
                };
                hunk.lines.push((kind, body[1..].to_string()));
                index += 1;
            }
            parsed.hunks.push(hunk);
            continue;
        }
        if line.starts_with("Index:") {
            parsed.has_index = true;
        } else if line.starts_with("diff --git ") {
        } else if let Some(rest) = line.strip_prefix("--- ") {
            parsed.old_file = Some(strip_file_meta(rest));
        } else if let Some(rest) = line.strip_prefix("+++ ") {
            parsed.new_file = Some(strip_file_meta(rest));
        }
        index += 1;
    }
    parsed
}

fn is_header_line(line: &str) -> bool {
    line.starts_with("Index:")
        || line.starts_with("diff --git ")
        || line.starts_with("--- ")
        || line.starts_with("+++ ")
        || line.starts_with("===")
}

fn strip_file_meta(value: &str) -> String {
    value.split('\t').next().unwrap_or(value).trim().to_string()
}

fn parse_hunk_header(line: &str) -> Option<(i64, i64, i64)> {
    let rest = line.strip_prefix("@@ -")?;
    let (old_part, tail) = rest.split_once(" +")?;
    let (new_part, _) = tail.split_once(" @@")?;
    let old_start = old_part.split(',').next()?.parse::<i64>().ok()?;
    let old_count = old_part
        .split(',')
        .nth(1)
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(1);
    let new_start = new_part.split(',').next()?.parse::<i64>().ok()?;
    Some((old_start, new_start, old_count))
}
