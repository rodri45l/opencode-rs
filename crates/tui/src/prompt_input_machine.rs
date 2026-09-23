//! Prompt-input interaction machine.
//!
//! Derived from the observable behaviour pinned by
//! `packages/session-ui/src/v2/components/prompt-input/machine.ts` (upstream
//! 18ef3cc): slash commands open inline only when the prompt is just a slash
//! query, context completion follows the cursor, shell mode toggles on
//! `!`/Escape/backspace, ctrl-g closes a popover, and the command menu prepends
//! or stores selected suggestions. Solid store/accessor plumbing is replaced by
//! plain state values.

use std::fmt;

/// Error raised by the interaction machine.
#[derive(Debug, PartialEq, Eq)]
pub struct NotImplemented(pub &'static str);

impl fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for NotImplemented {}

/// Result alias for the interaction machine.
pub type PortResult<T> = Result<T, NotImplemented>;

/// Message used by the ported tests when expecting success.
pub const NOTE: &str = "session-ui prompt-input machine";

/// One persisted prompt part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptItem {
    Text {
        content: String,
        start: i64,
        end: i64,
    },
    File {
        path: String,
        content: String,
        start: i64,
        end: i64,
    },
    Image {
        id: String,
        filename: String,
        mime: String,
    },
}

/// One persisted context item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextItem {
    pub key: String,
    pub kind: String,
    pub path: String,
}

/// The persisted prompt context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptContext {
    pub items: Vec<ContextItem>,
}

/// The persisted prompt state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedState {
    pub prompt: Vec<PromptItem>,
    pub cursor: i64,
    pub context: PromptContext,
}

/// A completion suggestion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub path: Option<String>,
}

/// The active popover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Popover {
    Closed,
    CommandInline {
        query: String,
    },
    Context {
        query: String,
        active_id: Option<String>,
    },
    CommandMenu {
        query: String,
    },
}

/// The prompt input mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Shell,
}

/// The interaction state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionState {
    pub popover: Popover,
    pub mode: Mode,
    pub focus: Option<String>,
}

/// A side effect emitted by a transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    SetText { value: String },
    MentionAdd { item: Suggestion },
}

/// An interaction event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    InputChanged {
        value: String,
        persist: bool,
    },
    KeyDown {
        key: String,
        ctrl: bool,
        composing: bool,
        ids: Vec<String>,
        empty: bool,
    },
    CommandsOpen,
    PopoverSelect {
        item: Suggestion,
    },
}

/// The result of an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub state: InteractionState,
    pub commands: Vec<Command>,
    pub handled: bool,
}

/// Initial interaction state.
pub fn create_prompt_input_v2_interaction_state() -> InteractionState {
    InteractionState {
        popover: Popover::Closed,
        mode: Mode::Normal,
        focus: None,
    }
}

fn prompt_text(persisted: &PersistedState) -> String {
    persisted
        .prompt
        .iter()
        .filter_map(|part| match part {
            PromptItem::Text { content, .. } => Some(content.clone()),
            _ => None,
        })
        .collect()
}

fn populated(persisted: &PersistedState) -> bool {
    !prompt_text(persisted).trim().is_empty()
        || !persisted.context.items.is_empty()
        || persisted
            .prompt
            .iter()
            .any(|part| matches!(part, PromptItem::File { .. } | PromptItem::Image { .. }))
}

fn replace_trigger(value: &str, trigger: char, replacement: &str) -> String {
    match value.find(trigger) {
        Some(index) => format!("{}{}", &value[..index], replacement),
        None => replacement.to_string(),
    }
}

fn changed(
    state: InteractionState,
    commands: Vec<Command>,
    handled: bool,
) -> PortResult<Transition> {
    Ok(Transition {
        state,
        commands,
        handled,
    })
}

fn context_query(value: &str, cursor: i64) -> Option<String> {
    let end = (cursor.max(0) as usize).min(value.len());
    let slice = &value[..end];
    let at = slice.rfind('@')?;
    if at > 0 {
        let before = slice.as_bytes()[at - 1];
        if before != b' ' && before != b'\t' && before != b'\n' {
            return None;
        }
    }
    let query = &slice[at + 1..];
    if query.contains('@') || query.contains(char::is_whitespace) {
        return None;
    }
    Some(query.to_string())
}

fn command_query(value: &str) -> Option<String> {
    let rest = value.strip_prefix('/')?;
    if rest.contains(char::is_whitespace) {
        return None;
    }
    Some(rest.to_string())
}

fn input_changed(
    state: InteractionState,
    value: &str,
    persist: bool,
    cursor: i64,
) -> PortResult<Transition> {
    let mut set_text: Vec<Command> = Vec::new();
    if persist {
        set_text.push(Command::SetText {
            value: value.to_string(),
        });
    }
    if state.mode == Mode::Normal && value == "!" {
        return changed(
            InteractionState {
                mode: Mode::Shell,
                popover: Popover::Closed,
                focus: Some("editor".into()),
            },
            vec![Command::SetText {
                value: String::new(),
            }],
            false,
        );
    }
    if let Some(query) = context_query(value, cursor) {
        return changed(
            InteractionState {
                popover: Popover::Context {
                    query,
                    active_id: None,
                },
                focus: Some("editor".into()),
                ..state
            },
            set_text,
            false,
        );
    }
    if let Some(query) = command_query(value) {
        return changed(
            InteractionState {
                popover: Popover::CommandInline { query },
                focus: Some("editor".into()),
                ..state
            },
            set_text,
            false,
        );
    }
    let popover = match state.popover {
        Popover::CommandMenu { .. } => state.popover,
        _ => Popover::Closed,
    };
    changed(
        InteractionState {
            popover,
            focus: Some("editor".into()),
            ..state
        },
        set_text,
        false,
    )
}

fn open_commands(state: InteractionState, persisted: &PersistedState) -> PortResult<Transition> {
    if !populated(persisted) {
        return changed(
            InteractionState {
                popover: Popover::CommandInline {
                    query: String::new(),
                },
                focus: Some("editor".into()),
                ..state
            },
            vec![Command::SetText {
                value: format!("{}/", prompt_text(persisted)),
            }],
            false,
        );
    }
    changed(
        InteractionState {
            popover: Popover::CommandMenu {
                query: String::new(),
            },
            focus: Some("command-search".into()),
            ..state
        },
        Vec::new(),
        false,
    )
}

fn suggestion_selected(
    state: InteractionState,
    item: &Suggestion,
    persisted: &PersistedState,
) -> PortResult<Transition> {
    let current = prompt_text(persisted);
    let mut commands: Vec<Command> = Vec::new();
    if item.kind == "command" {
        let value = if matches!(state.popover, Popover::CommandMenu { .. }) {
            if current.trim().is_empty() {
                format!("{} ", item.label)
            } else {
                format!("{} {}", item.label, current.trim())
            }
        } else {
            replace_trigger(&current, '/', &format!("{} ", item.label))
        };
        commands.push(Command::SetText { value });
    } else {
        commands.push(Command::MentionAdd { item: item.clone() });
    }
    changed(
        InteractionState {
            popover: Popover::Closed,
            focus: Some("editor".into()),
            ..state
        },
        commands,
        false,
    )
}

fn key_down(state: InteractionState, event: &Event) -> PortResult<Transition> {
    let Event::KeyDown {
        key,
        ctrl,
        composing,
        ids,
        empty,
    } = event
    else {
        return changed(state, Vec::new(), false);
    };

    if *ctrl && key.eq_ignore_ascii_case("g") {
        if matches!(state.popover, Popover::Closed) {
            return changed(state, Vec::new(), false);
        }
        return changed(
            InteractionState {
                popover: Popover::Closed,
                focus: Some("editor".into()),
                ..state
            },
            Vec::new(),
            true,
        );
    }

    if matches!(state.popover, Popover::Closed) {
        if state.mode == Mode::Shell && (key == "Escape" || (key == "Backspace" && *empty)) {
            return changed(
                InteractionState {
                    mode: Mode::Normal,
                    ..state
                },
                Vec::new(),
                true,
            );
        }
        return changed(state, Vec::new(), false);
    }

    if key == "Escape" {
        return changed(
            InteractionState {
                popover: Popover::Closed,
                focus: Some("editor".into()),
                ..state
            },
            Vec::new(),
            true,
        );
    }

    if key == "Tab" || (key == "Enter" && !*composing) {
        return changed(state, Vec::new(), true);
    }

    let direction: i64 = if key == "ArrowDown" || (*ctrl && key == "n") {
        1
    } else if key == "ArrowUp" || (*ctrl && key == "p") {
        -1
    } else {
        0
    };
    if direction == 0 || ids.is_empty() {
        return changed(state, Vec::new(), false);
    }
    let active = match &state.popover {
        Popover::Context { active_id, .. } => active_id.clone(),
        Popover::CommandInline { .. } | Popover::CommandMenu { .. } => None,
        Popover::Closed => None,
    };
    let current = active
        .and_then(|id| ids.iter().position(|item| *item == id))
        .map(|index| index as i64)
        .unwrap_or(-1);
    let length = ids.len() as i64;
    let index = if current < 0 {
        if direction == 1 {
            0
        } else {
            length - 1
        }
    } else {
        (current + direction + length) % length
    };
    let next_id = ids[index as usize].clone();
    let popover = match state.popover {
        Popover::Context { query, .. } => Popover::Context {
            query,
            active_id: Some(next_id),
        },
        Popover::CommandInline { query } => Popover::CommandInline { query },
        Popover::CommandMenu { query } => Popover::CommandMenu { query },
        Popover::Closed => Popover::Closed,
    };
    changed(InteractionState { popover, ..state }, Vec::new(), true)
}

/// Apply one event to the interaction state.
pub fn transition_prompt_input_v2(
    state: InteractionState,
    event: Event,
    persisted: PersistedState,
) -> PortResult<Transition> {
    match &event {
        Event::KeyDown { .. } => key_down(state, &event),
        Event::InputChanged { value, persist } => {
            input_changed(state, value, *persist, persisted.cursor)
        }
        Event::CommandsOpen => open_commands(state, &persisted),
        Event::PopoverSelect { item } => suggestion_selected(state, item, &persisted),
    }
}
