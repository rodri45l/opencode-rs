//! Prompt editor traits.
//!
//! Port of packages/tui/src/prompt/traits.ts `computePromptTraits` (upstream 18ef3cc).

/// The prompt mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptMode {
    Normal,
    Shell,
}

/// The capture keys a prompt claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capture {
    Tab,
    Escape,
    Navigate,
    Submit,
}

/// The traits computed for a prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptTraits {
    pub capture: Option<Vec<Capture>>,
    pub status: Option<String>,
}

/// Compute the capture keys and status label for a prompt.
pub fn compute_prompt_traits(mode: PromptMode, autocomplete_visible: bool) -> PromptTraits {
    let capture = match mode {
        PromptMode::Normal => {
            if autocomplete_visible {
                Some(vec![
                    Capture::Escape,
                    Capture::Navigate,
                    Capture::Submit,
                    Capture::Tab,
                ])
            } else {
                Some(vec![Capture::Tab])
            }
        }
        PromptMode::Shell => None,
    };
    PromptTraits {
        capture,
        status: if mode == PromptMode::Shell {
            Some("SHELL".to_string())
        } else {
            None
        },
    }
}
