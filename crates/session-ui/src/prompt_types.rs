//! Shared prompt-input v2 data types.
//!
//! Port of packages/session-ui/src/v2/components/prompt-input/types.ts
//! behaviour (upstream 18ef3cc), reduced to the host-neutral shapes used by the
//! interaction machine and store.

/// A structured prompt part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptPart {
    Text {
        content: String,
        start: usize,
        end: usize,
    },
    File {
        path: String,
        content: String,
        start: usize,
        end: usize,
    },
    Agent {
        name: String,
        content: String,
        start: usize,
        end: usize,
    },
    Image {
        id: String,
        filename: String,
        mime: String,
        blob_id: String,
        blob_url: String,
    },
}

impl PromptPart {
    /// The part's textual content, if it has any.
    pub fn content(&self) -> &str {
        match self {
            PromptPart::Text { content, .. }
            | PromptPart::File { content, .. }
            | PromptPart::Agent { content, .. } => content,
            PromptPart::Image { .. } => "",
        }
    }

    /// Whether this part participates in the text offset space.
    pub fn is_image(&self) -> bool {
        matches!(self, PromptPart::Image { .. })
    }

    /// Whether this part is a file or agent mention.
    pub fn is_mention(&self) -> bool {
        matches!(self, PromptPart::File { .. } | PromptPart::Agent { .. })
    }
}

/// A model selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptModel {
    pub provider_id: String,
    pub model_id: String,
    pub variant: Option<String>,
}

/// A context (review comment) item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptComment {
    pub key: String,
    pub path: String,
}

/// The persisted prompt-input state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedState {
    pub prompt: Vec<PromptPart>,
    pub cursor: Option<usize>,
    pub model: Option<PromptModel>,
    pub context: ContextItems,
}

/// The context list carried by the persisted state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextItems {
    pub items: Vec<PromptComment>,
}

impl Default for PersistedState {
    fn default() -> Self {
        PersistedState {
            prompt: vec![PromptPart::Text {
                content: String::new(),
                start: 0,
                end: 0,
            }],
            cursor: Some(0),
            model: None,
            context: ContextItems::default(),
        }
    }
}

/// The kind of a completion suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionKind {
    Agent,
    Command,
    File,
    Reference,
    Resource,
}

/// A completion suggestion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    pub id: String,
    pub kind: SuggestionKind,
    pub label: String,
    pub path: Option<String>,
}
