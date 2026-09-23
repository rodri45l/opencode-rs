//! Port of packages/tui/test/feature-plugins/diff-viewer-file-tree-utils.test.ts
//! (upstream 18ef3cc). Behaviour pinned by
//! packages/tui/src/feature-plugins/system/diff-viewer-file-tree-utils.ts; see
//! docs/TEST-PORT.md.

use opencode_tui::diff_viewer_file_tree::{
    all_expanded_file_tree_directories, build_file_tree, file_tree_file_selection,
    flatten_file_tree, move_file_tree_selection, move_file_tree_selection_to_file,
    move_file_tree_selection_to_first_child, move_file_tree_selection_to_parent,
    move_patch_file_index, ordered_patch_file_indexes, set_file_tree_directory_expanded,
    show_diff_viewer_file_tree, single_patch_file_index, toggle_file_tree_directory, FileTree,
    FileTreeItem, FileTreeRow, NodeKind,
};
use std::collections::HashSet;

fn files(paths: &[&str]) -> Vec<FileTreeItem> {
    paths
        .iter()
        .map(|file| FileTreeItem {
            file: file.to_string(),
            status: None,
        })
        .collect()
}

fn kind(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Directory => "directory",
        NodeKind::File => "file",
    }
}

fn row_labels(rows: &[FileTreeRow]) -> Vec<String> {
    rows.iter()
        .map(|row| format!("{}{}:{}", "  ".repeat(row.depth), kind(row.kind), row.name))
        .collect()
}

fn dir_id(tree: &FileTree, name: &str) -> usize {
    tree.nodes
        .iter()
        .find(|node| node.kind == NodeKind::Directory && node.name == name)
        .expect("directory")
        .id
}

fn file_id(tree: &FileTree, name: &str) -> usize {
    tree.nodes
        .iter()
        .find(|node| node.kind == NodeKind::File && node.name == name)
        .expect("file")
        .id
}

#[test]
fn builds_a_nested_tree_with_deduplicated_directories_and_file_indexes() {
    let tree = build_file_tree(&files(&[
        "src/config/tui.ts",
        "src/config/keybind.ts",
        "src/session/index.ts",
    ]));

    assert_eq!(
        tree.nodes
            .iter()
            .filter(|node| node.kind == NodeKind::Directory && node.name == "src")
            .count(),
        1
    );
    assert_eq!(
        tree.nodes
            .iter()
            .filter(|node| node.kind == NodeKind::Directory && node.name == "config")
            .count(),
        1
    );
    assert_eq!(
        tree.nodes
            .iter()
            .filter(|node| node.kind == NodeKind::Directory && node.name == "session")
            .count(),
        1
    );
    assert_eq!(
        tree.nodes
            .iter()
            .filter(|node| node.kind == NodeKind::File)
            .map(|node| (node.name.clone(), node.file_index, node.depth))
            .collect::<Vec<_>>(),
        vec![
            ("tui.ts".to_string(), Some(0), 2),
            ("keybind.ts".to_string(), Some(1), 2),
            ("index.ts".to_string(), Some(2), 2),
        ]
    );
}

#[test]
fn sorts_directories_before_files_and_alphabetically_within_each_group() {
    let rows = flatten_file_tree(
        &build_file_tree(&files(&[
            "z-file.ts",
            "b/file.ts",
            "a/zeta.ts",
            "b/alpha.ts",
            "a/alpha.ts",
        ])),
        None,
    );

    assert_eq!(
        row_labels(&rows),
        vec![
            "directory:a",
            "  file:alpha.ts",
            "  file:zeta.ts",
            "directory:b",
            "  file:alpha.ts",
            "  file:file.ts",
            "file:z-file.ts",
        ]
    );
}

#[test]
fn sorts_root_level_files_without_creating_directories() {
    let tree = build_file_tree(&files(&["zeta.ts", "alpha.ts", "beta.ts"]));

    assert!(tree.nodes.iter().all(|node| node.kind == NodeKind::File));
    assert_eq!(
        flatten_file_tree(&tree, None)
            .iter()
            .map(|row| row.name.clone())
            .collect::<Vec<_>>(),
        vec!["alpha.ts", "beta.ts", "zeta.ts"]
    );
}

#[test]
fn collapses_unary_directory_chains_while_flattening() {
    let rows = flatten_file_tree(
        &build_file_tree(&files(&[
            "packages/opencode/src/cli/app.ts",
            "packages/opencode/src/server/server.ts",
        ])),
        None,
    );

    assert_eq!(
        row_labels(&rows),
        vec![
            "directory:packages/opencode/src",
            "  directory:cli",
            "    file:app.ts",
            "  directory:server",
            "    file:server.ts",
        ]
    );
}

#[test]
fn does_not_collapse_a_directory_into_a_file_row() {
    let rows = flatten_file_tree(
        &build_file_tree(&files(&["packages/opencode/src/app.ts"])),
        None,
    );

    assert_eq!(
        row_labels(&rows),
        vec!["directory:packages/opencode/src", "  file:app.ts"]
    );
}

#[test]
fn stops_collapsing_at_branches() {
    let rows = flatten_file_tree(
        &build_file_tree(&files(&[
            "packages/opencode/src/cli/app.ts",
            "packages/opencode/src/server/server.ts",
            "packages/readme.md",
        ])),
        None,
    );

    assert_eq!(
        row_labels(&rows),
        vec![
            "directory:packages",
            "  directory:opencode/src",
            "    directory:cli",
            "      file:app.ts",
            "    directory:server",
            "      file:server.ts",
            "  file:readme.md",
        ]
    );
}

#[test]
fn keeps_same_directory_names_under_different_parents_separate() {
    let rows = flatten_file_tree(
        &build_file_tree(&files(&[
            "components/button.ts",
            "docs/components/usage.md",
        ])),
        None,
    );

    assert_eq!(
        row_labels(&rows),
        vec![
            "directory:components",
            "  file:button.ts",
            "directory:docs/components",
            "  file:usage.md",
        ]
    );
}

#[test]
fn flattens_all_expanded_rows_depth_first_with_depths_and_file_references() {
    let rows = flatten_file_tree(
        &build_file_tree(&files(&[
            "src/config/tui.ts",
            "src/config/keybind.ts",
            "README.md",
        ])),
        None,
    );

    assert_eq!(
        rows.iter()
            .map(|row| (row.name.clone(), kind(row.kind), row.depth, row.file_index))
            .collect::<Vec<_>>(),
        vec![
            ("src/config".to_string(), "directory", 0, None),
            ("keybind.ts".to_string(), "file", 1, Some(1)),
            ("tui.ts".to_string(), "file", 1, Some(0)),
            ("README.md".to_string(), "file", 0, Some(2)),
        ]
    );
}

#[test]
fn collapses_expanded_unary_children_under_the_first_visible_directory_id() {
    let tree = build_file_tree(&files(&[
        "packages/opencode/src/cli/app.ts",
        "packages/opencode/src/server/server.ts",
    ]));
    let packages = dir_id(&tree, "packages");

    let empty = HashSet::new();
    assert_eq!(
        flatten_file_tree(&tree, Some(&empty))
            .iter()
            .map(|row| row.name.clone())
            .collect::<Vec<_>>(),
        vec!["packages/opencode/src"]
    );
    let expanded: HashSet<usize> = [packages].into_iter().collect();
    assert_eq!(
        flatten_file_tree(&tree, Some(&expanded))
            .iter()
            .map(|row| row.name.clone())
            .collect::<Vec<_>>(),
        vec!["packages/opencode/src", "cli", "server"]
    );
}

#[test]
fn flattens_only_expanded_directory_descendants_when_expansion_is_provided() {
    let tree = build_file_tree(&files(&[
        "src/config/tui.ts",
        "src/session/index.ts",
        "README.md",
    ]));
    let src = dir_id(&tree, "src");
    let config = dir_id(&tree, "config");

    let empty = HashSet::new();
    assert_eq!(
        flatten_file_tree(&tree, Some(&empty))
            .iter()
            .map(|row| row.name.clone())
            .collect::<Vec<_>>(),
        vec!["src", "README.md"]
    );
    let src_only: HashSet<usize> = [src].into_iter().collect();
    assert_eq!(
        flatten_file_tree(&tree, Some(&src_only))
            .iter()
            .map(|row| row.name.clone())
            .collect::<Vec<_>>(),
        vec!["src", "config", "session", "README.md"]
    );
    let both: HashSet<usize> = [src, config].into_iter().collect();
    assert_eq!(
        flatten_file_tree(&tree, Some(&both))
            .iter()
            .map(|row| row.name.clone())
            .collect::<Vec<_>>(),
        vec!["src", "config", "tui.ts", "session", "README.md"]
    );
}

#[test]
fn moves_selection_across_visible_rows_and_clamps_to_bounds() {
    let rows = flatten_file_tree(
        &build_file_tree(&files(&["src/config/tui.ts", "README.md"])),
        None,
    );

    assert_eq!(move_file_tree_selection(&rows, None, 1), Some(rows[0].id));
    assert_eq!(
        move_file_tree_selection(&rows, Some(rows[0].id), 1),
        Some(rows[1].id)
    );
    assert_eq!(
        move_file_tree_selection(&rows, Some(rows[1].id), 99),
        Some(rows[rows.len() - 1].id)
    );
    assert_eq!(
        move_file_tree_selection(&rows, Some(rows[1].id), -99),
        Some(rows[0].id)
    );
    assert_eq!(move_file_tree_selection(&[], None, 1), None);
}

#[test]
fn moves_directory_selection_to_first_visible_child() {
    let rows = flatten_file_tree(
        &build_file_tree(&files(&["src/config/tui.ts", "src/session/index.ts"])),
        None,
    );
    let src = dir_id_from_rows(&rows, "src");
    let config = dir_id_from_rows(&rows, "config");
    let tui = file_id_from_rows(&rows, "tui.ts");

    assert_eq!(
        move_file_tree_selection_to_first_child(&rows, Some(src)),
        Some(config)
    );
    assert_eq!(
        move_file_tree_selection_to_first_child(&rows, Some(tui)),
        Some(tui)
    );
    assert_eq!(move_file_tree_selection_to_first_child(&rows, None), None);
}

#[test]
fn moves_collapsed_chain_selection_to_first_visible_child() {
    let rows = flatten_file_tree(
        &build_file_tree(&files(&[
            "packages/opencode/src/cli/app.ts",
            "packages/opencode/src/server/server.ts",
        ])),
        None,
    );
    let packages = dir_id_from_rows(&rows, "packages/opencode/src");
    let cli = dir_id_from_rows(&rows, "cli");

    assert_eq!(
        move_file_tree_selection_to_first_child(&rows, Some(packages)),
        Some(cli)
    );
}

#[test]
fn moves_file_and_collapsed_directory_selection_to_visible_parent() {
    let rows = flatten_file_tree(
        &build_file_tree(&files(&[
            "packages/opencode/src/cli/app.ts",
            "packages/opencode/src/server/server.ts",
        ])),
        None,
    );
    let root = dir_id_from_rows(&rows, "packages/opencode/src");
    let cli = dir_id_from_rows(&rows, "cli");
    let app = file_id_from_rows(&rows, "app.ts");

    assert_eq!(
        move_file_tree_selection_to_parent(&rows, Some(app)),
        Some(cli)
    );
    assert_eq!(
        move_file_tree_selection_to_parent(&rows, Some(cli)),
        Some(root)
    );
    assert_eq!(
        move_file_tree_selection_to_parent(&rows, Some(root)),
        Some(root)
    );
    assert_eq!(move_file_tree_selection_to_parent(&rows, None), None);
}

#[test]
fn moves_file_selection_relative_to_the_highlighted_row() {
    let rows = flatten_file_tree(
        &build_file_tree(&files(&[
            "src/config/tui.ts",
            "src/session/index.ts",
            "README.md",
        ])),
        None,
    );
    let config = dir_id_from_rows(&rows, "config");
    let session = dir_id_from_rows(&rows, "session");
    let tui = file_id_from_rows(&rows, "tui.ts");
    let index = file_id_from_rows(&rows, "index.ts");
    let readme = file_id_from_rows(&rows, "README.md");

    assert_eq!(move_file_tree_selection_to_file(&rows, None, 1), Some(tui));
    assert_eq!(
        move_file_tree_selection_to_file(&rows, None, -1),
        Some(readme)
    );
    assert_eq!(
        move_file_tree_selection_to_file(&rows, Some(config), 1),
        Some(tui)
    );
    assert_eq!(
        move_file_tree_selection_to_file(&rows, Some(session), -1),
        Some(tui)
    );
    assert_eq!(
        move_file_tree_selection_to_file(&rows, Some(tui), 1),
        Some(index)
    );
    assert_eq!(
        move_file_tree_selection_to_file(&rows, Some(index), -1),
        Some(tui)
    );
    assert_eq!(
        move_file_tree_selection_to_file(&rows, Some(readme), 1),
        Some(readme)
    );
}

#[test]
fn selects_a_file_tree_node_and_expands_its_parents_for_a_patch_file() {
    let tree = build_file_tree(&files(&[
        "src/config/tui.ts",
        "src/session/index.ts",
        "README.md",
    ]));
    let selection = file_tree_file_selection(&tree, 1).expect("selection");

    assert_eq!(
        Some(selection.highlighted_node),
        tree.nodes
            .iter()
            .find(|node| node.kind == NodeKind::File && node.name == "index.ts")
            .map(|node| node.id)
    );
    let names: std::collections::BTreeSet<String> = selection
        .expanded_nodes
        .iter()
        .map(|id| tree.nodes[*id].name.clone())
        .collect();
    assert_eq!(
        names,
        ["session".to_string(), "src".to_string()]
            .into_iter()
            .collect()
    );
    assert!(file_tree_file_selection(&tree, 99).is_none());
}

#[test]
fn prefers_the_selected_file_when_choosing_the_single_patch_file() {
    assert_eq!(
        single_patch_file_index(Some(2), Some(1), Some(0), Some(3)),
        Some(2)
    );
    assert_eq!(
        single_patch_file_index(None, Some(1), Some(0), Some(3)),
        Some(1)
    );
    assert_eq!(
        single_patch_file_index(None, None, Some(0), Some(3)),
        Some(0)
    );
    assert_eq!(single_patch_file_index(None, None, None, Some(3)), Some(3));
}

#[test]
fn orders_patches_by_the_flattened_file_tree_order() {
    let rows = flatten_file_tree(
        &build_file_tree(&files(&[
            "src/dir-8/juniper-4.ts",
            "src/dir-8/harbor-94.ts",
            "src/dir-8/cedar-16.ts",
        ])),
        None,
    );

    assert_eq!(ordered_patch_file_indexes(&rows), vec![2, 1, 0]);
}

#[test]
fn shows_the_diff_viewer_file_tree_only_when_enabled_and_files_exist() {
    assert!(show_diff_viewer_file_tree(true, 1));
    assert!(!show_diff_viewer_file_tree(true, 0));
    assert!(!show_diff_viewer_file_tree(false, 1));
    assert!(!show_diff_viewer_file_tree(false, 0));
}

#[test]
fn moves_patch_selection_through_the_ordered_patch_file_indexes() {
    let file_indexes = [2, 1, 0];

    assert_eq!(move_patch_file_index(&file_indexes, None, 1), Some(2));
    assert_eq!(move_patch_file_index(&file_indexes, None, -1), Some(2));
    assert_eq!(move_patch_file_index(&file_indexes, Some(2), 1), Some(1));
    assert_eq!(move_patch_file_index(&file_indexes, Some(1), -1), Some(2));
    assert_eq!(move_patch_file_index(&file_indexes, Some(0), 1), Some(0));
    assert_eq!(move_patch_file_index(&file_indexes, Some(99), 1), Some(2));
    assert_eq!(move_patch_file_index(&file_indexes, Some(99), -1), Some(2));
    assert_eq!(move_patch_file_index(&[], None, 1), None);
}

#[test]
fn toggles_only_selected_directory_expansion() {
    let tree = build_file_tree(&files(&["src/config/tui.ts", "README.md"]));
    let src = dir_id(&tree, "src");
    let readme = file_id(&tree, "README.md");
    let expanded = all_expanded_file_tree_directories(&tree);

    let collapsed = toggle_file_tree_directory(&tree, &expanded, Some(src));
    assert!(!collapsed.contains(&src));
    assert_eq!(
        flatten_file_tree(&tree, Some(&collapsed))
            .iter()
            .map(|row| row.name.clone())
            .collect::<Vec<_>>(),
        vec!["src/config", "README.md"]
    );

    let reopened = toggle_file_tree_directory(&tree, &collapsed, Some(src));
    assert!(reopened.contains(&src));

    assert_eq!(
        toggle_file_tree_directory(&tree, &reopened, Some(readme)),
        reopened
    );
    assert_eq!(toggle_file_tree_directory(&tree, &reopened, None), reopened);
}

#[test]
fn sets_only_selected_directory_expansion() {
    let tree = build_file_tree(&files(&["src/config/tui.ts", "README.md"]));
    let src = dir_id(&tree, "src");
    let readme = file_id(&tree, "README.md");
    let expanded = all_expanded_file_tree_directories(&tree);

    let collapsed = set_file_tree_directory_expanded(&tree, &expanded, Some(src), false);
    assert!(!collapsed.contains(&src));

    let reopened = set_file_tree_directory_expanded(&tree, &collapsed, Some(src), true);
    assert!(reopened.contains(&src));

    assert_eq!(
        set_file_tree_directory_expanded(&tree, &reopened, Some(readme), false),
        reopened
    );
    assert_eq!(
        set_file_tree_directory_expanded(&tree, &reopened, None, false),
        reopened
    );
}

fn dir_id_from_rows(rows: &[FileTreeRow], name: &str) -> usize {
    rows.iter()
        .find(|row| row.kind == NodeKind::Directory && row.name == name)
        .expect("directory row")
        .id
}

fn file_id_from_rows(rows: &[FileTreeRow], name: &str) -> usize {
    rows.iter()
        .find(|row| row.kind == NodeKind::File && row.name == name)
        .expect("file row")
        .id
}
