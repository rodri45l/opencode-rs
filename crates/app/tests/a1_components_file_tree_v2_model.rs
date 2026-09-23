//! Port of packages/app/src/components/file-tree-v2-model.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use opencode_app::file_tree_v2_model::{
    build_file_tree_v2_model, flatten_file_tree_v2, flatten_live_file_tree_v2, FileNode,
};

fn file_node(name: &str, path: &str, node_type: &str) -> FileNode {
    FileNode {
        path: path.to_string(),
        node_type: node_type.to_string(),
        original_path: name.to_string(),
    }
}

#[test]
fn builds_a_sorted_tree_and_flattens_expanded_directories() {
    let model = build_file_tree_v2_model(&[
        "src/z.ts",
        "src/lib/b.ts",
        "src/lib/a.ts",
        "README.md",
        "docs/guide.md",
    ]);
    assert_eq!(model.total, 8);
    assert_eq!(
        flatten_file_tree_v2(&model, |_| true)
            .iter()
            .map(|row| (row.node.path.clone(), row.node.node_type.clone(), row.level))
            .collect::<Vec<_>>(),
        vec![
            ("docs".to_string(), "directory".to_string(), 0),
            ("docs/guide.md".to_string(), "file".to_string(), 1),
            ("src".to_string(), "directory".to_string(), 0),
            ("src/lib".to_string(), "directory".to_string(), 1),
            ("src/lib/a.ts".to_string(), "file".to_string(), 2),
            ("src/lib/b.ts".to_string(), "file".to_string(), 2),
            ("src/z.ts".to_string(), "file".to_string(), 1),
            ("README.md".to_string(), "file".to_string(), 0),
        ]
    );
}

#[test]
fn skips_children_of_collapsed_directories() {
    let model = build_file_tree_v2_model(&["src/lib/a.ts", "src/z.ts"]);
    assert_eq!(
        flatten_file_tree_v2(&model, |path| path != "src/lib")
            .iter()
            .map(|row| row.node.path.clone())
            .collect::<Vec<_>>(),
        vec![
            "src".to_string(),
            "src/lib".to_string(),
            "src/z.ts".to_string()
        ]
    );
}

#[test]
fn normalizes_duplicate_and_messy_paths() {
    let model = build_file_tree_v2_model(&["src\\lib\\a.ts", "src/lib/a.ts", "/src//lib/b.ts/"]);
    let rows = flatten_file_tree_v2(&model, |_| true);
    assert_eq!(
        rows.iter()
            .map(|row| row.node.path.clone())
            .collect::<Vec<_>>(),
        vec![
            "src".to_string(),
            "src/lib".to_string(),
            "src/lib/a.ts".to_string(),
            "src/lib/b.ts".to_string()
        ]
    );
    assert_eq!(
        rows.iter()
            .find(|row| row.node.path == "src/lib/a.ts")
            .map(|row| row.node.original_path.clone()),
        Some("src\\lib\\a.ts".to_string())
    );
}

#[test]
fn handles_deeply_nested_paths() {
    let file = format!(
        "{}/leaf.ts",
        (0..130)
            .map(|index| format!("d{index}"))
            .collect::<Vec<_>>()
            .join("/")
    );
    let model = build_file_tree_v2_model(&[&file]);
    assert_eq!(flatten_file_tree_v2(&model, |_| true).len(), 131);
}

#[test]
fn flattens_live_children_using_original_paths_for_nested_lookups() {
    let mut nodes: std::collections::BTreeMap<String, Vec<FileNode>> =
        std::collections::BTreeMap::new();
    nodes.insert(
        String::new(),
        vec![
            file_node("src", "src", "directory"),
            file_node("README.md", "README.md", "file"),
        ],
    );
    nodes.insert(
        "src".into(),
        vec![
            file_node("a.ts", "src/a.ts", "file"),
            file_node("lib", "src/lib", "directory"),
        ],
    );
    nodes.insert(
        "src/lib".into(),
        vec![file_node("b.ts", "src/lib/b.ts", "file")],
    );

    assert_eq!(
        flatten_live_file_tree_v2(
            |path| nodes.get(path).cloned().unwrap_or_default(),
            |path| path == "src"
        )
        .iter()
        .map(|row| (
            row.node.path.clone(),
            row.node.original_path.clone(),
            row.level
        ))
        .collect::<Vec<_>>(),
        vec![
            ("src".to_string(), "src".to_string(), 0),
            ("src/a.ts".to_string(), "src/a.ts".to_string(), 1),
            ("src/lib".to_string(), "src/lib".to_string(), 1),
            ("README.md".to_string(), "README.md".to_string(), 0),
        ]
    );
}
