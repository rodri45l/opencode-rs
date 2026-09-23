//! Layer node graph compilation.
//!
//! Ports the observable graph semantics of
//! `packages/core/test/effect/layer-node/layer-node.test.ts` without the Effect
//! runtime: dependency resolution, unbound-node rejection, node replacement
//! (including later replacements inside earlier ones), root exposure, and tagged
//! graph hoisting with conflict detection.

use std::collections::BTreeMap;

/// Error raised while compiling or hoisting a graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeError {
    /// Human-readable message.
    pub message: String,
}

impl NodeError {
    fn new(message: impl Into<String>) -> Self {
        NodeError {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for NodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for NodeError {}

/// The kind of a layer node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// A concrete service.
    Service,
    /// A grouping of nodes.
    Group,
    /// A placeholder that must be replaced before compilation.
    Unbound,
}

/// How a node computes its resolved value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueExpr {
    /// A literal value.
    Literal(String),
    /// Prefix `p` applied to the first dependency's value.
    Prefix(String),
    /// The first dependency's value (or the empty string).
    Identity,
    /// A literal value that counts as an acquisition when compiled.
    Effect(String),
}

/// A node in a layer graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// The node kind.
    pub kind: NodeKind,
    /// The provided service, if any.
    pub service: Option<String>,
    /// The manual name, if any.
    pub name: Option<String>,
    /// The declared tag, if any.
    pub tag: Option<String>,
    /// Dependencies.
    pub deps: Vec<Node>,
    /// The value expression.
    pub expr: ValueExpr,
}

/// Group `nodes` into a single node.
pub fn group(nodes: Vec<Node>) -> Node {
    Node {
        kind: NodeKind::Group,
        service: None,
        name: None,
        tag: None,
        deps: nodes,
        expr: ValueExpr::Identity,
    }
}

/// An unbound placeholder for `service` in `tag`.
pub fn unbound(service: &str, tag: &str) -> Node {
    Node {
        kind: NodeKind::Unbound,
        service: Some(service.to_string()),
        name: None,
        tag: Some(tag.to_string()),
        deps: Vec::new(),
        expr: ValueExpr::Identity,
    }
}

/// A service node for `service` in `tag`.
pub fn make(tag: &str, service: &str, deps: Vec<Node>) -> Node {
    Node {
        kind: NodeKind::Service,
        service: Some(service.to_string()),
        name: None,
        tag: Some(tag.to_string()),
        deps,
        expr: ValueExpr::Identity,
    }
}

/// A named node in `tag`.
pub fn make_named(tag: &str, name: &str, deps: Vec<Node>) -> Node {
    Node {
        kind: NodeKind::Service,
        service: None,
        name: Some(name.to_string()),
        tag: Some(tag.to_string()),
        deps,
        expr: ValueExpr::Identity,
    }
}

impl Node {
    /// Set a literal value expression.
    pub fn literal(mut self, value: &str) -> Self {
        self.expr = ValueExpr::Literal(value.to_string());
        self
    }

    /// Set a prefix value expression.
    pub fn prefix(mut self, value: &str) -> Self {
        self.expr = ValueExpr::Prefix(value.to_string());
        self
    }

    /// Set an effect value expression.
    pub fn effect(mut self, value: &str) -> Self {
        self.expr = ValueExpr::Effect(value.to_string());
        self
    }
}

/// A compiled graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compiled {
    /// The root services exposed by the graph.
    pub services: Vec<String>,
    /// The number of effect nodes in the compiled graph.
    pub acquisitions: usize,
    values: BTreeMap<String, String>,
}

impl Compiled {
    /// Resolve the value provided for `service`.
    pub fn resolve(&self, service: &str) -> Option<String> {
        self.values.get(service).cloned()
    }
}

/// Compile `root`, applying `replacements` in order.
pub fn compile(root: &Node, replacements: &[(Node, Node)]) -> Result<Compiled, NodeError> {
    let mut tree = root.clone();
    for (from, to) in replacements {
        tree = replace_all(&tree, from, to);
    }
    if let Some(service) = find_unbound(&tree) {
        return Err(NodeError::new(format!("Unbound layer node: {service}")));
    }
    let services = root_services(&tree);
    let mut values = BTreeMap::new();
    let mut acquisitions = 0;
    collect(&tree, &mut values, &mut acquisitions);
    Ok(Compiled {
        services,
        acquisitions,
        values,
    })
}

fn replace_all(node: &Node, from: &Node, to: &Node) -> Node {
    if node == from {
        return to.clone();
    }
    let mut replaced = node.clone();
    replaced.deps = node
        .deps
        .iter()
        .map(|dep| replace_all(dep, from, to))
        .collect();
    replaced
}

fn find_unbound(node: &Node) -> Option<String> {
    if node.kind == NodeKind::Unbound {
        return node.service.clone();
    }
    node.deps.iter().find_map(find_unbound)
}

fn root_services(root: &Node) -> Vec<String> {
    if root.kind == NodeKind::Group {
        root.deps
            .iter()
            .filter_map(|dep| dep.service.clone())
            .collect()
    } else {
        root.service.clone().into_iter().collect()
    }
}

fn collect(node: &Node, values: &mut BTreeMap<String, String>, acquisitions: &mut usize) {
    if let Some(service) = &node.service {
        values.insert(service.clone(), evaluate(node, acquisitions));
    }
    for dep in &node.deps {
        collect(dep, values, acquisitions);
    }
}

fn evaluate(node: &Node, acquisitions: &mut usize) -> String {
    match &node.expr {
        ValueExpr::Literal(value) => value.clone(),
        ValueExpr::Effect(value) => {
            *acquisitions += 1;
            value.clone()
        }
        ValueExpr::Prefix(prefix) => match node.deps.first() {
            Some(dep) => format!("{prefix}{}", evaluate(dep, acquisitions)),
            None => prefix.clone(),
        },
        ValueExpr::Identity => match node.deps.first() {
            Some(dep) => evaluate(dep, acquisitions),
            None => String::new(),
        },
    }
}

/// The result of hoisting tagged nodes out of a graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoistResult {
    /// The graph with hoisted nodes replaced by empty groups.
    pub node: Node,
    /// The hoisted nodes.
    pub hoisted: Vec<Node>,
}

/// Hoist nodes tagged `tag` to the top, rejecting conflicting implementations.
pub fn hoist(root: &Node, tag: &str) -> Result<HoistResult, NodeError> {
    let mut hoisted: Vec<Node> = Vec::new();
    let node = hoist_node(root, tag, &mut hoisted)?;
    Ok(HoistResult { node, hoisted })
}

fn hoist_node(node: &Node, tag: &str, hoisted: &mut Vec<Node>) -> Result<Node, NodeError> {
    if node.tag.as_deref() == Some(tag) && node.kind == NodeKind::Service {
        if let Some(existing) = hoisted.iter().find(|item| item.service == node.service) {
            if existing != node {
                let service = node.service.clone().unwrap_or_default();
                return Err(NodeError::new(format!(
                    "Tag {tag} has conflicting implementations for {service}"
                )));
            }
        } else {
            hoisted.push(node.clone());
        }
        return Ok(group(Vec::new()));
    }
    let mut hoisted_node = node.clone();
    let mut deps = Vec::new();
    for dep in &node.deps {
        deps.push(hoist_node(dep, tag, hoisted)?);
    }
    hoisted_node.deps = deps;
    Ok(hoisted_node)
}
