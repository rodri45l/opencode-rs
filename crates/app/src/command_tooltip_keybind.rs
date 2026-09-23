//! Command tooltip keybind lookup
//! (port of packages/app/src/components/command-tooltip-keybind.ts).

pub struct Command {
    pub parts: Vec<String>,
}

pub fn review_tooltip_keybind(command: &Command) -> Vec<String> {
    command.parts.clone()
}

pub fn new_tab_tooltip_keybind(command: &Command) -> Vec<String> {
    command.parts.clone()
}
