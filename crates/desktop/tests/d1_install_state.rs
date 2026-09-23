//! Port of packages/desktop/src/main/install-state.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/install-state.ts: benign files
//! Electron may create on a fresh install (`Local State`, `Crashpad/`) do not
//! count as prior OpenCode state, while OpenCode-owned files/directories do.

use opencode_desktop::install_state::{directory, file, has_existing_app_state};

#[test]
fn ignores_files_electron_may_create_on_a_fresh_install() {
    assert!(!has_existing_app_state(&[]));
    assert!(!has_existing_app_state(&[
        file("Local State"),
        directory("Crashpad")
    ]));
}

#[test]
fn recognizes_state_written_by_an_earlier_opencode_launch() {
    assert!(has_existing_app_state(&[file("opencode.settings")]));
    assert!(has_existing_app_state(&[file("opencode.global.dat")]));
    assert!(has_existing_app_state(&[file("window-state-abc.json")]));
    assert!(has_existing_app_state(&[directory("opencode")]));
}
