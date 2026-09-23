//! Prompt input placeholder selection
//! (port of packages/app/src/components/prompt-input/placeholder.ts).

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Shell,
    Normal,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlaceholderInput {
    pub mode: Mode,
    pub comment_count: usize,
    pub example: String,
    pub suggest: bool,
}

pub fn prompt_placeholder(input: PlaceholderInput) -> String {
    if input.mode == Mode::Shell {
        return format!("prompt.placeholder.shell:{}", input.example);
    }
    if input.comment_count > 1 {
        return format!("prompt.placeholder.summarizeComments:{}", input.example);
    }
    if input.comment_count == 1 {
        return format!("prompt.placeholder.summarizeComment:{}", input.example);
    }
    if !input.suggest {
        return "prompt.placeholder.simple".to_string();
    }
    format!("prompt.placeholder.normal:{}", input.example)
}

pub fn prompt_design_placeholder(mode: Mode, fallback: &str) -> String {
    if mode == Mode::Shell {
        return fallback.to_string();
    }
    "Ask anything, / for commands, @ for context...".to_string()
}
