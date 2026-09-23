//! Port of packages/opencode/test/cli/run/prompt.shared.test.ts (upstream 18ef3cc).
//!
//! RED-first: the run prompt-history helpers in `cli/cmd/run/prompt.shared` are
//! not implemented in this crate. Reference behaviour is pinned against local
//! typed stubs.

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunPrompt {
    text: String,
}

impl RunPrompt {
    fn new(text: &str) -> Self {
        RunPrompt {
            text: text.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PromptHistory {
    items: Vec<RunPrompt>,
    index: Option<usize>,
    draft: String,
}

impl PromptHistory {
    fn texts(&self) -> Vec<String> {
        self.items.iter().map(|p| p.text.clone()).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MoveResult {
    state: PromptHistory,
    apply: bool,
    text: String,
    cursor: usize,
}

fn display_width(text: &str) -> usize {
    text.chars().map(|c| if is_wide(c) { 2 } else { 1 }).sum()
}

fn is_wide(c: char) -> bool {
    matches!(
        c as u32,
        0x1100..=0x115F
            | 0x2E80..=0xA4CF
            | 0xAC00..=0xD7A3
            | 0xF900..=0xFAFF
            | 0xFE30..=0xFE4F
            | 0xFF00..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x20000..=0x3FFFD
    )
}

fn create_prompt_history(prompts: Vec<RunPrompt>) -> PromptHistory {
    let mut items: Vec<RunPrompt> = Vec::new();
    for prompt in prompts {
        if prompt.text.trim().is_empty() {
            continue;
        }
        if items.last().map(|last| last.text == prompt.text) == Some(true) {
            continue;
        }
        items.push(prompt);
    }
    PromptHistory {
        items,
        index: None,
        draft: String::new(),
    }
}

fn push_prompt_history(mut history: PromptHistory, prompt: RunPrompt) -> PromptHistory {
    if prompt.text.trim().is_empty() {
        return history;
    }
    if history.items.last().map(|last| last.text == prompt.text) == Some(true) {
        return history;
    }
    history.items.push(prompt);
    history
}

fn move_prompt_history(
    mut history: PromptHistory,
    direction: i32,
    draft: &str,
    cursor: usize,
) -> MoveResult {
    let noop = MoveResult {
        state: history.clone(),
        apply: false,
        text: String::new(),
        cursor: 0,
    };
    if history.items.is_empty() {
        return noop;
    }
    if direction == -1 && cursor != 0 {
        return noop;
    }
    if direction == 1 && cursor != display_width(draft) {
        return noop;
    }
    if history.index.is_none() {
        if direction == 1 {
            return noop;
        }
        let index = history.items.len() - 1;
        history.index = Some(index);
        history.draft = draft.to_string();
        return MoveResult {
            text: history.items[index].text.clone(),
            state: history,
            apply: true,
            cursor: 0,
        };
    }
    let index = history.index.unwrap() as i64 + direction as i64;
    if index < 0 {
        return noop;
    }
    if index as usize >= history.items.len() {
        let text = history.draft.clone();
        let cursor = display_width(&text);
        history.index = None;
        return MoveResult {
            text,
            state: history,
            apply: true,
            cursor,
        };
    }
    let index = index as usize;
    history.index = Some(index);
    let text = history.items[index].text.clone();
    let cursor = if direction == -1 {
        0
    } else {
        display_width(&text)
    };
    MoveResult {
        text,
        state: history,
        apply: true,
        cursor,
    }
}

fn is_exit_command(value: &str) -> bool {
    matches!(
        value.trim().to_lowercase().as_str(),
        "/exit" | "/quit" | ":q"
    )
}

fn is_new_command(value: &str) -> bool {
    value.trim().to_lowercase() == "/new"
}

#[test]
fn filters_blank_prompts_and_dedupes_consecutive_history() {
    let out = create_prompt_history(vec![
        RunPrompt::new("   "),
        RunPrompt::new("one"),
        RunPrompt::new("one"),
        RunPrompt::new("two"),
        RunPrompt::new("one"),
    ]);
    assert_eq!(out.texts(), vec!["one", "two", "one"]);
    assert_eq!(out.index, None);
    assert_eq!(out.draft, "");
}

#[test]
fn push_ignores_blanks_and_dedupes_only_the_latest_item() {
    let base = create_prompt_history(vec![RunPrompt::new("one")]);
    assert_eq!(
        push_prompt_history(base.clone(), RunPrompt::new("   ")).texts(),
        vec!["one"]
    );
    assert_eq!(
        push_prompt_history(base.clone(), RunPrompt::new("one")).texts(),
        vec!["one"]
    );
    assert_eq!(
        push_prompt_history(base, RunPrompt::new("two")).texts(),
        vec!["one", "two"]
    );
}

#[test]
fn moves_through_history_only_at_input_boundaries_and_restores_draft() {
    let base = create_prompt_history(vec![RunPrompt::new("one"), RunPrompt::new("two")]);

    let boundary = move_prompt_history(base.clone(), -1, "draft", 1);
    assert_eq!(
        boundary,
        MoveResult {
            state: base.clone(),
            apply: false,
            text: String::new(),
            cursor: 0,
        }
    );

    let up = move_prompt_history(base, -1, "draft", 0);
    assert!(up.apply);
    assert_eq!(up.text, "two");
    assert_eq!(up.cursor, 0);
    assert_eq!(up.state.index, Some(1));
    assert_eq!(up.state.draft, "draft");

    let older = move_prompt_history(up.state, -1, "two", 0);
    assert!(older.apply);
    assert_eq!(older.text, "one");
    assert_eq!(older.cursor, 0);
    assert_eq!(older.state.index, Some(0));

    let newer = move_prompt_history(older.state, 1, "one", 3);
    assert!(newer.apply);
    assert_eq!(newer.text, "two");
    assert_eq!(newer.cursor, 3);
    assert_eq!(newer.state.index, Some(1));

    let draft = move_prompt_history(newer.state, 1, "two", 3);
    assert!(draft.apply);
    assert_eq!(draft.text, "draft");
    assert_eq!(draft.cursor, 5);
    assert_eq!(draft.state.index, None);
}

#[test]
fn uses_display_width_cursors_for_history_restoration() {
    let base = create_prompt_history(vec![RunPrompt::new("one"), RunPrompt::new("中文")]);

    let latest = move_prompt_history(base, -1, "草稿", 0);
    assert!(latest.apply);
    assert_eq!(latest.text, "中文");
    assert_eq!(latest.cursor, 0);

    let older = move_prompt_history(latest.state, -1, "中文", 0);
    assert!(older.apply);
    assert_eq!(older.text, "one");
    assert_eq!(older.cursor, 0);

    let newer = move_prompt_history(older.state, 1, "one", 3);
    assert!(newer.apply);
    assert_eq!(newer.text, "中文");
    assert_eq!(newer.cursor, 4);

    let draft = move_prompt_history(newer.state, 1, "中文", 4);
    assert!(draft.apply);
    assert_eq!(draft.text, "草稿");
    assert_eq!(draft.cursor, 4);
}

#[test]
fn recognizes_exit_commands() {
    assert!(is_exit_command("/exit"));
    assert!(is_exit_command(" /Quit "));
    assert!(!is_exit_command("/quit now"));
}

#[test]
fn recognizes_the_new_session_command() {
    assert!(is_new_command("/new"));
    assert!(is_new_command(" /NEW "));
    assert!(!is_new_command("/new now"));
}
