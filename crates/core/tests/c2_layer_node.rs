//! Port of packages/core/test/effect/layer-node/layer-node.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: Effect `Layer`/`Context` are replaced by an explicit graph model
//! with value expressions. The observable graph semantics (dependency
//! resolution, unbound-node rejection, replacement, hoisting, conflict
//! detection) are preserved; type assertions about `Layer.Layer<...>` shapes
//! are dropped as white-box.

#![allow(dead_code)]

use opencode_core::layer_node::{compile, group, hoist, make, unbound, Node, NodeKind};

fn value() -> Node {
    make("app", "test/LayerNodeValue", vec![]).literal("production")
}

fn greeting() -> Node {
    make("app", "test/LayerNodeGreeting", vec![value()]).prefix("hello ")
}

#[test]
fn builds_an_untagged_graph() {
    let value = make("app", "test/LayerNodeValue", vec![]).literal("production");
    let greeting = make("app", "test/LayerNodeGreeting", vec![value]).prefix("hello ");
    let compiled = compile(&group(vec![greeting]), &[]).expect("compile");
    assert_eq!(
        compiled.resolve("test/LayerNodeGreeting").as_deref(),
        Some("hello production")
    );
}

#[test]
fn builds_a_dependency_graph() {
    let compiled = compile(&group(vec![greeting()]), &[]).expect("compile");
    assert_eq!(
        compiled.resolve("test/LayerNodeGreeting").as_deref(),
        Some("hello production")
    );
}

#[test]
fn exposes_roots_but_hides_transitive_dependencies() {
    let compiled = compile(&group(vec![greeting()]), &[]).expect("compile");
    assert_eq!(
        compiled.services,
        vec!["test/LayerNodeGreeting".to_string()]
    );
}

#[test]
fn preserves_branch_specific_implementations_across_roots() {
    let first = make("app", "test/LayerNodeValue", vec![]).literal("first");
    let second = make("app", "test/LayerNodeValue", vec![]).literal("second");
    let left = make("app", "test/LayerNodeLeft", vec![first]);
    let right = make("app", "test/LayerNodeRight", vec![second]);
    let compiled = compile(&group(vec![left, right]), &[]).expect("compile");
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
fn requires_unbound_nodes_to_be_replaced_before_compilation() {
    let unbound = unbound("test/LayerNodeValue", "app");
    let greeting = make("app", "test/LayerNodeGreeting", vec![unbound.clone()]).prefix("hello ");
    let tree = group(vec![greeting]);
    let err = compile(&tree, &[]).expect_err("unbound");
    assert!(err
        .to_string()
        .contains("Unbound layer node: test/LayerNodeValue"));

    let compiled = compile(&tree, &[(unbound, value())]).expect("compile");
    assert_eq!(
        compiled.resolve("test/LayerNodeGreeting").as_deref(),
        Some("hello production")
    );
}

#[test]
fn replaces_a_node_with_a_closed_layer() {
    let compiled = compile(
        &group(vec![greeting()]),
        &[(
            value(),
            make("app", "test/LayerNodeValue", vec![]).literal("simulation"),
        )],
    )
    .expect("compile");
    assert_eq!(
        compiled.resolve("test/LayerNodeGreeting").as_deref(),
        Some("hello simulation")
    );
}

#[test]
fn replaces_every_use_of_the_same_layer() {
    let value = value();
    let left = make("app", "test/LayerNodeLeft", vec![value.clone()]);
    let right = make("app", "test/LayerNodeRight", vec![value.clone()]);
    let compiled = compile(
        &group(vec![left, right]),
        &[(
            value,
            make("app", "test/LayerNodeValue", vec![]).literal("replaced"),
        )],
    )
    .expect("compile");
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
fn does_not_acquire_an_unused_replacement() {
    let other = make("app", "test/LayerNodeLeft", vec![]);
    let compiled = compile(
        &group(vec![greeting()]),
        &[(
            other,
            make("app", "test/LayerNodeLeft", vec![]).effect("replacement"),
        )],
    )
    .expect("compile");
    assert_eq!(compiled.acquisitions, 0);
}

#[test]
fn replaces_a_node_without_acquiring_its_dependencies() {
    let dependency = make("app", "test/LayerNodeValue", vec![]).effect("dependency");
    let original = make("app", "test/LayerNodeGreeting", vec![dependency]).prefix("hello ");
    let replacement = make("app", "test/LayerNodeGreeting", vec![]).literal("replacement");
    let compiled =
        compile(&group(vec![original.clone()]), &[(original, replacement)]).expect("compile");
    assert_eq!(
        compiled.resolve("test/LayerNodeGreeting").as_deref(),
        Some("replacement")
    );
    assert_eq!(compiled.acquisitions, 0);
}

#[test]
fn applies_later_replacements_inside_earlier_replacement_nodes() {
    let original = make("app", "test/LayerNodeGreeting", vec![value()]).prefix("hello ");
    let replacement = make("app", "test/LayerNodeGreeting", vec![value()]).prefix("hello ");
    let compiled = compile(
        &group(vec![original.clone()]),
        &[
            (original, replacement),
            (
                value(),
                make("app", "test/LayerNodeValue", vec![]).literal("replacement dependency"),
            ),
        ],
    )
    .expect("compile");
    assert_eq!(
        compiled.resolve("test/LayerNodeGreeting").as_deref(),
        Some("hello replacement dependency")
    );
}

#[test]
fn hoists_and_compiles_tagged_graphs() {
    let database = make("global", "test/GraphDatabase", vec![]);
    let users = make("location", "test/GraphUsers", vec![database]);
    let app = make("location", "test/GraphApp", vec![users]);

    let result = hoist(&group(vec![app]), "global").expect("hoist");
    assert_eq!(result.node.deps[0].deps[0].deps[0].kind, NodeKind::Group);
    assert!(result.node.deps[0].deps[0].deps[0].deps.is_empty());
    assert_eq!(result.hoisted.len(), 1);
}

#[test]
fn rejects_conflicting_hoisted_implementations() {
    let first = make("global", "test/GraphDatabase", vec![]).literal("first");
    let second = make("global", "test/GraphDatabase", vec![]).literal("second");
    let left = make("location", "test/GraphUsers", vec![first]);
    let right = make("location", "test/GraphApp", vec![second]);

    let err = hoist(&group(vec![left, right]), "global").expect_err("conflict");
    assert!(err
        .to_string()
        .contains("Tag global has conflicting implementations for test/GraphDatabase"));
}

#[test]
fn treats_dependency_groups_as_transparent_while_hoisting() {
    let database = make("global", "test/GraphDatabase", vec![]);
    let users = make("location", "test/GraphUsers", vec![group(vec![database])]);
    let result = hoist(&group(vec![users]), "global").expect("hoist");
    assert_eq!(result.node.deps[0].deps[0].deps[0].kind, NodeKind::Group);
}
