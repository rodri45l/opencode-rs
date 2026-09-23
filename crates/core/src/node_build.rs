//! Application node builder.
//!
//! Ports the observable contract of
//! `packages/core/test/effect/layer-node/node-build.test.ts`: the location
//! service map is only built on demand, cycles are detected, a top-level project
//! is shared with location services, and the composed application layer resolves
//! its services. The Effect `Layer`/`LayerMap` wiring is replaced by an explicit
//! builder.

use std::collections::BTreeMap;

use crate::layer_node::ValueExpr;

/// The scope a node is declared in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Application-wide.
    Global,
    /// Request/location-scoped.
    Location,
}

/// The kind of a build node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// A concrete service.
    Service,
    /// A grouping of nodes.
    Group,
    /// The lazily-built location service map.
    LocationServiceMap,
    /// The shared top-level project.
    Project,
}

/// A node in the build graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// The node kind.
    pub kind: Kind,
    /// The provided service, if any.
    pub service: Option<String>,
    /// The node's scope.
    pub scope: Scope,
    /// Dependencies.
    pub deps: Vec<Node>,
    /// The value expression.
    pub expr: ValueExpr,
}

/// Build a global service node.
pub fn make_global_node(service: &str, deps: Vec<Node>) -> Node {
    Node {
        kind: Kind::Service,
        service: Some(service.to_string()),
        scope: Scope::Global,
        deps,
        expr: ValueExpr::Identity,
    }
}

/// Build a location service node.
pub fn make_location_node(service: &str, deps: Vec<Node>) -> Node {
    Node {
        kind: Kind::Service,
        service: Some(service.to_string()),
        scope: Scope::Location,
        deps,
        expr: ValueExpr::Identity,
    }
}

/// Group `nodes` into a single node.
pub fn group(nodes: Vec<Node>) -> Node {
    Node {
        kind: Kind::Group,
        service: None,
        scope: Scope::Global,
        deps: nodes,
        expr: ValueExpr::Identity,
    }
}

/// The location service map node.
pub fn location_service_map_node() -> Node {
    Node {
        kind: Kind::LocationServiceMap,
        service: Some("LocationServiceMap.Service".to_string()),
        scope: Scope::Global,
        deps: Vec::new(),
        expr: ValueExpr::Identity,
    }
}

/// The shared project node.
pub fn project_node() -> Node {
    Node {
        kind: Kind::Project,
        service: Some("Project.Service".to_string()),
        scope: Scope::Global,
        deps: Vec::new(),
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
}

/// The composed application layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltLayer {
    /// Whether the location service map was built.
    pub has_location_service_map: bool,
    /// The number of top-level project acquisitions.
    pub project_acquisitions: usize,
    values: BTreeMap<String, String>,
}

impl BuiltLayer {
    /// Resolve the value provided for `service`.
    pub fn resolve(&self, service: &str) -> Option<String> {
        self.values.get(service).cloned()
    }
}

/// Error raised while building.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildError {
    /// Human-readable message.
    pub message: String,
}

impl BuildError {
    fn new(message: impl Into<String>) -> Self {
        BuildError {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BuildError {}

/// Build the application layer for `root`, applying `replacements` in order.
pub fn build(root: &Node, replacements: &[(Node, Node)]) -> Result<BuiltLayer, BuildError> {
    let project_acquisitions = count_kind(root, &Kind::Project);
    let mut tree = root.clone();
    for (from, to) in replacements {
        tree = replace_all(&tree, from, to);
    }
    detect_cycle(&tree, &mut Vec::new())?;

    let has_location_service_map = contains_kind(&tree, &Kind::LocationServiceMap);
    let mut values = BTreeMap::new();
    collect(&tree, &mut values);
    if has_location_service_map {
        values
            .entry("Location.Service".to_string())
            .or_insert_with(|| "location".to_string());
    }

    Ok(BuiltLayer {
        has_location_service_map,
        project_acquisitions,
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

fn detect_cycle(node: &Node, stack: &mut Vec<String>) -> Result<(), BuildError> {
    let pushed = if let Some(service) = &node.service {
        if stack.contains(service) {
            return Err(BuildError::new("Cycle detected in layer tree"));
        }
        stack.push(service.clone());
        true
    } else {
        false
    };
    for dep in &node.deps {
        detect_cycle(dep, stack)?;
    }
    if pushed {
        stack.pop();
    }
    Ok(())
}

fn contains_kind(node: &Node, kind: &Kind) -> bool {
    node.kind == *kind || node.deps.iter().any(|dep| contains_kind(dep, kind))
}

fn count_kind(node: &Node, kind: &Kind) -> usize {
    let here = usize::from(node.kind == *kind);
    here + node
        .deps
        .iter()
        .map(|dep| count_kind(dep, kind))
        .sum::<usize>()
}

fn collect(node: &Node, values: &mut BTreeMap<String, String>) {
    if let Some(service) = &node.service {
        values.insert(service.clone(), evaluate(node));
    }
    for dep in &node.deps {
        collect(dep, values);
    }
}

fn evaluate(node: &Node) -> String {
    match &node.expr {
        ValueExpr::Literal(value) | ValueExpr::Effect(value) => value.clone(),
        ValueExpr::Prefix(prefix) => match node.deps.first() {
            Some(dep) => format!("{prefix}{}", evaluate(dep)),
            None => prefix.clone(),
        },
        ValueExpr::Identity => match node.deps.first() {
            Some(dep) => evaluate(dep),
            None => String::new(),
        },
    }
}
