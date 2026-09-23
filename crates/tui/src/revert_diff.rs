//! Revert-diff file summary.
//!
//! Port of packages/tui/src/util/revert-diff.ts `getRevertDiffFiles` behaviour
//! (upstream 18ef3cc).

/// One file changed by a revert patch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevertDiffFile {
    pub filename: String,
    pub additions: i64,
    pub deletions: i64,
}

/// Summarize the files in a unified diff.
pub fn get_revert_diff_files(diff_text: &str) -> Vec<RevertDiffFile> {
    if diff_text.is_empty() {
        return Vec::new();
    }
    let mut files = Vec::new();
    let mut current: Option<(Option<String>, Option<String>, i64, i64)> = None;

    let flush = |files: &mut Vec<RevertDiffFile>,
                 current: &mut Option<(Option<String>, Option<String>, i64, i64)>| {
        if let Some((old, new, additions, deletions)) = current.take() {
            let filename = new
                .filter(|value| value != "/dev/null")
                .or_else(|| old.filter(|value| value != "/dev/null"))
                .unwrap_or_else(|| "unknown".to_string());
            files.push(RevertDiffFile {
                filename: strip_prefix(&filename),
                additions,
                deletions,
            });
        }
    };

    for line in diff_text.split('\n') {
        if line.starts_with("diff --git ") {
            flush(&mut files, &mut current);
            current = Some((None, None, 0, 0));
            continue;
        }
        let Some(entry) = current.as_mut() else {
            continue;
        };
        if let Some(rest) = line.strip_prefix("--- ") {
            entry.0 = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("+++ ") {
            entry.1 = Some(rest.trim().to_string());
        } else if line.starts_with("@@") {
            continue;
        } else if line.starts_with('+') {
            entry.2 += 1;
        } else if line.starts_with('-') {
            entry.3 += 1;
        }
    }
    flush(&mut files, &mut current);
    files
}

fn strip_prefix(value: &str) -> String {
    value
        .strip_prefix("a/")
        .or_else(|| value.strip_prefix("b/"))
        .unwrap_or(value)
        .to_string()
}
