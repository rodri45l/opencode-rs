//! Port of packages/tui/test/config.test.tsx (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/config/index.tsx and config/keybind.ts;
//! see docs/TEST-PORT.md. The Solid context provider assertion is visual
//! (human-verified) and omitted here.
#![allow(dead_code)]

use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

const LEADER_TIMEOUT_DEFAULT: i64 = 2000;
const ATTENTION_SOUND_NAMES: [&str; 6] = [
    "default",
    "question",
    "permission",
    "error",
    "done",
    "subagent_done",
];

// --- keybind surface (local stub) -------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
struct Binding {
    key: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct KeybindMap {
    by_command: BTreeMap<String, Vec<Binding>>,
}

impl KeybindMap {
    fn get(&self, command: &str) -> Vec<Binding> {
        self.by_command.get(command).cloned().unwrap_or_default()
    }

    fn has(&self, command: &str) -> bool {
        self.by_command.contains_key(command)
    }
}

#[derive(Debug, Clone, Default)]
struct KeybindOverrides {
    entries: BTreeMap<String, String>,
}

/// (keybind name, command, default binding string)
const DEFINITIONS: &[(&str, &str, &str)] = &[
    ("leader", "leader", "ctrl+x"),
    ("terminal_suspend", "terminal.suspend", "ctrl+z"),
    ("session_list", "session.list", "<leader>l"),
    ("session_new", "session.new", "<leader>n"),
    ("session_move", "session.move", "none"),
    ("input_undo", "input.undo", "ctrl+-,super+z"),
];

fn definitions_map() -> BTreeMap<&'static str, &'static str> {
    DEFINITIONS
        .iter()
        .map(|(name, _, _)| (*name, *name))
        .collect()
}

fn parse(overrides: &KeybindOverrides) -> Result<KeybindMap, String> {
    let known = definitions_map();
    for name in overrides.entries.keys() {
        if !known.contains_key(name.as_str()) {
            return Err(format!("Unrecognized keybind: {name}"));
        }
    }
    let mut map = KeybindMap::default();
    for (name, command, default) in DEFINITIONS {
        let effective = overrides
            .entries
            .get(*name)
            .map(String::as_str)
            .unwrap_or(*default);
        if effective == "none" || effective == "false" {
            continue;
        }
        map.by_command.insert(
            (*command).to_string(),
            vec![Binding {
                key: effective.to_string(),
            }],
        );
    }
    Ok(map)
}

// --- config info decoding (local stub) --------------------------------------------

fn decode_plugin_spec(value: &Value) -> Result<Value, String> {
    match value {
        Value::String(name) => Ok(Value::String(name.clone())),
        Value::Array(items) if items.len() == 2 => {
            let name = items[0].as_str().ok_or("plugin name")?;
            let options = items[1].as_object().ok_or("plugin options")?;
            Ok(json!([name, Value::Object(options.clone())]))
        }
        _ => Err("invalid plugin spec".into()),
    }
}

fn positive_int(value: &Value) -> Result<i64, String> {
    let number = value.as_i64().ok_or("expected integer")?;
    if number <= 0 {
        return Err("expected positive integer".into());
    }
    Ok(number)
}

fn decode_info(input: &Value) -> Result<Value, String> {
    let obj = input.as_object().ok_or("expected object")?;
    let mut out = Map::new();

    if let Some(value) = obj.get("leader_timeout") {
        out.insert("leader_timeout".into(), json!(positive_int(value)?));
    }
    if let Some(value) = obj.get("theme") {
        out.insert("theme".into(), value.clone());
    }
    if let Some(value) = obj.get("mouse") {
        if !value.is_boolean() {
            return Err("mouse must be boolean".into());
        }
        out.insert("mouse".into(), value.clone());
    }

    if let Some(attention) = obj.get("attention").and_then(Value::as_object) {
        let mut normalized = Map::new();
        if let Some(enabled) = attention.get("enabled") {
            if !enabled.is_boolean() {
                return Err("attention.enabled".into());
            }
            normalized.insert("enabled".into(), enabled.clone());
        }
        if let Some(notifications) = attention.get("notifications") {
            if !notifications.is_boolean() {
                return Err("attention.notifications".into());
            }
            normalized.insert("notifications".into(), notifications.clone());
        }
        if let Some(sound) = attention.get("sound") {
            if !sound.is_boolean() {
                return Err("attention.sound".into());
            }
            normalized.insert("sound".into(), sound.clone());
        }
        if let Some(volume) = attention.get("volume") {
            let value = volume.as_f64().ok_or("attention.volume")?;
            if !(0.0..=1.0).contains(&value) {
                return Err("attention.volume out of range".into());
            }
            normalized.insert("volume".into(), volume.clone());
        }
        if let Some(pack) = attention.get("sound_pack") {
            normalized.insert("sound_pack".into(), pack.clone());
        }
        if let Some(sounds) = attention.get("sounds") {
            let sounds = sounds.as_object().ok_or("attention.sounds")?;
            let mut kept = Map::new();
            for name in ATTENTION_SOUND_NAMES {
                if let Some(path) = sounds.get(name) {
                    kept.insert(name.into(), path.clone());
                }
            }
            normalized.insert("sounds".into(), Value::Object(kept));
        }
        out.insert("attention".into(), Value::Object(normalized));
    }

    if let Some(prompt) = obj.get("prompt").and_then(Value::as_object) {
        let mut normalized = Map::new();
        if let Some(height) = prompt.get("max_height") {
            normalized.insert("max_height".into(), json!(positive_int(height)?));
        }
        if let Some(width) = prompt.get("max_width") {
            match width {
                Value::String(auto) if auto == "auto" => {
                    normalized.insert("max_width".into(), width.clone());
                }
                _ => {
                    normalized.insert("max_width".into(), json!(positive_int(width)?));
                }
            }
        }
        out.insert("prompt".into(), Value::Object(normalized));
    }

    if let Some(speed) = obj.get("scroll_speed") {
        let value = speed.as_f64().ok_or("scroll_speed")?;
        if value < 0.001 {
            return Err("scroll_speed out of range".into());
        }
        out.insert("scroll_speed".into(), speed.clone());
    }

    if let Some(style) = obj.get("diff_style") {
        let value = style.as_str().ok_or("diff_style")?;
        if !["auto", "stacked"].contains(&value) {
            return Err("diff_style invalid".into());
        }
        out.insert("diff_style".into(), style.clone());
    }

    if let Some(cursor) = obj.get("cursor").and_then(Value::as_object) {
        let mut normalized = Map::new();
        if let Some(style) = cursor.get("style") {
            let value = style.as_str().ok_or("cursor.style")?;
            if !["block", "underline", "line", "default"].contains(&value) {
                return Err("cursor.style invalid".into());
            }
            normalized.insert("style".into(), style.clone());
        }
        if let Some(blinking) = cursor.get("blinking") {
            if !blinking.is_boolean() {
                return Err("cursor.blinking".into());
            }
            normalized.insert("blinking".into(), blinking.clone());
        }
        out.insert("cursor".into(), Value::Object(normalized));
    }

    if let Some(plugin) = obj.get("plugin") {
        let items = plugin.as_array().ok_or("plugin must be array")?;
        let mut decoded = Vec::with_capacity(items.len());
        for item in items {
            decoded.push(decode_plugin_spec(item)?);
        }
        out.insert("plugin".into(), Value::Array(decoded));
    }

    Ok(Value::Object(out))
}

// --- resolved config (local stub) -------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
struct Attention {
    enabled: bool,
    notifications: bool,
    sound: bool,
    volume: f64,
    sound_pack: String,
    sounds: BTreeMap<String, String>,
}

impl Default for Attention {
    fn default() -> Self {
        Self {
            enabled: false,
            notifications: true,
            sound: true,
            volume: 0.4,
            sound_pack: "opencode.default".to_string(),
            sounds: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Cursor {
    style: String,
    blinking: bool,
}

#[derive(Debug, Clone)]
struct Resolved {
    theme: Option<String>,
    mouse: bool,
    leader_timeout: i64,
    attention: Attention,
    cursor: Option<Cursor>,
    keybinds: KeybindMap,
}

fn resolve(input: &Value, terminal_suspend: bool) -> Resolved {
    let obj = input.as_object().cloned().unwrap_or_default();

    let mut overrides = KeybindOverrides::default();
    if let Some(keybinds) = obj.get("keybinds").and_then(Value::as_object) {
        for (name, value) in keybinds {
            let text = match value {
                Value::String(text) => text.clone(),
                Value::Bool(false) => "none".to_string(),
                other => other.to_string(),
            };
            overrides.entries.insert(name.clone(), text);
        }
    }

    if !terminal_suspend {
        overrides
            .entries
            .insert("terminal_suspend".into(), "none".into());
        if !overrides.entries.contains_key("input_undo") {
            let default = "ctrl+-,super+z";
            let mut values = vec!["ctrl+z".to_string()];
            values.extend(default.split(',').map(str::to_string));
            let mut seen = Vec::new();
            for value in values {
                if !seen.contains(&value) {
                    seen.push(value);
                }
            }
            overrides
                .entries
                .insert("input_undo".into(), seen.join(","));
        }
    }

    let attention_input = obj.get("attention").and_then(Value::as_object);
    let attention = Attention {
        enabled: attention_input
            .and_then(|a| a.get("enabled"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        notifications: attention_input
            .and_then(|a| a.get("notifications"))
            .and_then(Value::as_bool)
            .unwrap_or(true),
        sound: attention_input
            .and_then(|a| a.get("sound"))
            .and_then(Value::as_bool)
            .unwrap_or(true),
        volume: attention_input
            .and_then(|a| a.get("volume"))
            .and_then(Value::as_f64)
            .unwrap_or(0.4),
        sound_pack: attention_input
            .and_then(|a| a.get("sound_pack"))
            .and_then(Value::as_str)
            .unwrap_or("opencode.default")
            .to_string(),
        sounds: attention_input
            .and_then(|a| a.get("sounds"))
            .and_then(Value::as_object)
            .map(|sounds| {
                sounds
                    .iter()
                    .map(|(key, value)| {
                        (key.clone(), value.as_str().unwrap_or_default().to_string())
                    })
                    .collect()
            })
            .unwrap_or_default(),
    };

    let cursor = obj
        .get("cursor")
        .and_then(Value::as_object)
        .map(|cursor| Cursor {
            style: cursor
                .get("style")
                .and_then(Value::as_str)
                .unwrap_or("block")
                .to_string(),
            blinking: cursor
                .get("blinking")
                .and_then(Value::as_bool)
                .unwrap_or(true),
        });

    Resolved {
        theme: obj.get("theme").and_then(Value::as_str).map(str::to_string),
        mouse: obj.get("mouse").and_then(Value::as_bool).unwrap_or(true),
        leader_timeout: obj
            .get("leader_timeout")
            .and_then(Value::as_i64)
            .unwrap_or(LEADER_TIMEOUT_DEFAULT),
        attention,
        cursor,
        keybinds: parse(&overrides).expect("resolved keybinds should parse"),
    }
}

// --- tests ------------------------------------------------------------------------

#[test]
#[ignore = "porting: tui config schema/resolution not implemented"]
fn defines_package_owned_plugin_specs_and_attention_sound_names() {
    assert_eq!(
        decode_plugin_spec(&json!("example-plugin")).unwrap(),
        json!("example-plugin")
    );
    assert_eq!(
        decode_plugin_spec(&json!(["example-plugin", { "enabled": true }])).unwrap(),
        json!(["example-plugin", { "enabled": true }])
    );
    assert!(decode_plugin_spec(&json!(["example-plugin"])).is_err());
    assert_eq!(
        ATTENTION_SOUND_NAMES,
        [
            "default",
            "question",
            "permission",
            "error",
            "done",
            "subagent_done"
        ]
    );
}

#[test]
#[ignore = "porting: tui config schema/resolution not implemented"]
fn validates_config_constraints() {
    let decoded = decode_info(&json!({
        "leader_timeout": 250,
        "attention": { "volume": 1, "sounds": { "done": "done.wav" } },
        "prompt": { "max_height": 10, "max_width": "auto" },
        "scroll_speed": 0.001,
        "diff_style": "stacked",
        "cursor": { "blinking": false },
        "plugin": ["example-plugin"],
    }))
    .unwrap();

    assert_eq!(decoded["leader_timeout"], json!(250));
    assert_eq!(decoded["attention"]["volume"], json!(1));
    assert_eq!(decoded["diff_style"], json!("stacked"));
    assert_eq!(decoded["cursor"]["blinking"], json!(false));

    assert!(decode_info(&json!({ "leader_timeout": 0 })).is_err());
    assert!(decode_info(&json!({ "attention": { "volume": 1.1 } })).is_err());
    assert!(decode_info(&json!({ "prompt": { "max_width": 0 } })).is_err());
    assert!(decode_info(&json!({ "scroll_speed": 0 })).is_err());
    assert!(decode_info(&json!({ "cursor": { "style": "beam" } })).is_err());
    assert_eq!(
        decode_info(&json!({ "attention": { "sounds": { "unknown": "sound.wav" } } })).unwrap(),
        json!({ "attention": { "sounds": {} } })
    );
}

#[test]
#[ignore = "porting: tui config schema/resolution not implemented"]
fn resolves_host_neutral_defaults() {
    let config = resolve(&json!({}), true);

    assert_eq!(
        config.attention,
        Attention {
            enabled: false,
            notifications: true,
            sound: true,
            volume: 0.4,
            sound_pack: "opencode.default".to_string(),
            sounds: BTreeMap::new(),
        }
    );
    assert_eq!(config.leader_timeout, LEADER_TIMEOUT_DEFAULT);
    assert!(config.mouse);
    assert!(config.keybinds.has("terminal.suspend"));
    assert!(config.keybinds.has("session.list"));
    assert_eq!(config.cursor, None);
}

#[test]
#[ignore = "porting: tui config schema/resolution not implemented"]
fn resolves_overrides_without_mutating_input() {
    let input = json!({
        "theme": "custom",
        "mouse": false,
        "leader_timeout": 750,
        "attention": {
            "enabled": true,
            "notifications": false,
            "sound": false,
            "volume": 0.8,
            "sound_pack": "custom.pack",
            "sounds": { "question": "/sounds/question.wav" },
        },
        "keybinds": { "session_list": "ctrl+l" },
        "cursor": { "blinking": false },
    });
    let before = input.clone();
    let config = resolve(&input, true);

    assert_eq!(config.theme.as_deref(), Some("custom"));
    assert!(!config.mouse);
    assert_eq!(config.leader_timeout, 750);
    assert_eq!(
        config.attention,
        Attention {
            enabled: true,
            notifications: false,
            sound: false,
            volume: 0.8,
            sound_pack: "custom.pack".to_string(),
            sounds: BTreeMap::from([("question".to_string(), "/sounds/question.wav".to_string())]),
        }
    );
    assert_eq!(
        config.cursor,
        Some(Cursor {
            style: "block".to_string(),
            blinking: false,
        })
    );
    assert_eq!(config.keybinds.get("session.list").len(), 1);
    assert_eq!(input, before);
}

#[test]
#[ignore = "porting: tui config schema/resolution not implemented"]
fn resolves_a_session_move_keybind() {
    let config = resolve(&json!({ "keybinds": { "session_move": "ctrl+o" } }), true);

    assert_eq!(
        config.keybinds.get("session.move"),
        vec![Binding {
            key: "ctrl+o".to_string()
        }]
    );
}

#[test]
#[ignore = "porting: tui config schema/resolution not implemented"]
fn disables_suspend_and_assigns_ctrl_z_to_undo_when_unsupported() {
    let config = resolve(&json!({}), false);

    assert!(!config.keybinds.has("terminal.suspend"));
    assert_eq!(
        config.keybinds.get("input.undo"),
        vec![Binding {
            key: "ctrl+z,ctrl+-,super+z".to_string()
        }]
    );
}

#[test]
#[ignore = "porting: tui config schema/resolution not implemented"]
fn preserves_an_explicit_undo_binding_when_suspend_is_unsupported() {
    let config = resolve(
        &json!({ "keybinds": { "input_undo": "ctrl+u", "terminal_suspend": "ctrl+s" } }),
        false,
    );

    assert!(!config.keybinds.has("terminal.suspend"));
    assert_eq!(config.keybinds.get("input.undo").len(), 1);
    assert_eq!(
        config.keybinds.get("input.undo"),
        vec![Binding {
            key: "ctrl+u".to_string()
        }]
    );
}
