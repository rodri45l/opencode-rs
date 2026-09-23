//! Port of packages/tui/test/keymap.test.tsx (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/keymap.tsx and config/keybind.ts; see
//! docs/TEST-PORT.md. The OpenTUI render harness is visual (human-verified).
#![allow(dead_code)]

use std::collections::BTreeMap;

const KEY_ALIASES: &[(&str, &str)] = &[
    ("enter", "return"),
    ("esc", "escape"),
    ("pgdown", "pagedown"),
    ("pgup", "pageup"),
];

fn is_boundary(c: char) -> bool {
    matches!(c, '+' | ',' | ' ' | '\t' | '>' | '<')
}

fn expand_key_aliases(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let mut matched = false;
        for (alias, key) in KEY_ALIASES {
            let alias_chars: Vec<char> = alias.chars().collect();
            let end = i + alias_chars.len();
            if end <= chars.len() && chars[i..end] == alias_chars[..] {
                let before_ok = i == 0 || is_boundary(chars[i - 1]);
                let after_ok = end == chars.len() || is_boundary(chars[end]);
                if before_ok && after_ok {
                    out.push_str(key);
                    i = end;
                    matched = true;
                    break;
                }
            }
        }
        if !matched {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// Splits one effective binding string into sequences of stroke names.
fn binding_sequences(effective: &str) -> Vec<Vec<String>> {
    let expanded = expand_key_aliases(effective);
    expanded
        .split(',')
        .filter(|part| !part.is_empty())
        .map(|part| part.split('+').map(str::to_string).collect())
        .collect()
}

/// Default binding strings for the commands exercised by the reference test.
const DEFAULT_BINDINGS: &[(&str, &str)] = &[
    ("session.list", "<leader>l"),
    ("session.new", "<leader>n"),
    ("session.page.up", "pageup,ctrl+alt+b"),
    ("session.first", "ctrl+g,home"),
    ("model.list", "<leader>m"),
];

fn gather_counts(commands: &[&str]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for command in commands {
        let default = DEFAULT_BINDINGS
            .iter()
            .find(|(name, _)| name == command)
            .map(|(_, value)| *value)
            .unwrap_or_default();
        let count = binding_sequences(default).len();
        counts.insert((*command).to_string(), count);
    }
    counts
}

const GLOBAL_COMMANDS: &[&str] = &[
    "session.list",
    "session.new",
    "session.page.up",
    "session.first",
];
const BASE_COMMANDS: &[&str] = &["model.list"];

const OPENCODE_BASE_MODE: &str = "base";

#[derive(Debug, Clone)]
struct Keymap {
    global: BTreeMap<String, usize>,
    base: BTreeMap<String, usize>,
    stack: Vec<String>,
}

impl Keymap {
    fn new(global: BTreeMap<String, usize>, base: BTreeMap<String, usize>) -> Self {
        Self {
            global,
            base,
            stack: Vec::new(),
        }
    }

    fn current_mode(&self) -> &str {
        self.stack
            .last()
            .map(String::as_str)
            .unwrap_or(OPENCODE_BASE_MODE)
    }

    fn active_counts(&self, commands: &[&str]) -> BTreeMap<String, usize> {
        let mut out = BTreeMap::new();
        for command in commands {
            let mut count = *self.global.get(*command).unwrap_or(&0);
            if self.current_mode() == OPENCODE_BASE_MODE {
                count += *self.base.get(*command).unwrap_or(&0);
            }
            out.insert((*command).to_string(), count);
        }
        out
    }

    fn push(&mut self, mode: &str) -> usize {
        self.stack.push(mode.to_string());
        self.stack.len() - 1
    }

    fn truncate(&mut self, mark: usize) {
        self.stack.truncate(mark);
    }
}

#[test]
fn legacy_page_key_aliases_compile_as_page_keys() {
    assert_eq!(binding_sequences("pgup"), vec![vec!["pageup".to_string()]]);
    assert_eq!(
        binding_sequences("pgdown"),
        vec![vec!["pagedown".to_string()]]
    );
    assert_eq!(
        binding_sequences("ctrl+pgup"),
        vec![vec!["ctrl".to_string(), "pageup".to_string()]]
    );
}

#[test]
fn mode_less_bindings_stay_active_when_opencode_mode_changes() {
    let mut keymap = Keymap::new(gather_counts(GLOBAL_COMMANDS), gather_counts(BASE_COMMANDS));

    let mut commands: Vec<&str> = GLOBAL_COMMANDS.to_vec();
    commands.extend_from_slice(BASE_COMMANDS);

    let base = keymap.active_counts(&commands);
    assert_eq!(base["session.list"], 1);
    assert_eq!(base["session.new"], 1);
    assert_eq!(base["session.page.up"], 2);
    assert_eq!(base["session.first"], 2);
    assert_eq!(base["model.list"], 1);

    let mark = keymap.push("question");
    let question = keymap.active_counts(&commands);
    assert_eq!(question["session.list"], 1);
    assert_eq!(question["session.new"], 1);
    assert_eq!(question["session.page.up"], 2);
    assert_eq!(question["session.first"], 2);
    assert_eq!(question["model.list"], 0);
    keymap.truncate(mark);

    let mark = keymap.push("autocomplete");
    let autocomplete = keymap.active_counts(&commands);
    assert_eq!(autocomplete["session.list"], 1);
    assert_eq!(autocomplete["session.new"], 1);
    assert_eq!(autocomplete["session.page.up"], 2);
    assert_eq!(autocomplete["session.first"], 2);
    assert_eq!(autocomplete["model.list"], 0);
    keymap.truncate(mark);
}
