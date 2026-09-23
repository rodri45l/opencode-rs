//! Port of packages/desktop/src/main/window-registry.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/window-registry.ts: persisted
//! ids are restored while malformed entries are dropped; registration persists
//! each id once; deliberately closing a window forgets it unless it was the last
//! one or the app is quitting; and the last focused window falls back on close.
//! Re-derived: the JSON persistence blob is represented as a typed id list.

#[allow(dead_code)]
mod window_registry {
    use std::fmt;
    use std::marker::PhantomData;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: desktop window registry not implemented";

    fn stub<T>() -> PortResult<T> {
        Err(NotImplemented(NOTE))
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PersistedId {
        Id(String),
        Invalid,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Window {
        pub name: String,
    }

    pub struct WindowRegistry<T> {
        _marker: PhantomData<T>,
    }

    impl<T: Clone> WindowRegistry<T> {
        pub fn persisted(&self) -> PortResult<Vec<String>> {
            stub()
        }

        pub fn register(&mut self, _id: &str, _window: T) {}

        pub fn closed(&mut self, _id: &str) {}

        pub fn set_quitting(&mut self) {}

        pub fn set_quitting_flag(&mut self, _value: bool) {}

        pub fn focused(&mut self, _id: &str) {}

        pub fn last_focused(&self) -> PortResult<Option<T>> {
            stub()
        }
    }

    pub fn create_window_registry<T>(
        _read: impl Fn() -> Vec<PersistedId>,
        _write: impl Fn(&[String]),
        _cleanup: impl Fn(&str),
    ) -> WindowRegistry<T> {
        WindowRegistry {
            _marker: PhantomData,
        }
    }
}

use std::cell::RefCell;
use std::rc::Rc;

use window_registry::{create_window_registry, PersistedId, Window, WindowRegistry, NOTE};

fn ids(names: &[&str]) -> Vec<PersistedId> {
    names
        .iter()
        .map(|name| PersistedId::Id((*name).to_string()))
        .collect()
}

type Setup = (
    WindowRegistry<Window>,
    Rc<RefCell<Vec<PersistedId>>>,
    Rc<RefCell<Vec<String>>>,
);

fn setup(initial: Vec<PersistedId>) -> Setup {
    let stored = Rc::new(RefCell::new(initial));
    let cleaned = Rc::new(RefCell::new(Vec::new()));
    let read_store = stored.clone();
    let write_store = stored.clone();
    let cleanup_store = cleaned.clone();

    let registry: WindowRegistry<Window> = create_window_registry(
        move || read_store.borrow().clone(),
        move |ids: &[String]| {
            *write_store.borrow_mut() = ids.iter().map(|id| PersistedId::Id(id.clone())).collect();
        },
        move |id: &str| cleanup_store.borrow_mut().push(id.to_string()),
    );

    (registry, stored, cleaned)
}

#[test]
#[ignore = "porting: desktop window registry not implemented"]
fn restores_persisted_ids_and_ignores_malformed_entries() {
    let (registry, ..) = setup(ids(&["a", "", "b"]));
    assert_eq!(registry.persisted().expect(NOTE), vec!["a", "b"]);

    let (registry, ..) = setup(vec![PersistedId::Invalid]);
    assert_eq!(registry.persisted().expect(NOTE), Vec::<String>::new());

    let (registry, ..) = setup(Vec::new());
    assert_eq!(registry.persisted().expect(NOTE), Vec::<String>::new());
}

#[test]
#[ignore = "porting: desktop window registry not implemented"]
fn registers_windows_and_persists_each_id_once() {
    let (mut registry, stored, ..) = setup(Vec::new());
    registry.register(
        "a",
        Window {
            name: "a".to_string(),
        },
    );
    registry.register(
        "a",
        Window {
            name: "a".to_string(),
        },
    );
    registry.register(
        "b",
        Window {
            name: "b".to_string(),
        },
    );
    assert_eq!(*stored.borrow(), ids(&["a", "b"]));
}

#[test]
#[ignore = "porting: desktop window registry not implemented"]
fn forgets_a_deliberately_closed_window_while_others_remain_open() {
    let (mut registry, stored, cleaned) = setup(Vec::new());
    registry.register(
        "a",
        Window {
            name: "a".to_string(),
        },
    );
    registry.register(
        "b",
        Window {
            name: "b".to_string(),
        },
    );
    registry.closed("a");
    assert_eq!(*stored.borrow(), ids(&["b"]));
    assert_eq!(*cleaned.borrow(), vec!["a".to_string()]);
}

#[test]
#[ignore = "porting: desktop window registry not implemented"]
fn keeps_the_id_when_the_last_window_closes_so_relaunch_restores_it() {
    let (mut registry, stored, cleaned) = setup(Vec::new());
    registry.register(
        "a",
        Window {
            name: "a".to_string(),
        },
    );
    registry.closed("a");
    assert_eq!(*stored.borrow(), ids(&["a"]));
    assert!(cleaned.borrow().is_empty());
}

#[test]
#[ignore = "porting: desktop window registry not implemented"]
fn keeps_every_id_when_windows_close_during_quit() {
    let (mut registry, stored, cleaned) = setup(Vec::new());
    registry.register(
        "a",
        Window {
            name: "a".to_string(),
        },
    );
    registry.register(
        "b",
        Window {
            name: "b".to_string(),
        },
    );
    registry.set_quitting();
    registry.closed("a");
    registry.closed("b");
    assert_eq!(*stored.borrow(), ids(&["a", "b"]));
    assert!(cleaned.borrow().is_empty());
}

#[test]
#[ignore = "porting: desktop window registry not implemented"]
fn tracks_the_last_focused_window_and_falls_back_on_close() {
    let (mut registry, ..) = setup(Vec::new());
    registry.register(
        "a",
        Window {
            name: "a".to_string(),
        },
    );
    registry.register(
        "b",
        Window {
            name: "b".to_string(),
        },
    );
    registry.focused("a");
    assert_eq!(
        registry.last_focused().expect(NOTE),
        Some(Window {
            name: "a".to_string()
        })
    );
    registry.closed("a");
    assert_eq!(
        registry.last_focused().expect(NOTE),
        Some(Window {
            name: "b".to_string()
        })
    );
    registry.closed("b");
    assert_eq!(registry.last_focused().expect(NOTE), None);
}

#[test]
#[ignore = "porting: desktop window registry not implemented"]
fn resumes_forgetting_closed_windows_after_the_quit_flag_resets() {
    let (mut registry, stored, cleaned) = setup(Vec::new());
    registry.register(
        "a",
        Window {
            name: "a".to_string(),
        },
    );
    registry.register(
        "b",
        Window {
            name: "b".to_string(),
        },
    );
    registry.set_quitting();
    registry.set_quitting_flag(false);
    registry.closed("a");
    assert_eq!(*stored.borrow(), ids(&["b"]));
    assert_eq!(*cleaned.borrow(), vec!["a".to_string()]);
}
