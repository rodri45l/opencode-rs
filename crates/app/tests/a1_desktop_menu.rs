//! Port of packages/app/src/desktop-menu.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
enum MenuItem {
    Item {
        label_key: String,
        command: Option<String>,
        action: Option<String>,
        role: Option<String>,
    },
    Separator,
}

#[derive(Clone, Debug, PartialEq)]
struct Menu {
    role: Option<String>,
    label_key: Option<String>,
    items: Vec<MenuItem>,
}

// Local stub (fast wave): real module lands later.
fn desktop_menu() -> Vec<Menu> {
    Vec::new()
}

#[test]
#[ignore = "porting: desktop-menu not implemented"]
fn exports_logs_through_the_desktop_command_registry() {
    let items: Vec<MenuItem> = desktop_menu()
        .into_iter()
        .flat_map(|menu| menu.items)
        .filter(|item| {
            matches!(
                item,
                MenuItem::Item { label_key, .. } if label_key == "desktop.menu.exportLogs"
            )
        })
        .collect();
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|item| matches!(
        item,
        MenuItem::Item { command: Some(command), action: None, .. } if command == "logs.export"
    )));
}

#[test]
#[ignore = "porting: desktop-menu not implemented"]
fn provides_translated_labels_for_role_backed_entries() {
    let menus = desktop_menu();
    let window_menu = menus
        .iter()
        .find(|menu| menu.role.as_deref() == Some("windowMenu"));
    assert_eq!(
        window_menu.and_then(|menu| menu.label_key.clone()),
        Some("desktop.menu.window".to_string())
    );
    let role_items: Vec<&MenuItem> = menus
        .iter()
        .flat_map(|menu| menu.items.iter())
        .filter(|item| matches!(item, MenuItem::Item { role: Some(_), label_key, .. } if !label_key.is_empty()))
        .collect();
    assert!(!role_items.is_empty());
}
