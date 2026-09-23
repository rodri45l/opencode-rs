//! Titlebar navigation history (port of packages/app/src/components/titlebar-history.ts).

pub const MAX_TITLEBAR_HISTORY: usize = 100;

#[derive(Clone, Debug, PartialEq)]
pub struct TitlebarHistory {
    pub stack: Vec<String>,
    pub index: i64,
    pub action: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathResult {
    pub to: String,
    pub state: TitlebarHistory,
}

pub fn trim_history(stack: Vec<String>, index: i64, max: usize) -> (Vec<String>, i64) {
    if stack.len() <= max {
        return (stack, index);
    }
    let cut = stack.len() - max;
    (stack[cut..].to_vec(), std::cmp::max(0, index - cut as i64))
}

pub fn push_path(state: TitlebarHistory, path: &str, max: usize) -> TitlebarHistory {
    let mut stack: Vec<String> =
        state.stack[..=(state.index as usize).min(state.stack.len() - 1)].to_vec();
    stack.push(path.to_string());
    let next_index = stack.len() as i64 - 1;
    let (stack, index) = trim_history(stack, next_index, max);
    TitlebarHistory {
        stack,
        index,
        action: None,
    }
}

pub fn apply_path(state: TitlebarHistory, current: &str, max: usize) -> TitlebarHistory {
    if state.stack.is_empty() {
        let stack = if current == "/" {
            vec!["/".to_string()]
        } else {
            vec!["/".to_string(), current.to_string()]
        };
        return TitlebarHistory {
            index: stack.len() as i64 - 1,
            stack,
            action: None,
        };
    }

    let active = state.stack.get(state.index as usize).cloned();
    if active.as_deref() == Some(current) {
        if state.action.is_none() {
            return state;
        }
        return TitlebarHistory {
            action: None,
            ..state
        };
    }

    if state.action.is_some() {
        return TitlebarHistory {
            action: None,
            ..state
        };
    }

    push_path(state, current, max)
}

pub fn back_path(state: TitlebarHistory) -> Option<PathResult> {
    if state.index <= 0 {
        return None;
    }
    let index = state.index - 1;
    let to = state.stack.get(index as usize)?.clone();
    Some(PathResult {
        to,
        state: TitlebarHistory {
            index,
            action: Some("back".to_string()),
            ..state
        },
    })
}

pub fn forward_path(state: TitlebarHistory) -> Option<PathResult> {
    if state.index >= state.stack.len() as i64 - 1 {
        return None;
    }
    let index = state.index + 1;
    let to = state.stack.get(index as usize)?.clone();
    Some(PathResult {
        to,
        state: TitlebarHistory {
            index,
            action: Some("forward".to_string()),
            ..state
        },
    })
}
