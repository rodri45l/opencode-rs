//! Session trimming (port of packages/app/src/context/global-sync/session-trim.ts).

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};

pub const SESSION_RECENT_WINDOW: i64 = 4 * 60 * 60 * 1000;
pub const SESSION_RECENT_LIMIT: usize = 50;

#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub id: String,
    pub parent_id: Option<String>,
    pub created: i64,
    pub updated: Option<i64>,
    pub archived: Option<i64>,
}

#[derive(Default)]
pub struct TrimOptions {
    pub limit: usize,
    pub permission: BTreeSet<String>,
    pub now: i64,
}

fn updated_at(session: &Session) -> i64 {
    session.updated.unwrap_or(session.created)
}

fn compare_recent(a: &Session, b: &Session) -> Ordering {
    let a_updated = updated_at(a);
    let b_updated = updated_at(b);
    if a_updated != b_updated {
        return b_updated.cmp(&a_updated);
    }
    a.id.cmp(&b.id)
}

fn take_recent(sessions: &[Session], limit: usize, cutoff: i64) -> Vec<Session> {
    if limit == 0 {
        return Vec::new();
    }
    let mut selected: Vec<Session> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for session in sessions {
        if session.id.is_empty() || !seen.insert(session.id.clone()) {
            continue;
        }
        if updated_at(session) <= cutoff {
            continue;
        }
        match selected
            .iter()
            .position(|item| compare_recent(session, item) == Ordering::Less)
        {
            Some(index) => selected.insert(index, session.clone()),
            None => selected.push(session.clone()),
        }
        if selected.len() > limit {
            selected.pop();
        }
    }
    selected
}

pub fn trim_sessions(input: &[Session], options: &TrimOptions) -> Vec<Session> {
    let limit = options.limit;
    let cutoff = options.now - SESSION_RECENT_WINDOW;

    let mut all: Vec<Session> = input
        .iter()
        .filter(|session| !session.id.is_empty() && session.archived.is_none())
        .cloned()
        .collect();
    all.sort_by(|a, b| a.id.cmp(&b.id));

    let mut roots: Vec<Session> = all
        .iter()
        .filter(|session| session.parent_id.is_none())
        .cloned()
        .collect();
    roots.sort_by(compare_recent);
    let children: Vec<Session> = all
        .iter()
        .filter(|session| session.parent_id.is_some())
        .cloned()
        .collect();

    let base: Vec<Session> = roots.iter().take(limit).cloned().collect();
    let tail = if limit < roots.len() {
        &roots[limit..]
    } else {
        &[]
    };
    let recent = take_recent(tail, SESSION_RECENT_LIMIT, cutoff);
    let mut keep_roots = base;
    keep_roots.extend(recent);
    let keep_root_ids: HashSet<String> = keep_roots
        .iter()
        .map(|session| session.id.clone())
        .collect();

    let keep_children: Vec<Session> = children
        .into_iter()
        .filter(|session| {
            if let Some(parent_id) = &session.parent_id {
                if keep_root_ids.contains(parent_id) {
                    return true;
                }
            }
            if options.permission.contains(&session.id) {
                return true;
            }
            updated_at(session) > cutoff
        })
        .collect();

    let mut result = keep_roots;
    result.extend(keep_children);
    result.sort_by(|a, b| a.id.cmp(&b.id));
    result
}
