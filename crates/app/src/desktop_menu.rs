//! Desktop application menu (port of packages/app/src/desktop-menu.ts).

#[derive(Clone, Debug, PartialEq)]
pub enum MenuItem {
    Item {
        label_key: String,
        command: Option<String>,
        action: Option<String>,
        role: Option<String>,
    },
    Separator,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Menu {
    pub id: String,
    pub role: Option<String>,
    pub label_key: String,
    pub items: Vec<MenuItem>,
}

fn item(
    label_key: &str,
    command: Option<&str>,
    action: Option<&str>,
    role: Option<&str>,
) -> MenuItem {
    MenuItem::Item {
        label_key: label_key.to_string(),
        command: command.map(|value| value.to_string()),
        action: action.map(|value| value.to_string()),
        role: role.map(|value| value.to_string()),
    }
}

fn role_item(role: &str) -> MenuItem {
    item("", None, None, Some(role))
}

pub fn desktop_menu() -> Vec<Menu> {
    vec![
        Menu {
            id: "app".into(),
            role: None,
            label_key: "desktop.menu.app".into(),
            items: vec![
                role_item("about"),
                item(
                    "desktop.menu.checkForUpdates",
                    None,
                    Some("app.checkForUpdates"),
                    None,
                ),
                item("desktop.menu.settings", Some("settings.open"), None, None),
                item(
                    "desktop.menu.reloadWebview",
                    None,
                    Some("view.reload"),
                    None,
                ),
                item("desktop.menu.restart", None, Some("app.relaunch"), None),
                item("desktop.menu.exportLogs", Some("logs.export"), None, None),
                MenuItem::Separator,
                role_item("hide"),
                role_item("hideOthers"),
                role_item("unhide"),
                MenuItem::Separator,
                role_item("quit"),
            ],
        },
        Menu {
            id: "file".into(),
            role: None,
            label_key: "desktop.menu.file".into(),
            items: vec![
                item("desktop.menu.newSession", Some("session.new"), None, None),
                item("desktop.menu.openProject", Some("project.open"), None, None),
                item("desktop.menu.settings", Some("settings.open"), None, None),
                item("desktop.menu.newWindow", None, Some("window.new"), None),
                MenuItem::Separator,
                item(
                    "desktop.menu.closeWindow",
                    None,
                    Some("window.close"),
                    Some("close"),
                ),
            ],
        },
        Menu {
            id: "edit".into(),
            role: None,
            label_key: "desktop.menu.edit".into(),
            items: vec![
                item("desktop.menu.undo", None, Some("edit.undo"), Some("undo")),
                item("desktop.menu.redo", None, Some("edit.redo"), Some("redo")),
                MenuItem::Separator,
                item("desktop.menu.cut", None, Some("edit.cut"), Some("cut")),
                item("desktop.menu.copy", None, Some("edit.copy"), Some("copy")),
                item(
                    "desktop.menu.paste",
                    None,
                    Some("edit.paste"),
                    Some("paste"),
                ),
                item("desktop.menu.delete", None, Some("edit.delete"), None),
                item(
                    "desktop.menu.selectAll",
                    None,
                    Some("edit.selectAll"),
                    Some("selectAll"),
                ),
            ],
        },
        Menu {
            id: "view".into(),
            role: None,
            label_key: "desktop.menu.view".into(),
            items: vec![
                item(
                    "desktop.menu.toggleSidebar",
                    Some("sidebar.toggle"),
                    None,
                    None,
                ),
                item(
                    "desktop.menu.toggleTerminal",
                    Some("terminal.toggle"),
                    None,
                    None,
                ),
                item(
                    "desktop.menu.toggleFileTree",
                    Some("fileTree.toggle"),
                    None,
                    None,
                ),
                MenuItem::Separator,
                item(
                    "desktop.menu.reload",
                    None,
                    Some("view.reload"),
                    Some("reload"),
                ),
                item(
                    "desktop.menu.toggleDeveloperTools",
                    None,
                    Some("view.toggleDevTools"),
                    Some("toggleDevTools"),
                ),
                MenuItem::Separator,
                item(
                    "desktop.menu.actualSize",
                    None,
                    Some("view.resetZoom"),
                    Some("resetZoom"),
                ),
                item(
                    "desktop.menu.zoomIn",
                    None,
                    Some("view.zoomIn"),
                    Some("zoomIn"),
                ),
                item(
                    "desktop.menu.zoomOut",
                    None,
                    Some("view.zoomOut"),
                    Some("zoomOut"),
                ),
                MenuItem::Separator,
                item(
                    "desktop.menu.toggleFullScreen",
                    None,
                    Some("view.toggleFullscreen"),
                    Some("togglefullscreen"),
                ),
            ],
        },
        Menu {
            id: "go".into(),
            role: None,
            label_key: "desktop.menu.go".into(),
            items: vec![
                item("desktop.menu.back", Some("common.goBack"), None, None),
                item("desktop.menu.forward", Some("common.goForward"), None, None),
                MenuItem::Separator,
                item(
                    "desktop.menu.previousSession",
                    Some("session.previous"),
                    None,
                    None,
                ),
                item("desktop.menu.nextSession", Some("session.next"), None, None),
                MenuItem::Separator,
                item(
                    "desktop.menu.previousProject",
                    Some("project.previous"),
                    None,
                    None,
                ),
                item("desktop.menu.nextProject", Some("project.next"), None, None),
            ],
        },
        Menu {
            id: "window".into(),
            role: Some("windowMenu".into()),
            label_key: "desktop.menu.window".into(),
            items: vec![
                item("desktop.menu.minimize", None, Some("window.minimize"), None),
                item(
                    "desktop.menu.maximize",
                    None,
                    Some("window.toggleMaximize"),
                    None,
                ),
                MenuItem::Separator,
                item("desktop.menu.closeWindow", None, Some("window.close"), None),
            ],
        },
        Menu {
            id: "help".into(),
            role: None,
            label_key: "desktop.menu.help".into(),
            items: vec![
                item("desktop.menu.documentation", None, None, None),
                item("desktop.menu.supportForum", None, None, None),
                item("desktop.menu.exportLogs", Some("logs.export"), None, None),
                MenuItem::Separator,
                item("desktop.menu.shareFeedback", None, None, None),
                item("desktop.menu.reportBug", None, None, None),
            ],
        },
    ]
}
