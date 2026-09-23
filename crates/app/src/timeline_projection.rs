//! Timeline row reconciliation (port of
//! packages/app/src/pages/session/timeline/row-reconciliation.ts).

use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub enum TimelineRow {
    AssistantPart {
        user_message_id: String,
        group_key: String,
        part_ids: Vec<String>,
    },
    UserMessage {
        user_message_id: String,
    },
}

pub fn row_key(row: &TimelineRow) -> String {
    match row {
        TimelineRow::AssistantPart {
            user_message_id,
            group_key,
            ..
        } => format!("assistant-part:{user_message_id}:{group_key}"),
        TimelineRow::UserMessage { user_message_id } => format!("user-message:{user_message_id}"),
    }
}

fn assistant_context(row: &TimelineRow) -> Option<(&str, &str, &[String])> {
    match row {
        TimelineRow::AssistantPart {
            user_message_id,
            group_key,
            part_ids,
        } => Some((user_message_id, group_key, part_ids)),
        TimelineRow::UserMessage { .. } => None,
    }
}

struct PriorContext {
    index: usize,
    row: TimelineRow,
}

pub fn reuse_timeline_rows(previous: Vec<TimelineRow>, rows: Vec<TimelineRow>) -> Vec<TimelineRow> {
    if previous.is_empty() {
        return rows;
    }

    let mut by_key: HashMap<String, TimelineRow> = HashMap::new();
    for row in &previous {
        by_key.insert(row_key(row), row.clone());
    }

    let mut context_by_part: HashMap<String, PriorContext> = HashMap::new();
    for (index, row) in previous.iter().enumerate() {
        if let Some((user_message_id, _group_key, part_ids)) = assistant_context(row) {
            for part_id in part_ids {
                context_by_part.insert(
                    format!("{user_message_id}:{part_id}"),
                    PriorContext {
                        index,
                        row: row.clone(),
                    },
                );
            }
        }
    }

    let mut reserved: HashMap<String, usize> = HashMap::new();
    for (index, row) in rows.iter().enumerate() {
        if assistant_context(row).is_none() {
            continue;
        }
        let key = row_key(row);
        if by_key.contains_key(&key) && !reserved.contains_key(&key) {
            reserved.insert(key, index);
        }
    }

    let mut claimed: HashSet<String> = HashSet::new();
    let next: Vec<TimelineRow> = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let stabilized =
                stabilize_context_key(&context_by_part, &reserved, row, index, &mut claimed);
            match by_key.get(&row_key(&stabilized)) {
                Some(existing) if *existing == stabilized => existing.clone(),
                _ => stabilized,
            }
        })
        .collect();

    if previous.len() == next.len() && previous == next {
        return previous;
    }
    next
}

fn stabilize_context_key(
    context_by_part: &HashMap<String, PriorContext>,
    reserved: &HashMap<String, usize>,
    row: &TimelineRow,
    row_index: usize,
    claimed: &mut HashSet<String>,
) -> TimelineRow {
    let (user_message_id, group_key, part_ids) = match assistant_context(row) {
        Some(value) => value,
        None => return row.clone(),
    };

    let mut existing: Option<&PriorContext> = None;
    for part_id in part_ids {
        let candidate = match context_by_part.get(&format!("{user_message_id}:{part_id}")) {
            Some(candidate) => candidate,
            None => continue,
        };
        let key = row_key(&candidate.row);
        if claimed.contains(&key) {
            continue;
        }
        if let Some(owner) = reserved.get(&key) {
            if *owner != row_index {
                continue;
            }
        }
        existing = match existing {
            Some(current) if current.index <= candidate.index => Some(current),
            _ => Some(candidate),
        };
    }

    let existing = match existing {
        Some(existing) => existing,
        None => return row.clone(),
    };
    let key = row_key(&existing.row);
    claimed.insert(key);
    if group_key
        == assistant_context(&existing.row)
            .map(|value| value.1)
            .unwrap_or("")
    {
        return row.clone();
    }
    TimelineRow::AssistantPart {
        user_message_id: user_message_id.to_string(),
        group_key: assistant_context(&existing.row)
            .map(|value| value.1.to_string())
            .unwrap_or_default(),
        part_ids: part_ids.to_vec(),
    }
}
