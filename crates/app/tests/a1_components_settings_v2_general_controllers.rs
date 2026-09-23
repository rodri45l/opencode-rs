//! Port of packages/app/src/components/settings-v2/general-controllers.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct ShellInfo {
    path: String,
    name: String,
    acceptable: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct ShellOption {
    id: String,
    value: String,
    name: String,
    terminal_only: bool,
}

// Local stub (fast wave): real module lands later.
fn create_shell_options(_shells: &[ShellInfo], _current: &str) -> Vec<ShellOption> {
    Vec::new()
}

#[derive(Default)]
struct SoundPreview {
    played: Vec<String>,
    stopped: Vec<String>,
    pending: Option<String>,
}

impl SoundPreview {
    // Local stubs (fast wave): real module lands later.
    fn play(&mut self, _id: &str) {}
    fn advance(&mut self, _ms: i64) {}
    fn dispose(&mut self) {}
}

#[test]
#[ignore = "porting: components/settings-v2/general-controllers not implemented"]
fn normalizes_shell_names_and_preserves_an_unavailable_configured_shell() {
    let shells = vec![
        ShellInfo {
            path: "/bin/bash".into(),
            name: "bash".into(),
            acceptable: true,
        },
        ShellInfo {
            path: "/opt/bash".into(),
            name: "bash".into(),
            acceptable: false,
        },
        ShellInfo {
            path: "/bin/zsh".into(),
            name: "zsh".into(),
            acceptable: true,
        },
    ];
    assert_eq!(
        create_shell_options(&shells, "fish"),
        vec![
            ShellOption {
                id: "auto".into(),
                value: String::new(),
                name: String::new(),
                terminal_only: false
            },
            ShellOption {
                id: "/bin/bash".into(),
                value: "/bin/bash".into(),
                name: "/bin/bash".into(),
                terminal_only: false
            },
            ShellOption {
                id: "/opt/bash".into(),
                value: "/opt/bash".into(),
                name: "/opt/bash".into(),
                terminal_only: true
            },
            ShellOption {
                id: "/bin/zsh".into(),
                value: "zsh".into(),
                name: "zsh".into(),
                terminal_only: false
            },
            ShellOption {
                id: "fish".into(),
                value: "fish".into(),
                name: "fish".into(),
                terminal_only: false
            },
        ]
    );
}

#[test]
#[ignore = "porting: components/settings-v2/general-controllers not implemented"]
fn debounces_previews_and_stops_owned_audio_on_disposal() {
    let mut preview = SoundPreview::default();
    preview.play("first");
    preview.advance(99);
    assert!(preview.played.is_empty());

    preview.play("second");
    preview.advance(100);
    assert_eq!(preview.played, vec!["second".to_string()]);

    preview.dispose();
    assert_eq!(preview.stopped, vec!["second".to_string()]);
}
