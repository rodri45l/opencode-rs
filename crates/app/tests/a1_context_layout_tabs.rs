//! Port of packages/app/src/context/layout-tabs.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

const SESSION_OPEN_FILE_TAB: &str = "open-file";

#[derive(Clone, Debug, PartialEq)]
struct SessionTabState {
    all: Vec<String>,
    active: Option<String>,
    preview: Option<String>,
}

// Local stubs (fast wave): real module lands later.
fn preview_session_tab(_state: SessionTabState, _tab: &str) -> SessionTabState {
    SessionTabState {
        all: Vec::new(),
        active: None,
        preview: None,
    }
}

fn open_session_tab(_state: SessionTabState, _tab: &str) -> SessionTabState {
    SessionTabState {
        all: Vec::new(),
        active: None,
        preview: None,
    }
}

fn close_session_tab(_state: SessionTabState, _tab: &str) -> SessionTabState {
    SessionTabState {
        all: Vec::new(),
        active: None,
        preview: None,
    }
}

fn state(all: &[&str], active: Option<&str>, preview: Option<&str>) -> SessionTabState {
    SessionTabState {
        all: all.iter().map(|s| (*s).to_string()).collect(),
        active: active.map(str::to_string),
        preview: preview.map(str::to_string),
    }
}

fn tab_state(all: &[&str], active: Option<&str>, preview: Option<&str>) -> SessionTabState {
    state(all, active, preview)
}

#[test]
#[ignore = "porting: context/layout-tabs not implemented"]
fn appends_the_open_file_placeholder() {
    assert_eq!(
        preview_session_tab(
            tab_state(&["file://a.ts"], Some("file://a.ts"), None),
            SESSION_OPEN_FILE_TAB
        ),
        state(
            &["file://a.ts", SESSION_OPEN_FILE_TAB],
            Some(SESSION_OPEN_FILE_TAB),
            Some(SESSION_OPEN_FILE_TAB)
        )
    );
}

#[test]
#[ignore = "porting: context/layout-tabs not implemented"]
fn replaces_the_current_preview_in_place() {
    assert_eq!(
        preview_session_tab(
            tab_state(
                &["context", SESSION_OPEN_FILE_TAB, "file://b.ts"],
                Some(SESSION_OPEN_FILE_TAB),
                Some(SESSION_OPEN_FILE_TAB)
            ),
            "file://a.ts"
        ),
        state(
            &["context", "file://a.ts", "file://b.ts"],
            Some("file://a.ts"),
            Some("file://a.ts")
        )
    );
}

#[test]
#[ignore = "porting: context/layout-tabs not implemented"]
fn activates_a_durable_tab_without_duplicating_it() {
    assert_eq!(
        preview_session_tab(
            tab_state(
                &["file://a.ts", SESSION_OPEN_FILE_TAB, "file://b.ts"],
                Some(SESSION_OPEN_FILE_TAB),
                Some(SESSION_OPEN_FILE_TAB)
            ),
            "file://b.ts"
        ),
        state(&["file://a.ts", "file://b.ts"], Some("file://b.ts"), None)
    );
}

#[test]
#[ignore = "porting: context/layout-tabs not implemented"]
fn replaces_a_restored_open_file_placeholder() {
    assert_eq!(
        preview_session_tab(
            tab_state(
                &["file://a.ts", SESSION_OPEN_FILE_TAB],
                Some(SESSION_OPEN_FILE_TAB),
                None
            ),
            "file://b.ts"
        ),
        state(
            &["file://a.ts", "file://b.ts"],
            Some("file://b.ts"),
            Some("file://b.ts")
        )
    );
}

#[test]
#[ignore = "porting: context/layout-tabs not implemented"]
fn pins_the_current_preview() {
    assert_eq!(
        open_session_tab(
            tab_state(&["file://a.ts"], Some("file://a.ts"), Some("file://a.ts")),
            "file://a.ts"
        ),
        state(&["file://a.ts"], Some("file://a.ts"), None)
    );
}

#[test]
#[ignore = "porting: context/layout-tabs not implemented"]
fn replaces_a_preview_with_a_directly_opened_file() {
    assert_eq!(
        open_session_tab(
            tab_state(&["file://a.ts"], Some("file://a.ts"), Some("file://a.ts")),
            "file://b.ts"
        ),
        state(&["file://b.ts"], Some("file://b.ts"), None)
    );
}

#[test]
#[ignore = "porting: context/layout-tabs not implemented"]
fn keeps_the_preview_when_switching_to_review() {
    assert_eq!(
        open_session_tab(
            tab_state(&["file://a.ts"], Some("file://a.ts"), Some("file://a.ts")),
            "review"
        ),
        state(&["file://a.ts"], Some("review"), Some("file://a.ts"))
    );
}

#[test]
#[ignore = "porting: context/layout-tabs not implemented"]
fn replaces_a_restored_open_file_placeholder_with_a_direct_open() {
    assert_eq!(
        open_session_tab(
            tab_state(
                &["file://a.ts", SESSION_OPEN_FILE_TAB],
                Some(SESSION_OPEN_FILE_TAB),
                None
            ),
            "file://b.ts"
        ),
        state(&["file://a.ts", "file://b.ts"], Some("file://b.ts"), None)
    );
}

#[test]
#[ignore = "porting: context/layout-tabs not implemented"]
fn clears_preview_metadata_and_selects_the_left_neighbor() {
    assert_eq!(
        close_session_tab(
            tab_state(
                &["file://a.ts", "file://b.ts", "file://c.ts"],
                Some("file://b.ts"),
                Some("file://b.ts")
            ),
            "file://b.ts"
        ),
        state(&["file://a.ts", "file://c.ts"], Some("file://a.ts"), None)
    );
}
