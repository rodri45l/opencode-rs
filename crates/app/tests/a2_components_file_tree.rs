//! Port of packages/app/src/components/file-tree.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, Default)]
struct DirState {
    loaded: Option<bool>,
    loading: Option<bool>,
    expanded: Option<bool>,
}

#[derive(Clone, Debug, Default)]
struct NodeInput {
    level: i32,
    dir: Option<DirState>,
}

// Local stubs (fast wave): real module lands later.
fn should_list_root(_input: NodeInput) -> Option<bool> {
    None
}

fn should_list_expanded(_input: NodeInput) -> Option<bool> {
    None
}

fn dirs_to_expand(_level: i32, _dirs: &[&str], _expanded: &dyn Fn(&str) -> bool) -> Vec<String> {
    Vec::new()
}

#[test]
#[ignore = "porting: components/file-tree not implemented"]
fn root_lists_on_mount_unless_already_loaded_or_loading() {
    assert_eq!(
        should_list_root(NodeInput {
            level: 0,
            ..Default::default()
        }),
        Some(true)
    );
    assert_eq!(
        should_list_root(NodeInput {
            level: 0,
            dir: Some(DirState {
                loaded: Some(true),
                ..Default::default()
            }),
        }),
        Some(false)
    );
    assert_eq!(
        should_list_root(NodeInput {
            level: 0,
            dir: Some(DirState {
                loading: Some(true),
                ..Default::default()
            }),
        }),
        Some(false)
    );
    assert_eq!(
        should_list_root(NodeInput {
            level: 1,
            ..Default::default()
        }),
        Some(false)
    );
}

#[test]
#[ignore = "porting: components/file-tree not implemented"]
fn nested_dirs_list_only_when_expanded_and_stale() {
    assert_eq!(
        should_list_expanded(NodeInput {
            level: 1,
            ..Default::default()
        }),
        Some(false)
    );
    assert_eq!(
        should_list_expanded(NodeInput {
            level: 1,
            dir: Some(DirState {
                expanded: Some(false),
                ..Default::default()
            }),
        }),
        Some(false)
    );
    assert_eq!(
        should_list_expanded(NodeInput {
            level: 1,
            dir: Some(DirState {
                expanded: Some(true),
                ..Default::default()
            }),
        }),
        Some(true)
    );
    assert_eq!(
        should_list_expanded(NodeInput {
            level: 1,
            dir: Some(DirState {
                expanded: Some(true),
                loaded: Some(true),
                ..Default::default()
            }),
        }),
        Some(false)
    );
    assert_eq!(
        should_list_expanded(NodeInput {
            level: 1,
            dir: Some(DirState {
                expanded: Some(true),
                loading: Some(true),
                ..Default::default()
            }),
        }),
        Some(false)
    );
    assert_eq!(
        should_list_expanded(NodeInput {
            level: 0,
            dir: Some(DirState {
                expanded: Some(true),
                ..Default::default()
            }),
        }),
        Some(false)
    );
}

#[test]
#[ignore = "porting: components/file-tree not implemented"]
fn allowed_auto_expand_picks_only_collapsed_dirs() {
    let dirs = ["src", "src/components"];
    let mut expanded: Vec<String> = Vec::new();

    let first = dirs_to_expand(0, &dirs, &|dir| expanded.iter().any(|d| d == dir));
    assert_eq!(first, vec!["src".to_string(), "src/components".to_string()]);

    for dir in &first {
        expanded.push(dir.clone());
    }

    let second = dirs_to_expand(0, &dirs, &|dir| expanded.iter().any(|d| d == dir));
    assert_eq!(second, Vec::<String>::new());
    assert_eq!(dirs_to_expand(1, &dirs, &|_| false), Vec::<String>::new());
}
