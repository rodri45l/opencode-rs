//! Port of packages/session-ui/src/v2/components/prompt-input/machine.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/v2/components/prompt-input/machine.ts:
//! slash commands open inline only when the prompt is just a slash query, context
//! completion follows the cursor, shell mode toggles on `!`/Escape/backspace, ctrl-g
//! closes a popover, and the command menu prepends or stores selected suggestions.
//! Re-derived: Solid store/accessor plumbing is replaced by plain state values.
//! Red-first: the prompt-input interaction machine is not implemented.

use opencode_tui::prompt_input_machine::{
    create_prompt_input_v2_interaction_state, transition_prompt_input_v2, Command, Event,
    InteractionState, Mode, PersistedState, Popover, PromptContext, PromptItem, Suggestion, NOTE,
};

fn persisted(value: &str) -> PersistedState {
    PersistedState {
        prompt: vec![PromptItem::Text {
            content: value.into(),
            start: 0,
            end: value.len() as i64,
        }],
        cursor: value.len() as i64,
        context: PromptContext { items: Vec::new() },
    }
}

fn command() -> Suggestion {
    Suggestion {
        id: "review".into(),
        kind: "command".into(),
        label: "/review".into(),
        path: None,
    }
}

#[test]
fn opens_inline_commands_only_when_slash_is_the_entire_prompt() {
    let state = create_prompt_input_v2_interaction_state();
    let open = transition_prompt_input_v2(
        state.clone(),
        Event::InputChanged {
            value: "/re".into(),
            persist: true,
        },
        persisted(""),
    )
    .expect(NOTE);
    let closed = transition_prompt_input_v2(
        state,
        Event::InputChanged {
            value: "explain /re".into(),
            persist: true,
        },
        persisted(""),
    )
    .expect(NOTE);

    assert_eq!(
        open.state.popover,
        Popover::CommandInline { query: "re".into() }
    );
    assert_eq!(closed.state.popover, Popover::Closed);
}

#[test]
fn completes_nested_slash_command_names() {
    let open = transition_prompt_input_v2(
        create_prompt_input_v2_interaction_state(),
        Event::InputChanged {
            value: "/review/".into(),
            persist: true,
        },
        persisted(""),
    )
    .expect(NOTE);
    let item = Suggestion {
        label: "/review/nested".into(),
        ..command()
    };
    let selected = transition_prompt_input_v2(
        open.state.clone(),
        Event::PopoverSelect { item },
        persisted("/review/"),
    )
    .expect(NOTE);

    assert_eq!(
        open.state.popover,
        Popover::CommandInline {
            query: "review/".into()
        }
    );
    assert!(selected.commands.contains(&Command::SetText {
        value: "/review/nested ".into()
    }));
}

#[test]
fn opens_context_completion_at_the_cursor() {
    let value = "alpha @sr omega";
    let mut input = persisted(value);
    input.cursor = 9;

    let result = transition_prompt_input_v2(
        create_prompt_input_v2_interaction_state(),
        Event::InputChanged {
            value: value.into(),
            persist: false,
        },
        input,
    )
    .expect(NOTE);

    assert_eq!(
        result.state.popover,
        Popover::Context {
            query: "sr".into(),
            active_id: None
        }
    );
}

#[test]
fn enters_shell_mode_from_an_initial_exclamation_mark() {
    let result = transition_prompt_input_v2(
        create_prompt_input_v2_interaction_state(),
        Event::InputChanged {
            value: "!".into(),
            persist: false,
        },
        persisted("!"),
    )
    .expect(NOTE);

    assert_eq!(result.state.mode, Mode::Shell);
    assert!(result.commands.contains(&Command::SetText {
        value: String::new()
    }));
}

#[test]
fn leaves_shell_mode_with_escape() {
    let mut state = create_prompt_input_v2_interaction_state();
    state.mode = Mode::Shell;
    let result = transition_prompt_input_v2(
        state,
        Event::KeyDown {
            key: "Escape".into(),
            ctrl: false,
            composing: false,
            ids: Vec::new(),
            empty: false,
        },
        persisted(""),
    )
    .expect(NOTE);

    assert_eq!(result.state.mode, Mode::Normal);
    assert!(result.handled);
}

#[test]
fn leaves_shell_mode_with_backspace_when_empty() {
    let mut state = create_prompt_input_v2_interaction_state();
    state.mode = Mode::Shell;
    let result = transition_prompt_input_v2(
        state,
        Event::KeyDown {
            key: "Backspace".into(),
            ctrl: false,
            composing: false,
            ids: Vec::new(),
            empty: true,
        },
        persisted(""),
    )
    .expect(NOTE);

    assert_eq!(result.state.mode, Mode::Normal);
    assert!(result.handled);
}

#[test]
fn closes_a_popover_with_ctrl_g_before_stopping_a_run() {
    let mut state = create_prompt_input_v2_interaction_state();
    state.popover = Popover::Context {
        query: String::new(),
        active_id: Some("first".into()),
    };
    let result = transition_prompt_input_v2(
        state,
        Event::KeyDown {
            key: "g".into(),
            ctrl: true,
            composing: false,
            ids: vec!["first".into()],
            empty: false,
        },
        persisted(""),
    )
    .expect(NOTE);

    assert_eq!(result.state.popover, Popover::Closed);
    assert!(result.handled);
}

#[test]
fn opens_the_searchable_command_menu_for_a_populated_draft() {
    let result = transition_prompt_input_v2(
        create_prompt_input_v2_interaction_state(),
        Event::CommandsOpen,
        persisted("existing text"),
    )
    .expect(NOTE);

    assert_eq!(
        result.state.popover,
        Popover::CommandMenu {
            query: String::new()
        }
    );
    assert_eq!(result.state.focus.as_deref(), Some("command-search"));
}

#[test]
fn prepends_a_menu_command_and_preserves_existing_text_as_arguments() {
    let open = transition_prompt_input_v2(
        create_prompt_input_v2_interaction_state(),
        Event::CommandsOpen,
        persisted("existing text"),
    )
    .expect(NOTE);
    let selected = transition_prompt_input_v2(
        open.state.clone(),
        Event::PopoverSelect { item: command() },
        persisted("existing text"),
    )
    .expect(NOTE);

    assert!(selected.commands.contains(&Command::SetText {
        value: "/review existing text".into()
    }));
    assert_eq!(selected.state.popover, Popover::Closed);
}

#[test]
fn stores_selected_context_files_as_prompt_file_parts() {
    let item = Suggestion {
        id: "src/index.ts".into(),
        kind: "file".into(),
        label: "index.ts".into(),
        path: Some("src/index.ts".into()),
    };
    let mut state = create_prompt_input_v2_interaction_state();
    state.popover = Popover::Context {
        query: "index".into(),
        active_id: None,
    };

    let selected = transition_prompt_input_v2(
        state,
        Event::PopoverSelect { item: item.clone() },
        persisted("@index"),
    )
    .expect(NOTE);

    assert!(selected.commands.contains(&Command::MentionAdd { item }));
}

#[test]
fn loops_active_popover_items_with_arrow_keys() {
    let mut state: InteractionState = create_prompt_input_v2_interaction_state();
    state.popover = Popover::Context {
        query: String::new(),
        active_id: Some("second".into()),
    };
    let result = transition_prompt_input_v2(
        state,
        Event::KeyDown {
            key: "ArrowDown".into(),
            ctrl: false,
            composing: false,
            ids: vec!["first".into(), "second".into()],
            empty: false,
        },
        persisted(""),
    )
    .expect(NOTE);

    assert_eq!(
        result.state.popover,
        Popover::Context {
            query: String::new(),
            active_id: Some("first".into())
        }
    );
    assert!(result.handled);
}
