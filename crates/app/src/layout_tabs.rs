//! Session tab state transitions (port of packages/app/src/context/layout-tabs.ts).

pub const SESSION_OPEN_FILE_TAB: &str = "open-file";

#[derive(Clone, Debug, PartialEq)]
pub struct SessionTabState {
    pub all: Vec<String>,
    pub active: Option<String>,
    pub preview: Option<String>,
}

fn effective_preview(current: &SessionTabState) -> Option<String> {
    current.preview.clone().or_else(|| {
        if current.all.iter().any(|item| item == SESSION_OPEN_FILE_TAB) {
            Some(SESSION_OPEN_FILE_TAB.to_string())
        } else {
            None
        }
    })
}

fn position(list: &[String], value: &str) -> Option<usize> {
    list.iter().position(|item| item == value)
}

pub fn preview_session_tab(current: SessionTabState, tab: &str) -> SessionTabState {
    let preview = effective_preview(&current);
    let preview_index = preview
        .as_deref()
        .and_then(|value| position(&current.all, value));
    let existing_index = position(&current.all, tab);

    if let Some(_index) = existing_index {
        if preview_index.is_none() || preview.as_deref() == Some(tab) {
            let next_preview = if preview.as_deref() == Some(tab) {
                Some(tab.to_string())
            } else {
                None
            };
            return SessionTabState {
                all: current.all,
                active: Some(tab.to_string()),
                preview: next_preview,
            };
        }
        let all = current
            .all
            .into_iter()
            .filter(|item| Some(item.as_str()) != preview.as_deref())
            .collect();
        return SessionTabState {
            all,
            active: Some(tab.to_string()),
            preview: None,
        };
    }

    if preview_index.is_none() {
        let mut all = current.all;
        all.push(tab.to_string());
        return SessionTabState {
            all,
            active: Some(tab.to_string()),
            preview: Some(tab.to_string()),
        };
    }

    let index = preview_index.unwrap();
    let all = current
        .all
        .into_iter()
        .enumerate()
        .map(|(i, item)| if i == index { tab.to_string() } else { item })
        .collect();
    SessionTabState {
        all,
        active: Some(tab.to_string()),
        preview: Some(tab.to_string()),
    }
}

pub fn open_session_tab(current: SessionTabState, tab: &str) -> SessionTabState {
    let preview = effective_preview(&current);
    if tab == "review" {
        let all = current.all.into_iter().filter(|item| item != tab).collect();
        return SessionTabState {
            all,
            active: Some(tab.to_string()),
            preview,
        };
    }
    if tab == "context" {
        let mut all = vec![tab.to_string()];
        all.extend(current.all.into_iter().filter(|item| item != tab));
        return SessionTabState {
            all,
            active: Some(tab.to_string()),
            preview,
        };
    }

    let preview_index = preview
        .as_deref()
        .and_then(|value| position(&current.all, value));
    let existing_index = position(&current.all, tab);
    if existing_index.is_some() {
        if preview_index.is_none() || preview.as_deref() == Some(tab) {
            return SessionTabState {
                all: current.all,
                active: Some(tab.to_string()),
                preview: None,
            };
        }
        let all = current
            .all
            .into_iter()
            .filter(|item| Some(item.as_str()) != preview.as_deref())
            .collect();
        return SessionTabState {
            all,
            active: Some(tab.to_string()),
            preview: None,
        };
    }

    if preview_index.is_none() {
        let mut all = current.all;
        all.push(tab.to_string());
        return SessionTabState {
            all,
            active: Some(tab.to_string()),
            preview: None,
        };
    }

    let index = preview_index.unwrap();
    let all = current
        .all
        .into_iter()
        .enumerate()
        .map(|(i, item)| if i == index { tab.to_string() } else { item })
        .collect();
    SessionTabState {
        all,
        active: Some(tab.to_string()),
        preview: None,
    }
}

pub fn close_session_tab(current: SessionTabState, tab: &str) -> SessionTabState {
    if tab == "review" {
        if current.active.as_deref() != Some(tab) {
            return current;
        }
        return SessionTabState {
            all: current.all.clone(),
            active: current.all.first().cloned(),
            preview: current.preview,
        };
    }

    let all: Vec<String> = current
        .all
        .iter()
        .filter(|item| item.as_str() != tab)
        .cloned()
        .collect();
    let preview = if current.preview.as_deref() == Some(tab) {
        None
    } else {
        current.preview.clone()
    };
    if current.active.as_deref() != Some(tab) {
        return SessionTabState {
            all,
            active: current.active,
            preview,
        };
    }

    let index = position(&current.all, tab).unwrap_or(0);
    let active = if index > 0 {
        current.all.get(index - 1).cloned()
    } else {
        None
    }
    .or_else(|| current.all.get(index + 1).cloned())
    .or_else(|| all.first().cloned());
    SessionTabState {
        all,
        active,
        preview,
    }
}
