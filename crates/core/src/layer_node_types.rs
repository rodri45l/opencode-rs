//! Layer-node type validation.
//!
//! Ports the runtime-checkable parts of
//! `packages/core/test/effect/layer-node/layer-node-types.test.ts`: declared tag
//! references, service/name exclusivity, dependency scope direction, and
//! replacement service identity. The reference is a type-level
//! (`@ts-expect-error`) program; positive type assertions are dropped as
//! white-box and re-expressed as runtime validation.

/// The scope a node is declared in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Application-wide.
    Global,
    /// Request/location-scoped.
    Location,
}

/// A declared tag and the tags it may reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagConfig {
    /// Tag name.
    pub name: String,
    /// Declared tags this tag depends on.
    pub deps: Vec<String>,
}

/// A node in a layer graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// The provided service, if any.
    pub service: Option<String>,
    /// The manual node name, if any.
    pub name: Option<String>,
    /// The node's scope.
    pub scope: Scope,
    /// Dependencies.
    pub deps: Vec<Node>,
}

/// Error raised by layer-node validation.
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

/// Declare the tag scopes; each tag may only reference declared tags.
pub fn tags(specs: &[(&str, &[&str])]) -> Result<Vec<TagConfig>, NodeError> {
    let declared: Vec<&str> = specs.iter().map(|(name, _)| *name).collect();
    let mut out = Vec::new();
    for (name, deps) in specs {
        for dep in *deps {
            if !declared.contains(dep) {
                return Err(NodeError::new(
                    "Tag configuration can only reference declared tags",
                ));
            }
        }
        out.push(TagConfig {
            name: (*name).to_string(),
            deps: deps.iter().map(|dep| (*dep).to_string()).collect(),
        });
    }
    Ok(out)
}

/// Build a node, rejecting invalid service/name/dependency combinations.
pub fn make(
    scope: Scope,
    service: Option<&str>,
    name: Option<&str>,
    deps: Vec<Node>,
) -> Result<Node, NodeError> {
    if service.is_none() && name.is_none() {
        return Err(NodeError::new("A node must have a service or name"));
    }
    if service.is_some() && name.is_some() {
        return Err(NodeError::new("Service and name are mutually exclusive"));
    }
    for dep in &deps {
        if dep.service.is_none() {
            return Err(NodeError::new(
                "A dependency requires a service, not a name",
            ));
        }
        if scope == Scope::Global && dep.scope == Scope::Location {
            return Err(NodeError::new("Global cannot depend on location"));
        }
    }
    Ok(Node {
        service: service.map(str::to_string),
        name: name.map(str::to_string),
        scope,
        deps,
    })
}

/// Validate a replacement layer for a node.
pub fn validate_replacement(node: &Node, replacement_service: &str) -> Result<(), NodeError> {
    match node.service.as_deref() {
        Some(service) if service == replacement_service => Ok(()),
        Some(service) => Err(NodeError::new(format!(
            "Replacement must provide {service}"
        ))),
        None => Err(NodeError::new("Replacement must provide a service")),
    }
}
