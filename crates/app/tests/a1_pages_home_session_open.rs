//! Port of packages/app/src/pages/home-session-open.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::home_session_open::{should_open_session_in_background, Click};

#[test]
fn opens_middle_clicks_in_the_background() {
    assert!(should_open_session_in_background(Click {
        button: 1,
        mac: true,
        meta: false,
        ctrl: false,
        shift: false,
        alt: false
    }));
    assert!(!should_open_session_in_background(Click {
        button: 2,
        mac: true,
        meta: false,
        ctrl: false,
        shift: false,
        alt: false
    }));
}

#[test]
fn requires_only_the_platform_primary_modifier() {
    assert!(should_open_session_in_background(Click {
        button: 0,
        mac: true,
        meta: true,
        ctrl: false,
        shift: false,
        alt: false
    }));
    assert!(should_open_session_in_background(Click {
        button: 0,
        mac: false,
        meta: false,
        ctrl: true,
        shift: false,
        alt: false
    }));
    assert!(!should_open_session_in_background(Click {
        button: 0,
        mac: true,
        meta: true,
        ctrl: false,
        shift: true,
        alt: false
    }));
    assert!(!should_open_session_in_background(Click {
        button: 0,
        mac: false,
        meta: false,
        ctrl: true,
        shift: false,
        alt: true
    }));
    assert!(!should_open_session_in_background(Click {
        button: 0,
        mac: false,
        meta: true,
        ctrl: false,
        shift: false,
        alt: false
    }));
}
