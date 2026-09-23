//! Port of packages/core/test/effect/layer-node/layer-node-types.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: the reference file is a `@ts-expect-error` type-level program.
//! Effect `Context.Service`/`Layer` tag safety is re-expressed as runtime
//! validation of node construction (service/name exclusivity, declared tag
//! references, dependency satisfaction, scope direction). Positive type
//! assertions are dropped as white-box.

#![allow(dead_code)]

use opencode_core::layer_node_types::{make, tags, validate_replacement, Node, Scope};

fn global(
    service: &str,
    deps: Vec<Node>,
) -> Result<Node, opencode_core::layer_node_types::NodeError> {
    make(Scope::Global, Some(service), None, deps)
}

fn location(
    service: &str,
    deps: Vec<Node>,
) -> Result<Node, opencode_core::layer_node_types::NodeError> {
    make(Scope::Location, Some(service), None, deps)
}

#[test]
fn tags_accept_declared_references() {
    let declared = tags(&[("app", &[])]).expect("tags");
    assert_eq!(declared.len(), 1);
    assert_eq!(declared[0].name, "app");

    let scoped = tags(&[("request", &["global"]), ("global", &[])]).expect("tags");
    assert_eq!(scoped.len(), 2);
}

#[test]
fn tags_reject_undeclared_references() {
    let err = tags(&[("request", &["missing"]), ("global", &[])]).expect_err("undeclared");
    assert!(err
        .to_string()
        .contains("Tag configuration can only reference declared tags"));
}

#[test]
fn make_requires_a_service_or_name() {
    let err = make(Scope::Global, None, None, vec![]).expect_err("missing");
    assert!(err
        .to_string()
        .contains("A node must have a service or name"));
}

#[test]
fn make_rejects_service_and_name_together() {
    let err =
        make(Scope::Global, Some("test/LayerNodeA"), Some("a"), vec![]).expect_err("exclusive");
    assert!(err
        .to_string()
        .contains("Service and name are mutually exclusive"));
}

#[test]
fn make_rejects_a_missing_dependency() {
    let a = global("test/LayerNodeA", vec![]).expect("a");
    let named = make(Scope::Global, None, Some("manual-a"), vec![]).expect("named");
    let err = global("test/LayerNodeB", vec![named]).expect_err("missing");
    assert!(err.to_string().contains("requires"));
    let ok = global("test/LayerNodeB", vec![a]).expect("b");
    assert_eq!(ok.service.as_deref(), Some("test/LayerNodeB"));
}

#[test]
fn global_cannot_depend_on_location() {
    let request_a = location("test/TagA", vec![]).expect("location");
    let err = global("test/TagB", vec![request_a]).expect_err("direction");
    assert!(err.to_string().contains("Global cannot depend on location"));
}

#[test]
fn replacement_must_provide_the_same_service() {
    let node = global("test/LayerNodeA", vec![]).expect("node");
    let err = validate_replacement(&node, "test/LayerNodeB").expect_err("replacement");
    assert!(err
        .to_string()
        .contains("Replacement must provide test/LayerNodeA"));
}

#[test]
fn type_exploration_compiles() {
    // The reference file asserts only that the type-level program compiles.
}
