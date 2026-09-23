//! Port of packages/app/src/pages/session/session-panel-layout.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
struct Layout {
    visible: bool,
    stacked: bool,
}

// Local stub (fast wave): real module lands later.
fn session_panel_layout(_review: bool, _terminal: bool, _files: bool) -> Layout {
    Layout {
        visible: false,
        stacked: false,
    }
}

#[test]
#[ignore = "porting: pages/session/session-panel-layout not implemented"]
fn keeps_one_v2_owner_while_changing_panel_geometry() {
    assert_eq!(
        session_panel_layout(false, false, false),
        Layout {
            visible: false,
            stacked: false
        }
    );
    assert_eq!(
        session_panel_layout(false, true, false),
        Layout {
            visible: true,
            stacked: false
        }
    );
    assert_eq!(
        session_panel_layout(true, true, false),
        Layout {
            visible: true,
            stacked: true
        }
    );
}
