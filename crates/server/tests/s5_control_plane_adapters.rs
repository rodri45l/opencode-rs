//! Port of packages/opencode/test/control-plane/adapters.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: custom adapters are isolated per project and the latest
//! registration within a project wins.
#![allow(dead_code)]

// Fast-wave local stubs: `control_plane::adapters` is not implemented in this
// crate yet, so the registry surface is defined here.
mod adapters {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Target {
        pub kind: String,
        pub directory: String,
    }

    #[derive(Debug, Clone)]
    pub struct Adapter {
        pub name: String,
        pub description: String,
        pub directory: String,
    }

    impl Adapter {
        pub fn new(directory: &str) -> Adapter {
            Adapter {
                name: directory.to_string(),
                description: directory.to_string(),
                directory: directory.to_string(),
            }
        }

        pub fn target(&self) -> Target {
            Target {
                kind: "local".to_string(),
                directory: self.directory.clone(),
            }
        }
    }

    pub fn register_adapter(
        _project: &str,
        _ty: &str,
        _adapter: Adapter,
    ) -> Result<(), &'static str> {
        Err("porting: registerAdapter not implemented")
    }

    pub fn get_adapter(_project: &str, _ty: &str) -> Result<Option<Adapter>, &'static str> {
        Err("porting: getAdapter not implemented")
    }
}

fn target(directory: &str) -> adapters::Target {
    adapters::Target {
        kind: "local".to_string(),
        directory: directory.to_string(),
    }
}

#[test]
#[ignore = "porting: control-plane-adapters not implemented"]
fn isolates_custom_adapters_by_project() {
    let ty = "demo";
    let one = "project-one";
    let two = "project-two";
    adapters::register_adapter(one, ty, adapters::Adapter::new("/one")).unwrap();
    adapters::register_adapter(two, ty, adapters::Adapter::new("/two")).unwrap();

    assert_eq!(
        adapters::get_adapter(one, ty).unwrap().unwrap().target(),
        target("/one")
    );
    assert_eq!(
        adapters::get_adapter(two, ty).unwrap().unwrap().target(),
        target("/two")
    );
}

#[test]
#[ignore = "porting: control-plane-adapters not implemented"]
fn latest_install_wins_within_a_project() {
    let ty = "demo";
    let id = "project-one";
    adapters::register_adapter(id, ty, adapters::Adapter::new("/one")).unwrap();

    assert_eq!(
        adapters::get_adapter(id, ty).unwrap().unwrap().target(),
        target("/one")
    );

    adapters::register_adapter(id, ty, adapters::Adapter::new("/two")).unwrap();

    assert_eq!(
        adapters::get_adapter(id, ty).unwrap().unwrap().target(),
        target("/two")
    );
}
