//! Port of packages/tui/test/util/filetype.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/util/filetype.ts; see docs/TEST-PORT.md.

use opencode_tui::filetype::filetype;

#[test]
fn maps_filenames_to_presentation_languages() {
    assert_eq!(filetype(Some("component.tsx")), Some("typescript"));
    assert_eq!(filetype(Some("script.js")), Some("typescript"));
    assert_eq!(filetype(Some("main.py")), Some("python"));
    assert_eq!(filetype(Some("README.unknown")), None);
}

#[test]
fn uses_none_for_missing_filenames() {
    assert_eq!(filetype(None), Some("none"));
    assert_eq!(filetype(Some("")), Some("none"));
}
