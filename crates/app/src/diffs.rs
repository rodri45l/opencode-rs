//! Diff normalisation (port of packages/app/src/utils/diffs.ts).

#[derive(Clone, Debug, PartialEq)]
pub struct FileDiff {
    pub file: String,
    pub patch: String,
    pub additions: i64,
    pub deletions: i64,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Summary {
    pub title: String,
    pub diffs: Vec<FileDiff>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub id: String,
    pub summary: Option<Summary>,
}

pub enum DiffSource {
    Single(FileDiff),
    Many(Vec<FileDiff>),
    Keyed(Vec<FileDiff>),
}

fn valid(diff: &FileDiff) -> bool {
    matches!(diff.status.as_str(), "added" | "deleted" | "modified")
}

pub fn diffs(source: DiffSource) -> Vec<FileDiff> {
    match source {
        DiffSource::Single(diff) => {
            if valid(&diff) {
                vec![diff]
            } else {
                Vec::new()
            }
        }
        DiffSource::Many(values) => values.into_iter().filter(valid).collect(),
        DiffSource::Keyed(values) => values.into_iter().filter(valid).collect(),
    }
}

pub fn message(input: Message) -> Message {
    match input.summary {
        Some(summary) => Message {
            id: input.id,
            summary: Some(Summary {
                title: summary.title,
                diffs: diffs(DiffSource::Many(summary.diffs)),
            }),
        },
        None => Message {
            id: input.id,
            summary: None,
        },
    }
}
