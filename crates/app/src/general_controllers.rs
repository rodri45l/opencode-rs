//! General settings controllers (port of
//! packages/app/src/components/settings-v2/general-controller-behavior.ts).

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct ShellInfo {
    pub path: String,
    pub name: String,
    pub acceptable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShellOption {
    pub id: String,
    pub value: String,
    pub name: String,
    pub terminal_only: bool,
}

pub fn create_shell_options(shells: &[ShellInfo], current: &str) -> Vec<ShellOption> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for shell in shells {
        *counts.entry(shell.name.clone()).or_insert(0) += 1;
    }

    let mut options = vec![ShellOption {
        id: "auto".to_string(),
        value: String::new(),
        name: String::new(),
        terminal_only: false,
    }];
    for shell in shells {
        let ambiguous = counts.get(&shell.name).copied().unwrap_or(0) > 1;
        options.push(ShellOption {
            id: shell.path.clone(),
            value: if ambiguous {
                shell.path.clone()
            } else {
                shell.name.clone()
            },
            name: if ambiguous {
                shell.path.clone()
            } else {
                shell.name.clone()
            },
            terminal_only: !shell.acceptable,
        });
    }

    if !current.is_empty() && !options.iter().any(|option| option.value == current) {
        options.push(ShellOption {
            id: current.to_string(),
            value: current.to_string(),
            name: current.to_string(),
            terminal_only: false,
        });
    }
    options
}

#[derive(Default)]
pub struct SoundPreview {
    pub played: Vec<String>,
    pub stopped: Vec<String>,
    pending: Option<String>,
    current: Option<String>,
    elapsed: i64,
}

impl SoundPreview {
    fn stop(&mut self) {
        self.pending = None;
        self.elapsed = 0;
    }

    pub fn play(&mut self, id: &str) {
        self.stop();
        if id.is_empty() {
            return;
        }
        self.pending = Some(id.to_string());
    }

    pub fn advance(&mut self, ms: i64) {
        self.elapsed += ms;
        if let Some(pending) = self.pending.clone() {
            if self.elapsed >= 100 {
                self.pending = None;
                self.current = Some(pending.clone());
                self.played.push(pending);
            }
        }
    }

    pub fn dispose(&mut self) {
        if let Some(current) = self.current.take() {
            self.stopped.push(current);
        }
        self.pending = None;
    }
}
