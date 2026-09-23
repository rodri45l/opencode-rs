//! Port of packages/core/test/effect/layer-node/layer-node-types.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: the reference file is a `@ts-expect-error` type-level program.
//! Effect `Context.Service`/`Layer` tag safety is re-expressed as runtime
//! validation of node construction (service/name exclusivity, declared tag
//! references, dependency satisfaction, scope direction). Positive type
//! assertions are dropped as white-box.

#![allow(dead_code)]

mod layer_node_types {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl std::fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "not implemented: {}", self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Scope {
        Global,
        Location,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TagConfig {
        pub name: String,
        pub deps: Vec<String>,
    }

    /// Declare the tag scopes; each tag may only reference declared tags.
    pub fn tags(specs: &[(&str, &[&str])]) -> Result<Vec<TagConfig>, NotImplemented> {
        let _ = specs;
        Err(NotImplemented("layer-node tags"))
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Node {
        pub service: Option<String>,
        pub name: Option<String>,
        pub scope: Scope,
        pub deps: Vec<Node>,
    }

    /// Build a node, rejecting invalid service/name/dependency combinations.
    pub fn make(
        scope: Scope,
        service: Option<&str>,
        name: Option<&str>,
        deps: Vec<Node>,
    ) -> Result<Node, NotImplemented> {
        let _ = (scope, service, name, deps);
        Err(NotImplemented("layer-node make"))
    }

    /// Validate a replacement layer for a node.
    pub fn validate_replacement(
        _node: &Node,
        _replacement_service: &str,
    ) -> Result<(), NotImplemented> {
        Err(NotImplemented("layer-node replacement"))
    }
}

use layer_node_types::{make, tags, Scope};

const NOTE: &str = "porting: layer-node types not implemented";

fn global(
    service: &str,
    deps: Vec<layer_node_types::Node>,
) -> Result<layer_node_types::Node, layer_node_types::NotImplemented> {
    make(Scope::Global, Some(service), None, deps)
}

fn location(
    service: &str,
    deps: Vec<layer_node_types::Node>,
) -> Result<layer_node_types::Node, layer_node_types::NotImplemented> {
    make(Scope::Location, Some(service), None, deps)
}

#[test]
#[ignore = "porting: layer-node types not implemented"]
fn tags_accept_declared_references() {
    let declared = tags(&[("app", &[])]).expect(NOTE);
    assert_eq!(declared.len(), 1);
    assert_eq!(declared[0].name, "app");

    let scoped = tags(&[("request", &["global"]), ("global", &[])]).expect(NOTE);
    assert_eq!(scoped.len(), 2);
}

#[test]
#[ignore = "porting: layer-node types not implemented"]
fn tags_reject_undeclared_references() {
    let err = tags(&[("request", &["missing"]), ("global", &[])]).expect_err(NOTE);
    assert!(err
        .to_string()
        .contains("Tag configuration can only reference declared tags"));
}

#[test]
#[ignore = "porting: layer-node types not implemented"]
fn make_requires_a_service_or_name() {
    let err = make(Scope::Global, None, None, vec![]).expect_err(NOTE);
    assert!(err
        .to_string()
        .contains("A node must have a service or name"));
}

#[test]
#[ignore = "porting: layer-node types not implemented"]
fn make_rejects_service_and_name_together() {
    let err = make(Scope::Global, Some("test/LayerNodeA"), Some("a"), vec![]).expect_err(NOTE);
    assert!(err
        .to_string()
        .contains("Service and name are mutually exclusive"));
}

#[test]
#[ignore = "porting: layer-node types not implemented"]
fn make_rejects_a_missing_dependency() {
    let a = global("test/LayerNodeA", vec![]).expect(NOTE);
    let err = global("test/LayerNodeB", vec![]).expect_err(NOTE);
    assert!(err.to_string().contains("requires"));
    let ok = global("test/LayerNodeB", vec![a]).expect(NOTE);
    assert_eq!(ok.service.as_deref(), Some("test/LayerNodeB"));
}

#[test]
#[ignore = "porting: layer-node types not implemented"]
fn global_cannot_depend_on_location() {
    let request_a = location("test/TagA", vec![]).expect(NOTE);
    let err = global("test/TagB", vec![request_a]).expect_err(NOTE);
    assert!(err.to_string().contains("Global cannot depend on location"));
}

#[test]
#[ignore = "porting: layer-node types not implemented"]
fn replacement_must_provide_the_same_service() {
    let node = global("test/LayerNodeA", vec![]).expect(NOTE);
    let err = layer_node_types::validate_replacement(&node, "test/LayerNodeB").expect_err(NOTE);
    assert!(err
        .to_string()
        .contains("Replacement must provide test/LayerNodeA"));
}

#[test]
fn type_exploration_compiles() {
    // The reference file asserts only that the type-level program compiles.
}
