//! Diff-viewer file tree model.
//!
//! Port of
//! packages/tui/src/feature-plugins/system/diff-viewer-file-tree-utils.ts
//! behaviour (upstream 18ef3cc).

use std::collections::{HashMap, HashSet};

/// A file entry in the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeItem {
    pub file: String,
    pub status: Option<String>,
}

/// Whether a node is a directory or a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Directory,
    File,
}

/// A node in the file tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeNode {
    pub id: usize,
    pub name: String,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub depth: usize,
    pub kind: NodeKind,
    pub file_index: Option<usize>,
}

/// A built file tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileTree {
    pub roots: Vec<usize>,
    pub nodes: Vec<FileTreeNode>,
}

/// A flattened tree row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeRow {
    pub id: usize,
    pub depth: usize,
    pub kind: NodeKind,
    pub name: String,
    pub file_index: Option<usize>,
}

/// A file selection with its expanded parent directories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeSelection {
    pub highlighted_node: usize,
    pub expanded_nodes: HashSet<usize>,
}

/// Build a nested file tree from file paths.
pub fn build_file_tree(files: &[FileTreeItem]) -> FileTree {
    let mut roots = Vec::new();
    let mut nodes: Vec<FileTreeNode> = Vec::new();
    let mut directory_by_path: HashMap<String, usize> = HashMap::new();

    for (file_index, file) in files.iter().enumerate() {
        let segments: Vec<&str> = file
            .file
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();
        if segments.is_empty() {
            continue;
        }
        let mut parent: Option<usize> = None;
        let mut path = String::new();
        let mut depth = 0;
        for segment in &segments[..segments.len() - 1] {
            if !path.is_empty() {
                path.push('/');
            }
            path.push_str(segment);
            if let Some(existing) = directory_by_path.get(&path) {
                parent = Some(*existing);
                depth += 1;
                continue;
            }
            let id = add_node(
                &mut nodes,
                &mut roots,
                segment,
                parent,
                depth,
                NodeKind::Directory,
                None,
            );
            directory_by_path.insert(path.clone(), id);
            parent = Some(id);
            depth += 1;
        }
        add_node(
            &mut nodes,
            &mut roots,
            segments[segments.len() - 1],
            parent,
            depth,
            NodeKind::File,
            Some(file_index),
        );
    }

    let mut tree = FileTree { roots, nodes };
    let mut roots = std::mem::take(&mut tree.roots);
    roots.sort_by(|left, right| compare_file_tree_nodes(&tree, *left, *right));
    tree.roots = roots;
    let sorted_children: Vec<Vec<usize>> = tree
        .nodes
        .iter()
        .map(|node| {
            let mut children = node.children.clone();
            children.sort_by(|left, right| compare_file_tree_nodes(&tree, *left, *right));
            children
        })
        .collect();
    for (node, children) in tree.nodes.iter_mut().zip(sorted_children) {
        node.children = children;
    }
    tree
}

fn add_node(
    nodes: &mut Vec<FileTreeNode>,
    roots: &mut Vec<usize>,
    name: &str,
    parent: Option<usize>,
    depth: usize,
    kind: NodeKind,
    file_index: Option<usize>,
) -> usize {
    let id = nodes.len();
    nodes.push(FileTreeNode {
        id,
        name: name.to_string(),
        parent,
        children: Vec::new(),
        depth,
        kind,
        file_index,
    });
    match parent {
        Some(parent) => nodes[parent].children.push(id),
        None => roots.push(id),
    }
    id
}

/// Order nodes: directories first, then by name, then by id.
pub fn compare_file_tree_nodes(tree: &FileTree, left: usize, right: usize) -> std::cmp::Ordering {
    let left_node = &tree.nodes[left];
    let right_node = &tree.nodes[right];
    if left_node.kind != right_node.kind {
        return if left_node.kind == NodeKind::Directory {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        };
    }
    left_node
        .name
        .cmp(&right_node.name)
        .then_with(|| left.cmp(&right))
}

/// Flatten a tree into rows, collapsing unary directory chains.
pub fn flatten_file_tree(tree: &FileTree, expanded: Option<&HashSet<usize>>) -> Vec<FileTreeRow> {
    let mut rows = Vec::new();
    for root in &tree.roots {
        visit(tree, *root, 0, expanded, &mut rows);
    }
    rows
}

fn visit(
    tree: &FileTree,
    id: usize,
    depth: usize,
    expanded: Option<&HashSet<usize>>,
    rows: &mut Vec<FileTreeRow>,
) {
    let node = &tree.nodes[id];
    if node.kind == NodeKind::File {
        rows.push(FileTreeRow {
            id: node.id,
            depth,
            kind: node.kind,
            name: node.name.clone(),
            file_index: node.file_index,
        });
        return;
    }
    let chain = collapsed_directory_chain(tree, id);
    let last = *chain.last().unwrap();
    rows.push(FileTreeRow {
        id: node.id,
        depth,
        kind: node.kind,
        name: chain
            .iter()
            .map(|node| node.name.as_str())
            .collect::<Vec<_>>()
            .join("/"),
        file_index: node.file_index,
    });
    if expanded.is_none_or(|expanded| expanded.contains(&id)) {
        for child in &last.children {
            visit(tree, *child, depth + 1, expanded, rows);
        }
    }
}

fn collapsed_directory_chain(tree: &FileTree, id: usize) -> Vec<&FileTreeNode> {
    let node = &tree.nodes[id];
    let child = if node.children.len() == 1 {
        Some(&tree.nodes[node.children[0]])
    } else {
        None
    };
    match child {
        Some(child) if child.kind == NodeKind::Directory => {
            let mut chain = vec![node];
            chain.extend(collapsed_directory_chain(tree, child.id));
            chain
        }
        _ => vec![node],
    }
}

/// Move selection across visible rows, clamping to bounds.
pub fn move_file_tree_selection(
    rows: &[FileTreeRow],
    selected: Option<usize>,
    offset: i64,
) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }
    let index = selected.and_then(|id| rows.iter().position(|row| row.id == id));
    let Some(index) = index else {
        return Some(rows[0].id);
    };
    let next = (index as i64 + offset).clamp(0, rows.len() as i64 - 1) as usize;
    Some(rows[next].id)
}

/// Move a directory selection to its first visible child.
pub fn move_file_tree_selection_to_first_child(
    rows: &[FileTreeRow],
    selected: Option<usize>,
) -> Option<usize> {
    let Some(index) = selected.and_then(|id| rows.iter().position(|row| row.id == id)) else {
        return selected;
    };
    let row = &rows[index];
    if row.kind != NodeKind::Directory {
        return selected;
    }
    match rows.get(index + 1) {
        Some(child) if child.depth > row.depth => Some(child.id),
        _ => selected,
    }
}

/// Move a selection to its visible parent directory.
pub fn move_file_tree_selection_to_parent(
    rows: &[FileTreeRow],
    selected: Option<usize>,
) -> Option<usize> {
    let index = selected.and_then(|id| rows.iter().position(|row| row.id == id));
    let Some(index) = index else {
        return selected;
    };
    let row = &rows[index];
    if row.depth == 0 {
        return selected;
    }
    rows[..index]
        .iter()
        .rev()
        .find(|item| item.depth < row.depth)
        .map(|item| item.id)
        .or(selected)
}

/// Move selection to the previous/next visible file row.
pub fn move_file_tree_selection_to_file(
    rows: &[FileTreeRow],
    selected: Option<usize>,
    offset: i64,
) -> Option<usize> {
    let file_rows: Vec<&FileTreeRow> = rows.iter().filter(|row| row.file_index.is_some()).collect();
    if file_rows.is_empty() {
        return None;
    }
    let selected_index = selected.and_then(|id| rows.iter().position(|row| row.id == id));
    let Some(selected_index) = selected_index else {
        return Some(if offset < 0 {
            file_rows[file_rows.len() - 1].id
        } else {
            file_rows[0].id
        });
    };
    let next = if offset < 0 {
        file_rows
            .iter()
            .rev()
            .find(|row| rows.iter().position(|item| item.id == row.id) < Some(selected_index))
    } else {
        file_rows
            .iter()
            .find(|row| rows.iter().position(|item| item.id == row.id) > Some(selected_index))
    };
    match next {
        Some(row) => Some(row.id),
        None => Some(if offset < 0 {
            file_rows[0].id
        } else {
            file_rows[file_rows.len() - 1].id
        }),
    }
}

/// Select a file node and expand its parent directories.
pub fn file_tree_file_selection(tree: &FileTree, file_index: usize) -> Option<FileTreeSelection> {
    let node = tree
        .nodes
        .iter()
        .find(|node| node.kind == NodeKind::File && node.file_index == Some(file_index))?;
    Some(FileTreeSelection {
        highlighted_node: node.id,
        expanded_nodes: file_tree_parent_directories(tree, node.id),
    })
}

/// Choose the single patch file index from fallbacks.
pub fn single_patch_file_index(
    selected: Option<usize>,
    active: Option<usize>,
    current: Option<usize>,
    first: Option<usize>,
) -> Option<usize> {
    selected.or(active).or(current).or(first)
}

/// The file indexes in flattened tree order.
pub fn ordered_patch_file_indexes(rows: &[FileTreeRow]) -> Vec<usize> {
    rows.iter().filter_map(|row| row.file_index).collect()
}

/// Whether the diff viewer file tree should be shown.
pub fn show_diff_viewer_file_tree(show_file_tree: bool, file_count: usize) -> bool {
    show_file_tree && file_count > 0
}

/// Move the patch selection through ordered file indexes.
pub fn move_patch_file_index(
    file_indexes: &[usize],
    current: Option<usize>,
    offset: i64,
) -> Option<usize> {
    if file_indexes.is_empty() {
        return None;
    }
    let index = current.and_then(|current| file_indexes.iter().position(|&item| item == current));
    let Some(index) = index else {
        return Some(file_indexes[0]);
    };
    let next = (index as i64 + offset).clamp(0, file_indexes.len() as i64 - 1) as usize;
    Some(file_indexes[next])
}

/// All directory ids in the tree.
pub fn all_expanded_file_tree_directories(tree: &FileTree) -> HashSet<usize> {
    tree.nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Directory)
        .map(|node| node.id)
        .collect()
}

/// Toggle a directory's expansion.
pub fn toggle_file_tree_directory(
    tree: &FileTree,
    expanded: &HashSet<usize>,
    selected: Option<usize>,
) -> HashSet<usize> {
    if selected.map(|id| tree.nodes[id].kind) != Some(NodeKind::Directory) {
        return expanded.clone();
    }
    let selected = selected.unwrap();
    let mut next = expanded.clone();
    if !next.remove(&selected) {
        next.insert(selected);
    }
    next
}

/// Set a directory's expansion state.
pub fn set_file_tree_directory_expanded(
    tree: &FileTree,
    expanded: &HashSet<usize>,
    selected: Option<usize>,
    value: bool,
) -> HashSet<usize> {
    if selected.map(|id| tree.nodes[id].kind) != Some(NodeKind::Directory) {
        return expanded.clone();
    }
    let selected = selected.unwrap();
    let mut next = expanded.clone();
    if value {
        next.insert(selected);
    } else {
        next.remove(&selected);
    }
    next
}

fn file_tree_parent_directories(tree: &FileTree, id: usize) -> HashSet<usize> {
    let mut result = HashSet::new();
    let mut parent = tree.nodes[id].parent;
    while let Some(current) = parent {
        result.insert(current);
        parent = tree.nodes[current].parent;
    }
    result
}
