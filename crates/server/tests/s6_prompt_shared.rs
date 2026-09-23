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

fn create_prompt_history(_prompts: Vec<RunPrompt>) -> PromptHistory {
    PromptHistory {
        items: Vec::new(),
        index: None,
        draft: String::new(),
    }
}

fn push_prompt_history(history: PromptHistory, _prompt: RunPrompt) -> PromptHistory {
    history
}

fn move_prompt_history(
    history: PromptHistory,
    _direction: i32,
    _draft: &str,
    _cursor: usize,
) -> MoveResult {
    MoveResult {
        state: history,
        apply: false,
        text: String::new(),
        cursor: 0,
    }
}

fn is_exit_command(_value: &str) -> bool {
    false
}

fn is_new_command(_value: &str) -> bool {
    false
}

#[test]
#[ignore = "porting: cli run prompt history not implemented"]
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
#[ignore = "porting: cli run prompt history not implemented"]
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
#[ignore = "porting: cli run prompt history not implemented"]
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
#[ignore = "porting: cli run prompt history not implemented"]
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
#[ignore = "porting: cli run prompt history not implemented"]
fn recognizes_exit_commands() {
    assert!(is_exit_command("/exit"));
    assert!(is_exit_command(" /Quit "));
    assert!(!is_exit_command("/quit now"));
}

#[test]
#[ignore = "porting: cli run prompt history not implemented"]
fn recognizes_the_new_session_command() {
    assert!(is_new_command("/new"));
    assert!(is_new_command(" /NEW "));
    assert!(!is_new_command("/new now"));
}
