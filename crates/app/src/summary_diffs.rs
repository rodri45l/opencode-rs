//! Unique summary diffs (port of packages/app/src/pages/session/timeline/summary-diffs.ts).

#[derive(Clone, Debug, PartialEq)]
pub struct Diff {
    pub file: Option<String>,
    pub additions: i64,
    pub deletions: i64,
}

pub fn unique_summary_diffs(diffs: Option<Vec<Diff>>) -> Vec<Diff> {
    let diffs = match diffs {
        Some(diffs) => diffs,
        None => return Vec::new(),
    };
    let mut last: std::collections::BTreeMap<String, (usize, Diff)> =
        std::collections::BTreeMap::new();
    for (index, diff) in diffs.into_iter().enumerate() {
        let file = match &diff.file {
            Some(file) => file.clone(),
            None => continue,
        };
        last.insert(file, (index, diff));
    }
    let mut entries: Vec<(usize, Diff)> = last.into_values().collect();
    entries.sort_by_key(|(index, _)| *index);
    entries.into_iter().map(|(_, diff)| diff).collect()
}
