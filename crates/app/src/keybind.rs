//! Keybind parsing/matching/formatting
//! (port of packages/app/src/context/command.tsx).

const IS_MAC: bool = cfg!(target_os = "macos");

#[derive(Clone, Debug, PartialEq)]
pub struct Keybind {
    pub key: String,
    pub ctrl: bool,
    pub meta: bool,
    pub shift: bool,
    pub alt: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyEvent {
    pub key: String,
    pub ctrl: bool,
    pub meta: bool,
    pub shift: bool,
    pub alt: bool,
}

fn normalize_key(key: &str) -> String {
    match key {
        "," => "comma".to_string(),
        "+" => "plus".to_string(),
        " " => "space".to_string(),
        other => other.to_lowercase(),
    }
}

pub fn parse_keybind(config: &str) -> Vec<Keybind> {
    if config.is_empty() || config == "none" {
        return Vec::new();
    }
    config
        .split(',')
        .map(|combo| {
            let mut keybind = Keybind {
                key: String::new(),
                ctrl: false,
                meta: false,
                shift: false,
                alt: false,
            };
            for part in combo.trim().to_lowercase().split('+') {
                match part {
                    "ctrl" | "control" => keybind.ctrl = true,
                    "meta" | "cmd" | "command" => keybind.meta = true,
                    "mod" => {
                        if IS_MAC {
                            keybind.meta = true;
                        } else {
                            keybind.ctrl = true;
                        }
                    }
                    "alt" | "option" => keybind.alt = true,
                    "shift" => keybind.shift = true,
                    other => keybind.key = other.to_string(),
                }
            }
            keybind
        })
        .collect()
}

pub fn match_keybind(keybinds: &[Keybind], event: &KeyEvent) -> bool {
    let event_key = normalize_key(&event.key);
    keybinds.iter().any(|keybind| {
        keybind.key == event_key
            && keybind.ctrl == event.ctrl
            && keybind.meta == event.meta
            && keybind.shift == event.shift
            && keybind.alt == event.alt
    })
}

fn display_keybind_parts(keybind: &Keybind) -> Vec<String> {
    let mut parts = Vec::new();
    if keybind.ctrl {
        parts.push(if IS_MAC {
            "⌃".to_string()
        } else {
            "Ctrl".to_string()
        });
    }
    if keybind.alt {
        parts.push(if IS_MAC {
            "⌥".to_string()
        } else {
            "Alt".to_string()
        });
    }
    if keybind.shift {
        parts.push(if IS_MAC {
            "⇧".to_string()
        } else {
            "Shift".to_string()
        });
    }
    if keybind.meta {
        parts.push(if IS_MAC {
            "⌘".to_string()
        } else {
            "Meta".to_string()
        });
    }
    if keybind.key.is_empty() {
        return parts;
    }

    let named = |key: &str| -> Option<String> {
        Some(
            match key {
                "arrowup" => "↑",
                "arrowdown" => "↓",
                "arrowleft" => "←",
                "arrowright" => "→",
                "comma" => ",",
                "plus" => "+",
                "space" => "Space",
                "backspace" => "Backspace",
                "delete" => "Delete",
                "end" => "End",
                "enter" => "Enter",
                "esc" | "escape" => "Esc",
                "home" => "Home",
                "insert" => "Insert",
                "pagedown" => "PageDown",
                "pageup" => "PageUp",
                "tab" => "Tab",
                _ => return None,
            }
            .to_string(),
        )
    };
    let key = keybind.key.to_lowercase();
    let display = named(&key).unwrap_or_else(|| {
        if key.chars().count() == 1 {
            key.to_uppercase()
        } else {
            let mut chars = key.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => key.clone(),
            }
        }
    });
    parts.push(display);
    parts
}

pub fn format_keybind(config: &str) -> String {
    if config.is_empty() || config == "none" {
        return String::new();
    }
    let keybind = match parse_keybind(config).into_iter().next() {
        Some(keybind) => keybind,
        None => return String::new(),
    };
    let parts = display_keybind_parts(&keybind);
    if parts.is_empty() {
        return String::new();
    }
    if IS_MAC {
        parts.join("")
    } else {
        parts.join("+")
    }
}
