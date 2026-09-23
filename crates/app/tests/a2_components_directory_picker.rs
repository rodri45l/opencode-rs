//! Port of packages/app/src/components/directory-picker.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlatformKind {
    Desktop,
    Web,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ProjectKind {
    Sidecar,
    Ssh,
}

#[derive(Clone, Debug, PartialEq)]
struct Project {
    kind: ProjectKind,
    variant: Option<String>,
    host: Option<String>,
}

fn local() -> Project {
    Project {
        kind: ProjectKind::Sidecar,
        variant: Some("base".into()),
        host: None,
    }
}

fn remote() -> Project {
    Project {
        kind: ProjectKind::Ssh,
        variant: None,
        host: Some("example.test".into()),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PickerKind {
    Native,
    Server,
}

// Local stub (fast wave): real module lands later.
fn directory_picker_kind(_platform: PlatformKind, _project: &Project) -> Option<PickerKind> {
    None
}

#[test]
#[ignore = "porting: components/directory-picker not implemented"]
fn uses_the_native_picker_only_for_local_desktop_projects() {
    assert_eq!(
        directory_picker_kind(PlatformKind::Desktop, &local()),
        Some(PickerKind::Native)
    );
    assert_eq!(
        directory_picker_kind(PlatformKind::Desktop, &remote()),
        Some(PickerKind::Server)
    );
    assert_eq!(
        directory_picker_kind(PlatformKind::Web, &local()),
        Some(PickerKind::Server)
    );
}
