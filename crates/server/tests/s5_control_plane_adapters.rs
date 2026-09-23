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

    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    fn store() -> &'static Mutex<HashMap<(String, String), Adapter>> {
        static STORE: OnceLock<Mutex<HashMap<(String, String), Adapter>>> = OnceLock::new();
        STORE.get_or_init(|| Mutex::new(HashMap::new()))
    }

    pub fn register_adapter(project: &str, ty: &str, adapter: Adapter) -> Result<(), &'static str> {
        store()
            .lock()
            .map_err(|_| "poisoned")?
            .insert((project.to_string(), ty.to_string()), adapter);
        Ok(())
    }

    pub fn get_adapter(project: &str, ty: &str) -> Result<Option<Adapter>, &'static str> {
        Ok(store()
            .lock()
            .map_err(|_| "poisoned")?
            .get(&(project.to_string(), ty.to_string()))
            .cloned())
    }
}

fn target(directory: &str) -> adapters::Target {
    adapters::Target {
        kind: "local".to_string(),
        directory: directory.to_string(),
    }
}

#[test]
fn isolates_custom_adapters_by_project() {
    let ty = "demo";
    let one = "project-iso-one";
    let two = "project-iso-two";
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
fn latest_install_wins_within_a_project() {
    let ty = "demo";
    let id = "project-latest";
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
