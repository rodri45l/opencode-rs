//! Prompt-input v2 interaction state machine.
//!
//! Port of packages/session-ui/src/v2/components/prompt-input/machine.ts
//! behaviour (upstream 18ef3cc).

use crate::prompt_types::{PersistedState, PromptPart, Suggestion, SuggestionKind};

/// The prompt mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Shell,
}

/// The popover shown above the prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Popover {
    Closed,
    Context {
        query: String,
        active_id: Option<String>,
    },
    CommandInline {
        query: String,
        active_id: Option<String>,
    },
    CommandMenu {
        query: String,
        active_id: Option<String>,
    },
}

impl Popover {
    fn active_id(&self) -> Option<&str> {
        match self {
            Popover::Closed => None,
            Popover::Context { active_id, .. }
            | Popover::CommandInline { active_id, .. }
            | Popover::CommandMenu { active_id, .. } => active_id.as_deref(),
        }
    }

    fn with_query(&self, query: String) -> Popover {
        match self {
            Popover::Closed => Popover::Closed,
            Popover::Context { .. } => Popover::Context {
                query,
                active_id: None,
            },
            Popover::CommandInline { .. } => Popover::CommandInline {
                query,
                active_id: None,
            },
            Popover::CommandMenu { .. } => Popover::CommandMenu {
                query,
                active_id: None,
            },
        }
    }

    fn with_active_id(&self, active_id: String) -> Popover {
        match self {
            Popover::Closed => Popover::Closed,
            Popover::Context { query, .. } => Popover::Context {
                query: query.clone(),
                active_id: Some(active_id),
            },
            Popover::CommandInline { query, .. } => Popover::CommandInline {
                query: query.clone(),
                active_id: Some(active_id),
            },
            Popover::CommandMenu { query, .. } => Popover::CommandMenu {
                query: query.clone(),
                active_id: Some(active_id),
            },
        }
    }
}

/// Drag state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drag {
    Idle,
    Active,
}

/// Focus target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Editor,
    CommandSearch,
    External,
}

/// A saved history entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub prompt: Vec<PromptPart>,
}

/// The interaction state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionState {
    pub mode: Mode,
    pub popover: Popover,
    pub drag: Drag,
    pub focus: Focus,
    pub active_context_id: Option<String>,
    pub history_index: i64,
    pub saved_history: Option<HistoryEntry>,
}

/// Create the initial interaction state.
pub fn create_prompt_input_v2_interaction_state() -> InteractionState {
    InteractionState {
        mode: Mode::Normal,
        popover: Popover::Closed,
        drag: Drag::Idle,
        focus: Focus::External,
        active_context_id: None,
        history_index: -1,
        saved_history: None,
    }
}

/// Which filter a popover change applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterPopover {
    Command,
    Context,
}

/// A command emitted by a transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    DraftSetText {
        value: String,
    },
    MentionAdd {
        item: Suggestion,
    },
    PopoverFilter {
        popover: FilterPopover,
        query: String,
    },
    SuggestionSelect {
        id: String,
    },
    FocusEditor,
    FocusCommandSearch,
}

/// An input event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    InputChanged {
        value: String,
        persist: bool,
    },
    CommandsOpen,
    ContextOpen,
    PopoverQuery {
        value: String,
    },
    PopoverResults {
        ids: Vec<String>,
    },
    PopoverActive {
        id: String,
    },
    PopoverClose,
    PopoverSelect {
        item: Suggestion,
    },
    KeyDown {
        key: String,
        ctrl: bool,
        composing: bool,
        ids: Vec<String>,
        empty: bool,
    },
    ModeShell,
    ModeNormal,
    DragEnter,
    DragLeave,
    FocusEditor,
    FocusExternal,
    ContextActive {
        id: String,
    },
}

/// The result of a transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub state: InteractionState,
    pub commands: Vec<Command>,
    pub handled: bool,
}

/// Advance the interaction state for an event.
pub fn transition_prompt_input_v2(
    state: InteractionState,
    event: Event,
    persisted: &PersistedState,
) -> Transition {
    match event {
        Event::InputChanged { value, persist } => {
            input_changed(state, &value, persist, persisted.cursor)
        }
        Event::CommandsOpen => open_commands(state, persisted),
        Event::ContextOpen => open_context(state, persisted),
        Event::PopoverQuery { value } => query_changed(state, &value),
        Event::PopoverResults { ids } => results_changed(state, &ids),
        Event::PopoverActive { id } => active_changed(state, &id),
        Event::PopoverClose => changed(
            InteractionState {
                popover: Popover::Closed,
                ..state
            },
            Vec::new(),
            false,
        ),
        Event::PopoverSelect { item } => suggestion_selected(state, &item, persisted),
        Event::KeyDown {
            key,
            ctrl,
            composing,
            ids,
            empty,
        } => key_down(state, &key, ctrl, composing, &ids, empty),
        Event::ModeShell => changed(
            InteractionState {
                mode: Mode::Shell,
                popover: Popover::Closed,
                ..state
            },
            Vec::new(),
            false,
        ),
        Event::ModeNormal => changed(
            InteractionState {
                mode: Mode::Normal,
                ..state
            },
            Vec::new(),
            false,
        ),
        Event::DragEnter => changed(
            InteractionState {
                drag: Drag::Active,
                ..state
            },
            Vec::new(),
            false,
        ),
        Event::DragLeave => changed(
            InteractionState {
                drag: Drag::Idle,
                ..state
            },
            Vec::new(),
            false,
        ),
        Event::FocusEditor => changed(
            InteractionState {
                focus: Focus::Editor,
                ..state
            },
            Vec::new(),
            false,
        ),
        Event::FocusExternal => changed(
            InteractionState {
                focus: Focus::External,
                ..state
            },
            Vec::new(),
            false,
        ),
        Event::ContextActive { id } => {
            let active_context_id = if state.active_context_id.as_deref() == Some(&id) {
                None
            } else {
                Some(id)
            };
            changed(
                InteractionState {
                    active_context_id,
                    ..state
                },
                Vec::new(),
                false,
            )
        }
    }
}

fn input_changed(
    state: InteractionState,
    value: &str,
    persist: bool,
    cursor: Option<usize>,
) -> Transition {
    let mut set_text = Vec::new();
    if persist {
        set_text.push(Command::DraftSetText {
            value: value.to_string(),
        });
    }
    if state.mode == Mode::Normal && value == "!" {
        return changed(
            InteractionState {
                mode: Mode::Shell,
                popover: Popover::Closed,
                focus: Focus::Editor,
                ..state
            },
            vec![Command::DraftSetText {
                value: String::new(),
            }],
            false,
        );
    }

    let end = cursor.unwrap_or(value.len()).min(value.len());
    let prefix = value.get(..end).unwrap_or(value);
    if let Some(query) = context_query(prefix) {
        let mut commands = set_text;
        commands.push(Command::PopoverFilter {
            popover: FilterPopover::Context,
            query: query.clone(),
        });
        return changed(
            InteractionState {
                popover: Popover::Context {
                    query,
                    active_id: None,
                },
                focus: Focus::Editor,
                ..state
            },
            commands,
            false,
        );
    }
    if let Some(query) = command_query(value) {
        let mut commands = set_text;
        commands.push(Command::PopoverFilter {
            popover: FilterPopover::Command,
            query: query.clone(),
        });
        return changed(
            InteractionState {
                popover: Popover::CommandInline {
                    query,
                    active_id: None,
                },
                focus: Focus::Editor,
                ..state
            },
            commands,
            false,
        );
    }

    let popover = match &state.popover {
        Popover::CommandMenu { .. } => state.popover.clone(),
        _ => Popover::Closed,
    };
    changed(
        InteractionState {
            popover,
            focus: Focus::Editor,
            ..state
        },
        set_text,
        false,
    )
}

fn open_commands(state: InteractionState, persisted: &PersistedState) -> Transition {
    if !populated(persisted) {
        return changed(
            InteractionState {
                popover: Popover::CommandInline {
                    query: String::new(),
                    active_id: None,
                },
                focus: Focus::Editor,
                ..state
            },
            vec![
                Command::DraftSetText {
                    value: format!("{}/", prompt_text(persisted)),
                },
                Command::PopoverFilter {
                    popover: FilterPopover::Command,
                    query: String::new(),
                },
                Command::FocusEditor,
            ],
            false,
        );
    }
    changed(
        InteractionState {
            popover: Popover::CommandMenu {
                query: String::new(),
                active_id: None,
            },
            focus: Focus::CommandSearch,
            ..state
        },
        vec![
            Command::PopoverFilter {
                popover: FilterPopover::Command,
                query: String::new(),
            },
            Command::FocusCommandSearch,
        ],
        false,
    )
}

fn open_context(state: InteractionState, persisted: &PersistedState) -> Transition {
    changed(
        InteractionState {
            popover: Popover::Context {
                query: String::new(),
                active_id: None,
            },
            focus: Focus::Editor,
            ..state
        },
        vec![
            Command::DraftSetText {
                value: format!("{}@", prompt_text(persisted)),
            },
            Command::PopoverFilter {
                popover: FilterPopover::Context,
                query: String::new(),
            },
            Command::FocusEditor,
        ],
        false,
    )
}

fn query_changed(state: InteractionState, query: &str) -> Transition {
    if state.popover == Popover::Closed {
        return unchanged(state, false);
    }
    let popover = match &state.popover {
        Popover::Context { .. } => FilterPopover::Context,
        _ => FilterPopover::Command,
    };
    changed(
        InteractionState {
            popover: state.popover.with_query(query.to_string()),
            ..state
        },
        vec![Command::PopoverFilter {
            popover,
            query: query.to_string(),
        }],
        false,
    )
}

fn results_changed(state: InteractionState, ids: &[String]) -> Transition {
    if state.popover == Popover::Closed {
        return unchanged(state, false);
    }
    let active_id = match state.popover.active_id() {
        Some(current) if ids.iter().any(|id| id == current) => Some(current.to_string()),
        _ => ids.first().cloned(),
    };
    if active_id.as_deref() == state.popover.active_id() {
        return unchanged(state, false);
    }
    changed(
        InteractionState {
            popover: match active_id {
                Some(id) => state.popover.with_active_id(id),
                None => state.popover.clone(),
            },
            ..state
        },
        Vec::new(),
        false,
    )
}

fn active_changed(state: InteractionState, id: &str) -> Transition {
    if state.popover == Popover::Closed || state.popover.active_id() == Some(id) {
        return unchanged(state, false);
    }
    changed(
        InteractionState {
            popover: state.popover.with_active_id(id.to_string()),
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
) -> Transition {
    let current = prompt_text(persisted);
    let mut commands = Vec::new();
    if item.kind == SuggestionKind::Command {
        let value = if matches!(state.popover, Popover::CommandMenu { .. }) {
            if current.trim().is_empty() {
                format!("{} ", item.label)
            } else {
                format!("{} {}", item.label, current.trim())
            }
        } else {
            replace_trigger(&current, "/", &format!("{} ", item.label))
        };
        commands.push(Command::DraftSetText { value });
    } else {
        commands.push(Command::MentionAdd { item: item.clone() });
    }
    commands.push(Command::FocusEditor);
    changed(
        InteractionState {
            popover: Popover::Closed,
            focus: Focus::Editor,
            ..state
        },
        commands,
        false,
    )
}

fn key_down(
    state: InteractionState,
    key: &str,
    ctrl: bool,
    composing: bool,
    ids: &[String],
    empty: bool,
) -> Transition {
    if ctrl && key.eq_ignore_ascii_case("g") {
        if state.popover == Popover::Closed {
            return unchanged(state, false);
        }
        return changed(
            InteractionState {
                popover: Popover::Closed,
                focus: Focus::Editor,
                ..state
            },
            vec![Command::FocusEditor],
            true,
        );
    }
    if state.popover == Popover::Closed {
        if state.mode == Mode::Shell && (key == "Escape" || (key == "Backspace" && empty)) {
            return changed(
                InteractionState {
                    mode: Mode::Normal,
                    ..state
                },
                Vec::new(),
                true,
            );
        }
        return unchanged(state, false);
    }
    if key == "Escape" {
        return changed(
            InteractionState {
                popover: Popover::Closed,
                focus: Focus::Editor,
                ..state
            },
            vec![Command::FocusEditor],
            true,
        );
    }
    if key == "Tab" || (key == "Enter" && !composing) {
        let active = state.popover.active_id().map(str::to_string);
        return match active {
            Some(id) => {
                unchanged(state, true).with_commands(vec![Command::SuggestionSelect { id }])
            }
            None => unchanged(state, true),
        };
    }
    let direction = if key == "ArrowDown" || (ctrl && key == "n") {
        1i64
    } else if key == "ArrowUp" || (ctrl && key == "p") {
        -1i64
    } else {
        0
    };
    if direction == 0 || ids.is_empty() {
        return unchanged(state, false);
    }
    let current = state
        .popover
        .active_id()
        .and_then(|active| ids.iter().position(|id| id == active))
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
    changed(
        InteractionState {
            popover: state.popover.with_active_id(ids[index as usize].clone()),
            ..state
        },
        Vec::new(),
        true,
    )
}

fn prompt_text(persisted: &PersistedState) -> String {
    persisted.prompt.iter().map(PromptPart::content).collect()
}

fn populated(persisted: &PersistedState) -> bool {
    !prompt_text(persisted).trim().is_empty()
        || !persisted.context.items.is_empty()
        || persisted.prompt.iter().any(PromptPart::is_mention)
        || persisted.prompt.iter().any(PromptPart::is_image)
}

fn replace_trigger(value: &str, trigger: &str, replacement: &str) -> String {
    let index = value.find(trigger);
    match index {
        Some(index) => format!("{}{}", &value[..index], replacement),
        None => replacement.to_string(),
    }
}

fn context_query(prefix: &str) -> Option<String> {
    let index = prefix.rfind('@')?;
    let after = &prefix[index + 1..];
    if after.chars().any(|ch| ch.is_whitespace() || ch == '@') {
        return None;
    }
    if index > 0 {
        let previous = prefix[..index].chars().next_back()?;
        if !previous.is_whitespace() {
            return None;
        }
    }
    Some(after.to_string())
}

fn command_query(value: &str) -> Option<String> {
    let rest = value.strip_prefix('/')?;
    if rest.chars().any(char::is_whitespace) {
        return None;
    }
    Some(rest.to_string())
}

fn changed(state: InteractionState, commands: Vec<Command>, handled: bool) -> Transition {
    Transition {
        state,
        commands,
        handled,
    }
}

fn unchanged(state: InteractionState, handled: bool) -> Transition {
    Transition {
        state,
        commands: Vec::new(),
        handled,
    }
}

impl Transition {
    fn with_commands(mut self, commands: Vec<Command>) -> Transition {
        self.commands = commands;
        self
    }
}
