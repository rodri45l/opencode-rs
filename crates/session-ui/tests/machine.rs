//! Port of packages/session-ui/src/v2/components/prompt-input/machine.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/v2/components/prompt-input/machine.ts; see docs/TEST-PORT.md.

use opencode_session_ui::prompt_input_machine::{
    create_prompt_input_v2_interaction_state, transition_prompt_input_v2, Command, Event, Focus,
    InteractionState, Mode, Popover,
};
use opencode_session_ui::prompt_types::{
    ContextItems, PersistedState, PromptPart, Suggestion, SuggestionKind,
};

fn persisted(value: &str) -> PersistedState {
    PersistedState {
        prompt: vec![PromptPart::Text {
            content: value.to_string(),
            start: 0,
            end: value.chars().count(),
        }],
        cursor: Some(value.chars().count()),
        model: None,
        context: ContextItems::default(),
    }
}

fn command(id: &str, label: &str) -> Suggestion {
    Suggestion {
        id: id.to_string(),
        kind: SuggestionKind::Command,
        label: label.to_string(),
        path: None,
    }
}

fn input(value: &str, persist: bool) -> Event {
    Event::InputChanged {
        value: value.to_string(),
        persist,
    }
}

fn key(key: &str, ctrl: bool, ids: Vec<&str>, empty: bool) -> Event {
    Event::KeyDown {
        key: key.to_string(),
        ctrl,
        composing: false,
        ids: ids.into_iter().map(str::to_string).collect(),
        empty,
    }
}

#[test]
fn opens_inline_commands_only_when_slash_is_the_entire_prompt() {
    let state = create_prompt_input_v2_interaction_state();
    let open = transition_prompt_input_v2(state.clone(), input("/re", true), &persisted(""));
    let closed = transition_prompt_input_v2(state, input("explain /re", true), &persisted(""));

    assert_eq!(
        open.state.popover,
        Popover::CommandInline {
            query: "re".to_string(),
            active_id: None
        }
    );
    assert_eq!(closed.state.popover, Popover::Closed);
}

#[test]
fn completes_nested_slash_command_names() {
    let open = transition_prompt_input_v2(
        create_prompt_input_v2_interaction_state(),
        input("/review/", true),
        &persisted(""),
    );
    let item = command("review", "/review/nested");
    let selected = transition_prompt_input_v2(
        open.state.clone(),
        Event::PopoverSelect { item },
        &persisted("/review/"),
    );

    assert_eq!(
        open.state.popover,
        Popover::CommandInline {
            query: "review/".to_string(),
            active_id: None
        }
    );
    assert!(selected.commands.contains(&Command::DraftSetText {
        value: "/review/nested ".to_string()
    }));
}

#[test]
fn opens_context_completion_at_the_cursor() {
    let value = "alpha @sr omega";
    let mut state = persisted(value);
    state.cursor = Some(9);

    let result = transition_prompt_input_v2(
        create_prompt_input_v2_interaction_state(),
        input(value, false),
        &state,
    );

    assert_eq!(
        result.state.popover,
        Popover::Context {
            query: "sr".to_string(),
            active_id: None
        }
    );
}

#[test]
fn enters_shell_mode_from_an_initial_exclamation_mark() {
    let result = transition_prompt_input_v2(
        create_prompt_input_v2_interaction_state(),
        input("!", false),
        &persisted("!"),
    );

    assert_eq!(result.state.mode, Mode::Shell);
    assert!(result.commands.contains(&Command::DraftSetText {
        value: String::new()
    }));
}

#[test]
fn leaves_shell_mode_with_escape() {
    let state = InteractionState {
        mode: Mode::Shell,
        ..create_prompt_input_v2_interaction_state()
    };
    let result =
        transition_prompt_input_v2(state, key("Escape", false, vec![], false), &persisted(""));

    assert_eq!(result.state.mode, Mode::Normal);
    assert!(result.handled);
}

#[test]
fn leaves_shell_mode_with_backspace_when_empty() {
    let state = InteractionState {
        mode: Mode::Shell,
        ..create_prompt_input_v2_interaction_state()
    };
    let result =
        transition_prompt_input_v2(state, key("Backspace", false, vec![], true), &persisted(""));

    assert_eq!(result.state.mode, Mode::Normal);
    assert!(result.handled);
}

#[test]
fn closes_a_popover_with_ctrl_g_before_stopping_a_run() {
    let state = InteractionState {
        popover: Popover::Context {
            query: String::new(),
            active_id: Some("first".to_string()),
        },
        ..create_prompt_input_v2_interaction_state()
    };
    let result =
        transition_prompt_input_v2(state, key("g", true, vec!["first"], false), &persisted(""));

    assert_eq!(result.state.popover, Popover::Closed);
    assert!(result.handled);
}

#[test]
fn opens_the_searchable_command_menu_for_a_populated_draft() {
    let result = transition_prompt_input_v2(
        create_prompt_input_v2_interaction_state(),
        Event::CommandsOpen,
        &persisted("existing text"),
    );

    assert_eq!(
        result.state.popover,
        Popover::CommandMenu {
            query: String::new(),
            active_id: None
        }
    );
    assert_eq!(result.state.focus, Focus::CommandSearch);
}

#[test]
fn prepends_a_menu_command_and_preserves_existing_text_as_arguments() {
    let open = transition_prompt_input_v2(
        create_prompt_input_v2_interaction_state(),
        Event::CommandsOpen,
        &persisted("existing text"),
    );
    let selected = transition_prompt_input_v2(
        open.state.clone(),
        Event::PopoverSelect {
            item: command("review", "/review"),
        },
        &persisted("existing text"),
    );

    assert!(selected.commands.contains(&Command::DraftSetText {
        value: "/review existing text".to_string()
    }));
    assert_eq!(selected.state.popover, Popover::Closed);
}

#[test]
fn stores_selected_context_files_as_prompt_file_parts() {
    let item = Suggestion {
        id: "src/index.ts".to_string(),
        kind: SuggestionKind::File,
        label: "index.ts".to_string(),
        path: Some("src/index.ts".to_string()),
    };
    let state = InteractionState {
        popover: Popover::Context {
            query: "index".to_string(),
            active_id: None,
        },
        ..create_prompt_input_v2_interaction_state()
    };

    let selected = transition_prompt_input_v2(
        state,
        Event::PopoverSelect { item: item.clone() },
        &persisted("@index"),
    );

    assert!(selected.commands.contains(&Command::MentionAdd { item }));
}

#[test]
fn loops_active_popover_items_with_arrow_keys() {
    let state = InteractionState {
        popover: Popover::Context {
            query: String::new(),
            active_id: Some("second".to_string()),
        },
        ..create_prompt_input_v2_interaction_state()
    };
    let result = transition_prompt_input_v2(
        state,
        key("ArrowDown", false, vec!["first", "second"], false),
        &persisted(""),
    );

    assert_eq!(
        result.state.popover,
        Popover::Context {
            query: String::new(),
            active_id: Some("first".to_string())
        }
    );
    assert!(result.handled);
}
