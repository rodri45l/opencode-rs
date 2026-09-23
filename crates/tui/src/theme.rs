//! Theme store, resolution, and terminal mode detection.
//!
//! Port of packages/tui/src/theme.ts and context/theme.ts behaviour (upstream
//! 18ef3cc). Colour rendering is human-verified; token resolution is ported.

use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

/// A theme definition with named colour tokens.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Theme {
    pub defs: BTreeMap<String, String>,
    pub theme: Option<BTreeMap<String, String>>,
}

/// A theme resolution failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeError {
    Circular,
}

/// The built-in themes.
pub fn default_themes() -> BTreeMap<String, Theme> {
    let mut themes = BTreeMap::new();
    let mut defs = BTreeMap::new();
    defs.insert("background".to_string(), "#1a1b26".to_string());
    defs.insert("foreground".to_string(), "#c0caf5".to_string());
    let mut tokens = BTreeMap::new();
    tokens.insert("primary".to_string(), "#7aa2f7".to_string());
    tokens.insert("secondary".to_string(), "#bb9af7".to_string());
    themes.insert(
        "opencode".to_string(),
        Theme {
            defs,
            theme: Some(tokens),
        },
    );
    themes
}

fn store() -> &'static Mutex<BTreeMap<String, Theme>> {
    static STORE: OnceLock<Mutex<BTreeMap<String, Theme>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Register a theme, keeping the first definition for duplicate names.
pub fn add_theme(name: &str, theme: Theme) -> bool {
    if theme.theme.is_none() {
        return false;
    }
    let mut store = store().lock().unwrap();
    if store.contains_key(name) {
        return false;
    }
    store.insert(name.to_string(), theme);
    true
}

/// All registered themes.
pub fn all_themes() -> BTreeMap<String, Theme> {
    store().lock().unwrap().clone()
}

/// Whether a theme is registered.
pub fn has_theme(name: &str) -> bool {
    store().lock().unwrap().contains_key(name)
}

/// Resolve a theme's colour references, rejecting cycles.
pub fn resolve_theme(theme: &Theme) -> Result<BTreeMap<String, String>, ThemeError> {
    let mut resolved = BTreeMap::new();
    let tokens = theme.theme.clone().unwrap_or_default();
    for (key, value) in tokens {
        let mut seen = HashSet::new();
        let mut current = value;
        while let Some(next) = theme.defs.get(&current) {
            if !seen.insert(current.clone()) {
                return Err(ThemeError::Circular);
            }
            current = next.clone();
        }
        resolved.insert(key, current);
    }
    Ok(resolved)
}

/// Derive light/dark mode from a refreshed terminal background colour.
pub fn terminal_mode(default_background: Option<&str>) -> Option<&'static str> {
    let background = default_background?;
    let luminance = hex_luminance(background)?;
    Some(if luminance > 0.5 { "light" } else { "dark" })
}

fn hex_luminance(value: &str) -> Option<f64> {
    let hex = value.strip_prefix('#')?;
    if hex.len() < 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64;
    Some((0.299 * r + 0.587 * g + 0.114 * b) / 255.0)
}

/// Discover custom themes from directories, later directories taking priority.
pub fn discover_themes(directories: &[String]) -> BTreeMap<String, Value> {
    let mut themes = BTreeMap::new();
    for directory in directories {
        let theme_dir = Path::new(directory).join("themes");
        let Ok(entries) = std::fs::read_dir(&theme_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            let Ok(contents) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<Value>(&contents) else {
                continue;
            };
            themes.insert(name.to_string(), value);
        }
    }
    themes
}
