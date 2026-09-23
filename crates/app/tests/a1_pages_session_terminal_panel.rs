//! Port of packages/app/src/pages/session/terminal-panel.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

fn t(key: &str, number: i64) -> String {
    match key {
        "terminal.title.numbered" => format!("Terminal {number}"),
        "terminal.title" => "Terminal".to_string(),
        _ => key.to_string(),
    }
}

// Local stub (fast wave): real module lands later.
fn terminal_tab_label(_title: &str, _title_number: i64) -> String {
    String::new()
}

#[test]
#[ignore = "porting: pages/session/terminal-panel not implemented"]
fn returns_custom_title_unchanged() {
    assert_eq!(
        terminal_tab_label("server", 3),
        t("terminal.title", 3).replace("Terminal", "server")
    );
}

#[test]
#[ignore = "porting: pages/session/terminal-panel not implemented"]
fn normalizes_default_numbered_title() {
    assert_eq!(
        terminal_tab_label("Terminal 2", 2),
        t("terminal.title.numbered", 2)
    );
}

#[test]
#[ignore = "porting: pages/session/terminal-panel not implemented"]
fn falls_back_to_generic_title() {
    assert_eq!(terminal_tab_label("", 0), t("terminal.title", 0));
}
