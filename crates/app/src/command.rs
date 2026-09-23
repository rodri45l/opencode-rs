//! Command palette + keybind helpers (port of packages/app/src/context/command.tsx).

const SUGGESTED_PREFIX: &str = "suggested.";

#[derive(Clone, Debug, PartialEq)]
pub struct CommandOption {
    pub id: String,
    pub title: String,
    pub hidden: bool,
    pub disabled: bool,
    pub when: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Registration {
    pub key: Option<String>,
    pub options: Vec<CommandOption>,
}

pub fn command_palette_options(options: &[CommandOption]) -> Vec<CommandOption> {
    options
        .iter()
        .filter(|option| {
            !option.disabled
                && !option.hidden
                && !option.id.starts_with(SUGGESTED_PREFIX)
                && option.id != "file.open"
        })
        .cloned()
        .collect()
}

pub fn add_command_registration(
    registrations: Vec<Registration>,
    next: Registration,
) -> Vec<Registration> {
    let mut result = Vec::with_capacity(registrations.len() + 1);
    result.push(next);
    result.extend(registrations);
    result
}

pub fn active_command_registrations(registrations: &[Registration]) -> Vec<Registration> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for entry in registrations {
        match &entry.key {
            None => result.push(entry.clone()),
            Some(key) => {
                if seen.insert(key.clone()) {
                    result.push(entry.clone());
                }
            }
        }
    }
    result
}

pub fn resolve_keybind_option(options: &[CommandOption]) -> Option<CommandOption> {
    options
        .iter()
        .find(|option| option.when == Some(true))
        .or_else(|| options.iter().find(|option| option.when.is_none()))
        .cloned()
}
