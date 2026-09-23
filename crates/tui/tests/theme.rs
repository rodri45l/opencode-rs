//! Port of packages/tui/test/theme.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/theme.ts and context/theme.ts; see docs/TEST-PORT.md.

use opencode_tui::theme::{
    add_theme, all_themes, default_themes, discover_themes, has_theme, resolve_theme,
    terminal_mode, ThemeError,
};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique(prefix: &str) -> String {
    format!(
        "{prefix}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

#[test]
fn add_theme_writes_into_module_theme_store() {
    let name = unique("plugin-theme");
    assert!(add_theme(&name, default_themes()["opencode"].clone()));
    assert!(all_themes().contains_key(&name));
}

#[test]
fn add_theme_keeps_first_theme_for_duplicate_names() {
    let name = unique("plugin-theme-keep");
    let mut one = default_themes()["opencode"].clone();
    let mut two = default_themes()["opencode"].clone();
    one.theme
        .as_mut()
        .unwrap()
        .insert("primary".to_string(), "#101010".to_string());
    two.theme
        .as_mut()
        .unwrap()
        .insert("primary".to_string(), "#fefefe".to_string());

    assert!(add_theme(&name, one));
    assert!(!add_theme(&name, two));
    assert_eq!(
        all_themes()[&name].theme.as_ref().unwrap()["primary"],
        "#101010"
    );
}

#[test]
fn add_theme_ignores_entries_without_a_theme_object() {
    let name = unique("plugin-theme-invalid");
    let invalid = opencode_tui::theme::Theme {
        defs: [("a".to_string(), "#ffffff".to_string())]
            .into_iter()
            .collect(),
        theme: None,
    };
    assert!(!add_theme(&name, invalid));
    assert!(!all_themes().contains_key(&name));
}

#[test]
fn has_theme_checks_theme_presence() {
    let name = unique("plugin-theme-has");
    assert!(!has_theme(&name));
    assert!(add_theme(&name, default_themes()["opencode"].clone()));
    assert!(has_theme(&name));
}

#[test]
fn resolve_theme_rejects_circular_color_refs() {
    let mut item = default_themes()["opencode"].clone();
    item.defs.insert("one".to_string(), "two".to_string());
    item.defs.insert("two".to_string(), "one".to_string());
    item.theme
        .as_mut()
        .unwrap()
        .insert("primary".to_string(), "one".to_string());
    assert_eq!(resolve_theme(&item), Err(ThemeError::Circular));
}

#[test]
fn terminal_mode_derives_mode_from_refreshed_background() {
    assert_eq!(terminal_mode(Some("#fbf1c7")), Some("light"));
    assert_eq!(terminal_mode(Some("#1a1b26")), Some("dark"));
}

#[test]
fn terminal_mode_does_not_derive_mode_from_ansi_slot_zero() {
    assert_eq!(terminal_mode(None), None);
}

#[test]
fn custom_theme_precedence_follows_directory_order() {
    let root = std::env::temp_dir().join(unique("opencode-tui-theme"));
    let global = root.join("global");
    let project = root.join("project");
    std::fs::create_dir_all(global.join("themes")).unwrap();
    std::fs::create_dir_all(project.join("themes")).unwrap();
    std::fs::write(
        global.join("themes").join("custom.json"),
        r#"{"source":"global"}"#,
    )
    .unwrap();
    std::fs::write(
        project.join("themes").join("custom.json"),
        r#"{"source":"project"}"#,
    )
    .unwrap();

    let themes = discover_themes(&[
        global.to_string_lossy().to_string(),
        project.to_string_lossy().to_string(),
    ]);
    assert_eq!(themes["custom"], serde_json::json!({ "source": "project" }));

    let _ = std::fs::remove_dir_all(&root);
}
