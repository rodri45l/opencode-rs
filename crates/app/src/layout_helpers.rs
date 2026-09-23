//! Layout session-key helpers (port of packages/app/src/context/layout-helpers.ts).

use std::collections::BTreeMap;
use std::collections::HashSet;

pub fn ensure_session_key(
    key: &str,
    mut touch: impl FnMut(&str),
    mut seed: impl FnMut(&str),
) -> String {
    touch(key);
    seed(key);
    key.to_string()
}

pub fn create_session_key_reader(key: &str) -> String {
    key.to_string()
}

pub struct PruneInput {
    pub keep: Option<String>,
    pub max: usize,
    pub used: BTreeMap<String, i64>,
    pub view: Vec<String>,
    pub tabs: Vec<String>,
}

pub fn prune_session_keys(input: PruneInput) -> Vec<String> {
    let keep = match &input.keep {
        Some(keep) => keep.clone(),
        None => return Vec::new(),
    };

    let mut keys: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for key in input.view.iter().chain(input.tabs.iter()) {
        if seen.insert(key.clone()) {
            keys.push(key.clone());
        }
    }
    if keys.len() <= input.max {
        return Vec::new();
    }

    let score = |key: &str| -> i64 {
        if key == keep {
            i64::MAX
        } else {
            *input.used.get(key).unwrap_or(&0)
        }
    };
    keys.sort_by_key(|key| std::cmp::Reverse(score(key)));
    keys.split_off(input.max)
}
