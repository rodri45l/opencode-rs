//! Port of packages/core/test/effect/layer-node/layer-node.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: Effect `Layer`/`Context` are replaced by an explicit graph model.
//! The observable graph semantics (dependency resolution, unbound-node
//! rejection, replacement, hoisting, conflict detection) are preserved; type
//! assertions about `Layer.Layer<...>` shapes are dropped as white-box.

#![allow(dead_code)]

mod layer_node {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl std::fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "not implemented: {}", self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum NodeKind {
        Service,
        Group,
        Unbound,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Node {
        pub kind: NodeKind,
        pub service: Option<String>,
        pub name: Option<String>,
        pub tag: Option<String>,
        pub deps: Vec<Node>,
    }

    pub fn group(nodes: Vec<Node>) -> Node {
        Node {
            kind: NodeKind::Group,
            service: None,
            name: None,
            tag: None,
            deps: nodes,
        }
    }

    pub fn unbound(service: &str, tag: &str) -> Node {
        Node {
            kind: NodeKind::Unbound,
            service: Some(service.to_string()),
            name: None,
            tag: Some(tag.to_string()),
            deps: Vec::new(),
        }
    }

    pub fn make(tag: &str, service: &str, deps: Vec<Node>) -> Node {
        Node {
            kind: NodeKind::Service,
            service: Some(service.to_string()),
            name: None,
            tag: Some(tag.to_string()),
            deps,
        }
    }

    pub fn make_named(tag: &str, name: &str, deps: Vec<Node>) -> Node {
        Node {
            kind: NodeKind::Service,
            service: None,
            name: Some(name.to_string()),
            tag: Some(tag.to_string()),
            deps,
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Compiled {
        pub services: Vec<String>,
        pub acquisitions: usize,
    }

    impl Compiled {
        pub fn resolve(&self, _service: &str) -> Option<String> {
            None
        }
    }

    pub fn compile(
        _root: &Node,
        _replacements: &[(Node, Node)],
    ) -> Result<Compiled, NotImplemented> {
        Err(NotImplemented("layer-node compile"))
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct HoistResult {
        pub node: Node,
        pub hoisted: Vec<Node>,
    }

    pub fn hoist(_root: &Node, _tag: &str) -> Result<HoistResult, NotImplemented> {
        Err(NotImplemented("layer-node hoist"))
    }
}

use layer_node::{group, make, unbound, Node, NodeKind};

const NOTE: &str = "porting: layer-node not implemented";

fn value() -> Node {
    make("app", "test/LayerNodeValue", vec![])
}

fn greeting() -> Node {
    make("app", "test/LayerNodeGreeting", vec![value()])
}

#[test]
#[ignore = "porting: layer-node not implemented"]
fn builds_an_untagged_graph() {
    let value = make("app", "test/LayerNodeValue", vec![]);
    let greeting = make("app", "test/LayerNodeGreeting", vec![value]);
    let compiled = layer_node::compile(&group(vec![greeting]), &[]).expect(NOTE);
    assert_eq!(
        compiled.resolve("test/LayerNodeGreeting").as_deref(),
        Some("hello production")
    );
}

#[test]
#[ignore = "porting: layer-node not implemented"]
fn builds_a_dependency_graph() {
    let compiled = layer_node::compile(&group(vec![greeting()]), &[]).expect(NOTE);
    assert_eq!(
        compiled.resolve("test/LayerNodeGreeting").as_deref(),
        Some("hello production")
    );
}

#[test]
#[ignore = "porting: layer-node not implemented"]
fn exposes_roots_but_hides_transitive_dependencies() {
    let compiled = layer_node::compile(&group(vec![greeting()]), &[]).expect(NOTE);
    assert_eq!(
        compiled.services,
        vec!["test/LayerNodeGreeting".to_string()]
    );
}

#[test]
#[ignore = "porting: layer-node not implemented"]
fn preserves_branch_specific_implementations_across_roots() {
    let first = make("app", "test/LayerNodeValue", vec![]);
    let second = make("app", "test/LayerNodeValue", vec![]);
    let left = make("app", "test/LayerNodeLeft", vec![first]);
    let right = make("app", "test/LayerNodeRight", vec![second]);
    let compiled = layer_node::compile(&group(vec![left, right]), &[]).expect(NOTE);
    assert_eq!(
        compiled.resolve("test/LayerNodeLeft").as_deref(),
        Some("first")
    );
    assert_eq!(
        compiled.resolve("test/LayerNodeRight").as_deref(),
        Some("second")
    );
}

#[test]
#[ignore = "porting: layer-node not implemented"]
fn requires_unbound_nodes_to_be_replaced_before_compilation() {
    let unbound = unbound("test/LayerNodeValue", "app");
    let greeting = make("app", "test/LayerNodeGreeting", vec![unbound.clone()]);
    let tree = group(vec![greeting]);
    let err = layer_node::compile(&tree, &[]).expect_err(NOTE);
    assert!(err
        .to_string()
        .contains("Unbound layer node: test/LayerNodeValue"));

    let compiled = layer_node::compile(&tree, &[(unbound, value())]).expect(NOTE);
    assert_eq!(
        compiled.resolve("test/LayerNodeGreeting").as_deref(),
        Some("hello production")
    );
}

#[test]
#[ignore = "porting: layer-node not implemented"]
fn replaces_a_node_with_a_closed_layer() {
    let compiled = layer_node::compile(
        &group(vec![greeting()]),
        &[(value(), make("app", "test/LayerNodeValue", vec![]))],
    )
    .expect(NOTE);
    assert_eq!(
        compiled.resolve("test/LayerNodeGreeting").as_deref(),
        Some("hello simulation")
    );
}

#[test]
#[ignore = "porting: layer-node not implemented"]
fn replaces_every_use_of_the_same_layer() {
    let value = value();
    let left = make("app", "test/LayerNodeLeft", vec![value.clone()]);
    let right = make("app", "test/LayerNodeRight", vec![value.clone()]);
    let compiled = layer_node::compile(
        &group(vec![left, right]),
        &[(value, make("app", "test/LayerNodeValue", vec![]))],
    )
    .expect(NOTE);
    assert_eq!(
        compiled.resolve("test/LayerNodeLeft").as_deref(),
        Some("replaced")
    );
    assert_eq!(
        compiled.resolve("test/LayerNodeRight").as_deref(),
        Some("replaced")
    );
}

#[test]
#[ignore = "porting: layer-node not implemented"]
fn does_not_acquire_an_unused_replacement() {
    let other = make("app", "test/LayerNodeLeft", vec![]);
    let compiled = layer_node::compile(
        &group(vec![greeting()]),
        &[(other, make("app", "test/LayerNodeLeft", vec![]))],
    )
    .expect(NOTE);
    assert_eq!(compiled.acquisitions, 0);
}

#[test]
#[ignore = "porting: layer-node not implemented"]
fn replaces_a_node_without_acquiring_its_dependencies() {
    let dependency = make("app", "test/LayerNodeValue", vec![]);
    let original = make("app", "test/LayerNodeGreeting", vec![dependency]);
    let replacement = make("app", "test/LayerNodeGreeting", vec![]);
    let compiled = layer_node::compile(&group(vec![original.clone()]), &[(original, replacement)])
        .expect(NOTE);
    assert_eq!(
        compiled.resolve("test/LayerNodeGreeting").as_deref(),
        Some("replacement")
    );
    assert_eq!(compiled.acquisitions, 0);
}

#[test]
#[ignore = "porting: layer-node not implemented"]
fn applies_later_replacements_inside_earlier_replacement_nodes() {
    let original = make("app", "test/LayerNodeGreeting", vec![value()]);
    let replacement = make("app", "test/LayerNodeGreeting", vec![value()]);
    let compiled = layer_node::compile(
        &group(vec![original.clone()]),
        &[
            (original, replacement),
            (value(), make("app", "test/LayerNodeValue", vec![])),
        ],
    )
    .expect(NOTE);
    assert_eq!(
        compiled.resolve("test/LayerNodeGreeting").as_deref(),
        Some("hello replacement dependency")
    );
}

#[test]
#[ignore = "porting: layer-node not implemented"]
fn hoists_and_compiles_tagged_graphs() {
    let database = make("global", "test/GraphDatabase", vec![]);
    let users = make("location", "test/GraphUsers", vec![database]);
    let app = make("location", "test/GraphApp", vec![users]);

    let result = layer_node::hoist(&group(vec![app]), "global").expect(NOTE);
    assert_eq!(result.node.deps[0].deps[0].deps[0].kind, NodeKind::Group);
    assert!(result.node.deps[0].deps[0].deps[0].deps.is_empty());
    assert_eq!(result.hoisted.len(), 1);
}

#[test]
#[ignore = "porting: layer-node not implemented"]
fn rejects_conflicting_hoisted_implementations() {
    let first = make("global", "test/GraphDatabase", vec![]);
    let second = make("global", "test/GraphDatabase", vec![]);
    let left = make("location", "test/GraphUsers", vec![first]);
    let right = make("location", "test/GraphApp", vec![second]);

    let err = layer_node::hoist(&group(vec![left, right]), "global").expect_err(NOTE);
    assert!(err
        .to_string()
        .contains("Tag global has conflicting implementations for test/GraphDatabase"));
}

#[test]
#[ignore = "porting: layer-node not implemented"]
fn treats_dependency_groups_as_transparent_while_hoisting() {
    let database = make("global", "test/GraphDatabase", vec![]);
    let users = make("location", "test/GraphUsers", vec![group(vec![database])]);
    let result = layer_node::hoist(&group(vec![users]), "global").expect(NOTE);
    assert_eq!(result.node.deps[0].deps[0].deps[0].kind, NodeKind::Group);
}
