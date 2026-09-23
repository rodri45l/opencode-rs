//! Port of packages/core/test/global.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the global scratch path is `<system-temp>/opencode`, the
//! bundle agrees with it, and the directory exists once the module is loaded.

use opencode_core::global::Global;

#[test]
fn tmp_path_is_under_the_system_temp_directory() {
    let expected = std::env::temp_dir().join("opencode");
    assert_eq!(Global::tmp().unwrap(), expected);
    assert_eq!(Global::make().unwrap().tmp, expected);
}

#[test]
fn tmp_path_is_created_on_module_load() {
    assert!(Global::tmp().unwrap().is_dir());
}
