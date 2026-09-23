//! Port of packages/app/src/pages/session/session-panel-layout.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::session_panel_layout::{session_panel_layout, Layout};

#[test]
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
