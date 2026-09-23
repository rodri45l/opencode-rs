//! Port of packages/core/test/global.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the global scratch path is `<system-temp>/opencode`, the
//! bundle agrees with it, and the directory exists once the module is loaded.

use opencode_core::global::Global;

const NOTE: &str = "porting: global paths not implemented";

#[test]
#[ignore = "porting: global paths not implemented"]
fn tmp_path_is_under_the_system_temp_directory() {
    let expected = std::env::temp_dir().join("opencode");
    assert_eq!(Global::tmp().expect(NOTE), expected);
    assert_eq!(Global::make().expect(NOTE).tmp, expected);
}

#[test]
#[ignore = "porting: global paths not implemented"]
fn tmp_path_is_created_on_module_load() {
    assert!(Global::tmp().expect(NOTE).is_dir());
}
