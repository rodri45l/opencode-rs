//! Session cache eviction (port of
//! packages/app/src/context/global-sync/session-cache.ts).

use std::collections::{BTreeMap, BTreeSet, HashSet};

pub const SESSION_CACHE_LIMIT: usize = 40;

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub id: String,
    pub session_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub id: String,
    pub session_id: String,
    pub message_id: String,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Store {
    pub session_status: BTreeMap<String, String>,
    pub session_diff: BTreeMap<String, Vec<String>>,
    pub todo: BTreeMap<String, Vec<String>>,
    pub message: BTreeMap<String, Vec<Message>>,
    pub session_message: BTreeMap<String, Vec<String>>,
    pub part: BTreeMap<String, Vec<Part>>,
    pub permission: BTreeMap<String, Vec<String>>,
    pub question: BTreeMap<String, Vec<String>>,
    pub part_text_accum_delta: BTreeMap<String, String>,
}

pub fn drop_session_caches(store: &mut Store, session_ids: &[&str]) {
    let stale: HashSet<String> = session_ids
        .iter()
        .filter(|session_id| !session_id.is_empty())
        .map(|session_id| (*session_id).to_string())
        .collect();
    if stale.is_empty() {
        return;
    }

    let keys: Vec<String> = store.part.keys().cloned().collect();
    for key in keys {
        let remove = store
            .part
            .get(&key)
            .map(|parts| parts.iter().any(|part| stale.contains(&part.session_id)))
            .unwrap_or(false);
        if !remove {
            continue;
        }
        if let Some(parts) = store.part.get(&key) {
            for part in parts {
                store.part_text_accum_delta.remove(&part.id);
            }
        }
        store.part.remove(&key);
    }

    for session_id in &stale {
        store.message.remove(session_id);
        store.todo.remove(session_id);
        store.session_message.remove(session_id);
        store.session_diff.remove(session_id);
        store.session_status.remove(session_id);
        store.permission.remove(session_id);
        store.question.remove(session_id);
    }
}

pub fn pick_session_cache_evictions(
    seen: &mut BTreeSet<String>,
    keep: &str,
    limit: usize,
    preserve: &[&str],
) -> Vec<String> {
    let mut stale: Vec<String> = Vec::new();
    let mut keep_set: HashSet<String> = HashSet::new();
    keep_set.insert(keep.to_string());
    for value in preserve {
        keep_set.insert((*value).to_string());
    }
    seen.remove(keep);
    seen.insert(keep.to_string());
    for id in seen.iter() {
        if seen.len() - stale.len() <= limit {
            break;
        }
        if keep_set.contains(id) {
            continue;
        }
        stale.push(id.clone());
    }
    for id in &stale {
        seen.remove(id);
    }
    stale
}
