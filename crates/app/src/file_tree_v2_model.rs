//! File tree model (port of packages/app/src/components/file-tree-v2-model.ts).

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct FileNode {
    pub path: String,
    pub node_type: String,
    pub original_path: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub node: FileNode,
    pub level: usize,
}

#[derive(Default)]
pub struct FileTreeModel {
    pub children: BTreeMap<String, Vec<FileNode>>,
    pub total: usize,
}

pub fn normalize_file_tree_v2_path(value: &str) -> String {
    let replaced = value.replace('\\', "/");
    let trimmed = replaced.trim_matches('/');
    let mut out = String::with_capacity(trimmed.len());
    let mut previous_slash = false;
    for character in trimmed.chars() {
        if character == '/' {
            if previous_slash {
                continue;
            }
            previous_slash = true;
        } else {
            previous_slash = false;
        }
        out.push(character);
    }
    out
}

fn name_of(path: &str) -> &str {
    match path.rfind('/') {
        Some(index) => &path[index + 1..],
        None => path,
    }
}

pub fn build_file_tree_v2_model(paths: &[&str]) -> FileTreeModel {
    let mut nodes: BTreeMap<String, FileNode> = BTreeMap::new();
    for value in paths {
        let file = normalize_file_tree_v2_path(value);
        if file.is_empty() {
            continue;
        }
        let parts: Vec<&str> = file.split('/').collect();
        for (index, _name) in parts.iter().enumerate() {
            let path = parts[..=index].join("/");
            if nodes.contains_key(&path) {
                continue;
            }
            let is_file = index == parts.len() - 1;
            nodes.insert(
                path.clone(),
                FileNode {
                    path: path.clone(),
                    node_type: if is_file { "file" } else { "directory" }.to_string(),
                    original_path: if is_file { value.to_string() } else { path },
                },
            );
        }
    }

    let mut children: BTreeMap<String, Vec<FileNode>> = BTreeMap::new();
    for node in nodes.values() {
        let parent = match node.path.rfind('/') {
            Some(index) => node.path[..index].to_string(),
            None => String::new(),
        };
        children.entry(parent).or_default().push(node.clone());
    }
    for list in children.values_mut() {
        list.sort_by(|a, b| {
            if a.node_type != b.node_type {
                if a.node_type == "directory" {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            } else {
                name_of(&a.path).cmp(name_of(&b.path))
            }
        });
    }

    FileTreeModel {
        total: nodes.len(),
        children,
    }
}

pub fn flatten_file_tree_v2(model: &FileTreeModel, expanded: impl Fn(&str) -> bool) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut stack: Vec<(FileNode, usize)> = model
        .children
        .get("")
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .rev()
        .map(|node| (node, 0))
        .collect();

    while let Some((node, level)) = stack.pop() {
        rows.push(Row {
            node: node.clone(),
            level,
        });
        if node.node_type != "directory" || !expanded(&node.path) {
            continue;
        }
        let nested = model.children.get(&node.path).cloned().unwrap_or_default();
        for child in nested.into_iter().rev() {
            stack.push((child, level + 1));
        }
    }
    rows
}

fn to_live_node(node: &FileNode) -> FileNode {
    FileNode {
        path: normalize_file_tree_v2_path(&node.path),
        node_type: node.node_type.clone(),
        original_path: node.path.clone(),
    }
}

pub fn flatten_live_file_tree_v2(
    children: impl Fn(&str) -> Vec<FileNode>,
    expanded: impl Fn(&str) -> bool,
) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut stack: Vec<(FileNode, usize)> = children("")
        .into_iter()
        .rev()
        .map(|node| (to_live_node(&node), 0))
        .collect();

    while let Some((node, level)) = stack.pop() {
        rows.push(Row {
            node: node.clone(),
            level,
        });
        if node.node_type != "directory" || !expanded(&node.path) {
            continue;
        }
        let nested = children(&node.original_path);
        for child in nested.into_iter().rev() {
            stack.push((to_live_node(&child), level + 1));
        }
    }
    rows
}
