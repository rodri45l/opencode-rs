//! Port of packages/tui/test/runtime.test.tsx (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/runtime.tsx; see docs/TEST-PORT.md.
//! The Solid runtime provider test is visual (n/a, human-verified).

use opencode_tui::runtime::abbreviate_home;

#[test]
fn abbreviates_paths_within_home_boundaries() {
    assert_eq!(abbreviate_home("/home/test", "/home/test"), "~");
    assert_eq!(
        abbreviate_home("/home/test/project", "/home/test"),
        "~/project"
    );
    assert_eq!(
        abbreviate_home("/home/tester/project", "/home/test"),
        "/home/tester/project"
    );
    assert_eq!(
        abbreviate_home("/tmp/project", "/home/test"),
        "/tmp/project"
    );
}
