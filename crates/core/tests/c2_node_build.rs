//! Port of packages/core/test/effect/layer-node/node-build.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: `AppNodeBuilder.build` and Effect `Layer`/`LayerMap` are replaced
//! by the real [`opencode_core::node_build`] builder. Observable contract
//! (location service map only built on demand, cycle detection, shared project
//! acquisition, composed application layer) is preserved.

#![allow(dead_code)]

use opencode_core::node_build::{build, group, make_global_node, BuiltLayer};

#[test]
fn does_not_build_a_location_service_map_when_the_graph_does_not_require_it() {
    let result = make_global_node("test/TagResult", vec![]).literal("plain");
    let built: BuiltLayer = build(&result, &[]).expect("build");
    assert!(!built.has_location_service_map);
    assert_eq!(built.resolve("test/TagResult").as_deref(), Some("plain"));
}

#[test]
fn detects_cycles_through_a_replaced_location_service_map() {
    let a = make_global_node(
        "test/NodeBuildA",
        vec![opencode_core::node_build::location_service_map_node()],
    );
    let b = make_global_node("test/NodeBuildB", vec![a.clone()]);
    let map = make_global_node("LocationServiceMap.Service", vec![b]);
    let err = build(
        &group(vec![a]),
        &[(opencode_core::node_build::location_service_map_node(), map)],
    )
    .expect_err("cycle");
    assert!(err.to_string().contains("Cycle detected in layer tree"));
}

#[test]
fn shares_top_level_project_with_location_services() {
    let built = build(
        &group(vec![
            opencode_core::node_build::project_node(),
            opencode_core::node_build::location_service_map_node(),
        ]),
        &[(
            opencode_core::node_build::project_node(),
            make_global_node("Project.Service", vec![]),
        )],
    )
    .expect("build");

    assert!(built.has_location_service_map);
    assert_eq!(built.project_acquisitions, 1);
    assert!(built.resolve("Location.Service").is_some());
}

#[test]
fn returns_a_composed_application_layer() {
    let value = make_global_node("test/TagValue", vec![]).literal("value");
    let result = make_global_node("test/TagResult", vec![value]);
    let built = build(&result, &[]).expect("build");
    assert!(!built.has_location_service_map);
    assert_eq!(built.resolve("test/TagResult").as_deref(), Some("value"));
}
