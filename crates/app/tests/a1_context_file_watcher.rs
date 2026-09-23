//! Port of packages/app/src/context/file/watcher.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct Node {
    path: String,
    node_type: String,
    ignored: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct WatcherEvent {
    event_type: String,
    file: String,
    event: String,
}

#[derive(Default)]
struct Invalidation {
    loads: Vec<String>,
    refresh: Vec<String>,
}

// Local stub (fast wave): real module lands later.
fn invalidate_from_watcher(
    _event: &WatcherEvent,
    _context: &WatcherContext,
    _out: &mut Invalidation,
) {
}

type NodeLookup = Box<dyn Fn(&str) -> Option<Node>>;

struct WatcherContext {
    has_file: Box<dyn Fn(&str) -> bool>,
    is_open: Box<dyn Fn(&str) -> bool>,
    node: NodeLookup,
    is_dir_loaded: Box<dyn Fn(&str) -> bool>,
}

#[test]
#[ignore = "porting: context/file/watcher not implemented"]
fn reloads_open_files_and_refreshes_loaded_parent_on_add() {
    let mut out = Invalidation::default();
    invalidate_from_watcher(
        &WatcherEvent {
            event_type: "file.watcher.updated".into(),
            file: "src/new.ts".into(),
            event: "add".into(),
        },
        &WatcherContext {
            has_file: Box::new(|path| path == "src/new.ts"),
            is_open: Box::new(|_| false),
            node: Box::new(|_| None),
            is_dir_loaded: Box::new(|path| path == "src"),
        },
        &mut out,
    );
    assert_eq!(out.loads, vec!["src/new.ts".to_string()]);
    assert_eq!(out.refresh, vec!["src".to_string()]);
}

#[test]
#[ignore = "porting: context/file/watcher not implemented"]
fn reloads_files_that_are_open_in_tabs() {
    let mut out = Invalidation::default();
    invalidate_from_watcher(
        &WatcherEvent {
            event_type: "file.watcher.updated".into(),
            file: "src/open.ts".into(),
            event: "change".into(),
        },
        &WatcherContext {
            has_file: Box::new(|_| false),
            is_open: Box::new(|path| path == "src/open.ts"),
            node: Box::new(|_| {
                Some(Node {
                    path: "src/open.ts".into(),
                    node_type: "file".into(),
                    ignored: false,
                })
            }),
            is_dir_loaded: Box::new(|_| false),
        },
        &mut out,
    );
    assert_eq!(out.loads, vec!["src/open.ts".to_string()]);
}

#[test]
#[ignore = "porting: context/file/watcher not implemented"]
fn refreshes_only_changed_loaded_directory_nodes() {
    let mut out = Invalidation::default();
    invalidate_from_watcher(
        &WatcherEvent {
            event_type: "file.watcher.updated".into(),
            file: "src".into(),
            event: "change".into(),
        },
        &WatcherContext {
            has_file: Box::new(|_| false),
            is_open: Box::new(|_| false),
            node: Box::new(|_| {
                Some(Node {
                    path: "src".into(),
                    node_type: "directory".into(),
                    ignored: false,
                })
            }),
            is_dir_loaded: Box::new(|path| path == "src"),
        },
        &mut out,
    );
    invalidate_from_watcher(
        &WatcherEvent {
            event_type: "file.watcher.updated".into(),
            file: "src/file.ts".into(),
            event: "change".into(),
        },
        &WatcherContext {
            has_file: Box::new(|_| false),
            is_open: Box::new(|_| false),
            node: Box::new(|_| {
                Some(Node {
                    path: "src/file.ts".into(),
                    node_type: "file".into(),
                    ignored: false,
                })
            }),
            is_dir_loaded: Box::new(|_| true),
        },
        &mut out,
    );
    assert_eq!(out.refresh, vec!["src".to_string()]);
}

#[test]
#[ignore = "porting: context/file/watcher not implemented"]
fn ignores_invalid_or_git_watcher_updates() {
    let mut out = Invalidation::default();
    invalidate_from_watcher(
        &WatcherEvent {
            event_type: "file.watcher.updated".into(),
            file: ".git/index.lock".into(),
            event: "change".into(),
        },
        &WatcherContext {
            has_file: Box::new(|_| true),
            is_open: Box::new(|_| false),
            node: Box::new(|_| None),
            is_dir_loaded: Box::new(|_| true),
        },
        &mut out,
    );
    invalidate_from_watcher(
        &WatcherEvent {
            event_type: "project.updated".into(),
            file: String::new(),
            event: String::new(),
        },
        &WatcherContext {
            has_file: Box::new(|_| false),
            is_open: Box::new(|_| false),
            node: Box::new(|_| None),
            is_dir_loaded: Box::new(|_| true),
        },
        &mut out,
    );
    assert!(out.refresh.is_empty());
}
