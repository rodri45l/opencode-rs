//! Port of packages/desktop/src/main/install-state.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/install-state.ts: benign files
//! Electron may create on a fresh install (`Local State`, `Crashpad/`) do not
//! count as prior OpenCode state, while OpenCode-owned files/directories do.

#[allow(dead_code)]
mod install_state {
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: desktop install-state policy not implemented";

    fn stub<T>() -> PortResult<T> {
        Err(NotImplemented(NOTE))
    }

    #[derive(Debug, Clone)]
    pub struct Entry {
        pub name: String,
        pub is_directory: bool,
    }

    pub fn file(name: &str) -> Entry {
        Entry {
            name: name.to_string(),
            is_directory: false,
        }
    }

    pub fn directory(name: &str) -> Entry {
        Entry {
            name: name.to_string(),
            is_directory: true,
        }
    }

    pub fn has_existing_app_state(_entries: &[Entry]) -> PortResult<bool> {
        stub()
    }
}

use install_state::{directory, file, has_existing_app_state, NOTE};

#[test]
#[ignore = "porting: desktop install-state policy not implemented"]
fn ignores_files_electron_may_create_on_a_fresh_install() {
    assert!(!has_existing_app_state(&[]).expect(NOTE));
    assert!(!has_existing_app_state(&[file("Local State"), directory("Crashpad")]).expect(NOTE));
}

#[test]
#[ignore = "porting: desktop install-state policy not implemented"]
fn recognizes_state_written_by_an_earlier_opencode_launch() {
    assert!(has_existing_app_state(&[file("opencode.settings")]).expect(NOTE));
    assert!(has_existing_app_state(&[file("opencode.global.dat")]).expect(NOTE));
    assert!(has_existing_app_state(&[file("window-state-abc.json")]).expect(NOTE));
    assert!(has_existing_app_state(&[directory("opencode")]).expect(NOTE));
}
