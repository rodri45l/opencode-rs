//! Tab migration + closed-tab stack (ports of
//! packages/app/src/context/tab-migration.ts and closed-tabs.ts).

const CLOSED_TAB_LIMIT: usize = 25;

#[derive(Clone, Debug, PartialEq)]
pub enum Tab {
    Session {
        server: String,
        session_id: String,
    },
    Draft {
        draft_id: String,
        server: String,
        directory: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum RawTab {
    Session {
        server: Option<String>,
        session_id: String,
        dir_base64: Option<String>,
    },
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClosedTab {
    pub tab: Tab,
    pub index: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TakeResult {
    pub entry: Option<ClosedTab>,
    pub stack: Vec<ClosedTab>,
}

pub fn migrate_tabs(tabs: Option<Vec<Option<RawTab>>>, fallback: &str) -> Vec<Tab> {
    let tabs = match tabs {
        Some(tabs) => tabs,
        None => return Vec::new(),
    };
    let mut result = Vec::new();
    for tab in tabs.into_iter().flatten() {
        match tab {
            RawTab::Session {
                server, session_id, ..
            } => {
                if session_id.is_empty() {
                    continue;
                }
                result.push(Tab::Session {
                    server: server.unwrap_or_else(|| fallback.to_string()),
                    session_id,
                });
            }
            RawTab::Unknown => {}
        }
    }
    result
}

pub fn push_closed_tab(stack: Vec<ClosedTab>, tab: Tab, index: i64) -> Vec<ClosedTab> {
    if !matches!(tab, Tab::Session { .. }) {
        return stack;
    }
    let mut next = stack;
    next.push(ClosedTab { tab, index });
    if next.len() > CLOSED_TAB_LIMIT {
        let drop = next.len() - CLOSED_TAB_LIMIT;
        next.drain(0..drop);
    }
    next
}

fn is_open(tabs: &[Tab], tab: &Tab) -> bool {
    match tab {
        Tab::Session { server, session_id } => tabs.iter().any(|item| {
            matches!(
                item,
                Tab::Session {
                    server: item_server,
                    session_id: item_session,
                } if item_server == server && item_session == session_id
            )
        }),
        Tab::Draft { .. } => false,
    }
}

pub fn take_closed_tab(stack: Vec<ClosedTab>, open: &[Tab]) -> TakeResult {
    let mut remaining = stack;
    while let Some(entry) = remaining.pop() {
        if !is_open(open, &entry.tab) {
            return TakeResult {
                entry: Some(entry),
                stack: remaining,
            };
        }
    }
    TakeResult {
        entry: None,
        stack: remaining,
    }
}

pub fn remove_closed_tabs(
    stack: Vec<ClosedTab>,
    server: &str,
    session_ids: &[&str],
) -> Vec<ClosedTab> {
    stack
        .into_iter()
        .filter(|entry| match &entry.tab {
            Tab::Session {
                server: entry_server,
                session_id,
            } => !(entry_server == server && session_ids.contains(&session_id.as_str())),
            Tab::Draft { .. } => true,
        })
        .collect()
}

pub fn next_tab_after_close(tabs: &[Tab], index: usize, navigate: bool) -> Option<Option<Tab>> {
    if !navigate {
        return None;
    }
    let next = tabs
        .get(index + 1)
        .cloned()
        .or_else(|| index.checked_sub(1).and_then(|i| tabs.get(i)).cloned());
    Some(next)
}
