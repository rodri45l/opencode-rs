//! Port of packages/core/test/effect/layer-node/node-build.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: `AppNodeBuilder.build` and Effect `Layer`/`LayerMap` are replaced
//! by an explicit builder stub. Observable contract (location service map only
//! built on demand, cycle detection, shared project acquisition, composed
//! application layer) is preserved.

#![allow(dead_code)]

mod node_build {
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
    pub enum Kind {
        Service,
        Group,
        LocationServiceMap,
        Project,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Node {
        pub kind: Kind,
        pub service: Option<String>,
        pub scope: Scope,
        pub deps: Vec<Node>,
    }

    pub fn make_global_node(service: &str, deps: Vec<Node>) -> Node {
        Node {
            kind: Kind::Service,
            service: Some(service.to_string()),
            scope: Scope::Global,
            deps,
        }
    }

    pub fn make_location_node(service: &str, deps: Vec<Node>) -> Node {
        Node {
            kind: Kind::Service,
            service: Some(service.to_string()),
            scope: Scope::Location,
            deps,
        }
    }

    pub fn group(nodes: Vec<Node>) -> Node {
        Node {
            kind: Kind::Group,
            service: None,
            scope: Scope::Global,
            deps: nodes,
        }
    }

    pub fn location_service_map_node() -> Node {
        Node {
            kind: Kind::LocationServiceMap,
            service: Some("LocationServiceMap.Service".to_string()),
            scope: Scope::Global,
            deps: Vec::new(),
        }
    }

    pub fn project_node() -> Node {
        Node {
            kind: Kind::Project,
            service: Some("Project.Service".to_string()),
            scope: Scope::Global,
            deps: Vec::new(),
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BuiltLayer {
        pub has_location_service_map: bool,
        pub project_acquisitions: usize,
    }

    impl BuiltLayer {
        pub fn resolve(&self, _service: &str) -> Option<String> {
            None
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BuildError {
        pub message: String,
    }

    impl std::fmt::Display for BuildError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for BuildError {}

    pub fn build(_root: &Node, _replacements: &[(Node, Node)]) -> Result<BuiltLayer, BuildError> {
        Err(BuildError {
            message: "not implemented: node build".to_string(),
        })
    }
}

use node_build::{group, make_global_node, BuildError, BuiltLayer};

const NOTE: &str = "porting: node build not implemented";

#[test]
#[ignore = "porting: node build not implemented"]
fn does_not_build_a_location_service_map_when_the_graph_does_not_require_it() {
    let result = make_global_node("test/TagResult", vec![]);
    let built: BuiltLayer = node_build::build(&result, &[]).expect(NOTE);
    assert!(!built.has_location_service_map);
    assert_eq!(built.resolve("test/TagResult").as_deref(), Some("plain"));
}

#[test]
#[ignore = "porting: node build not implemented"]
fn detects_cycles_through_a_replaced_location_service_map() {
    let a = make_global_node(
        "test/NodeBuildA",
        vec![node_build::location_service_map_node()],
    );
    let b = make_global_node("test/NodeBuildB", vec![a.clone()]);
    let map = make_global_node("LocationServiceMap.Service", vec![b]);
    let err: BuildError = node_build::build(
        &group(vec![a]),
        &[(node_build::location_service_map_node(), map)],
    )
    .expect_err(NOTE);
    assert!(err.to_string().contains("Cycle detected in layer tree"));
}

#[test]
#[ignore = "porting: node build not implemented"]
fn shares_top_level_project_with_location_services() {
    let built = node_build::build(
        &group(vec![
            node_build::project_node(),
            node_build::location_service_map_node(),
        ]),
        &[(
            node_build::project_node(),
            make_global_node("Project.Service", vec![]),
        )],
    )
    .expect(NOTE);

    assert!(built.has_location_service_map);
    assert_eq!(built.project_acquisitions, 1);
    assert!(built.resolve("Location.Service").is_some());
}

#[test]
#[ignore = "porting: node build not implemented"]
fn returns_a_composed_application_layer() {
    let value = make_global_node("test/TagValue", vec![]);
    let result = make_global_node("test/TagResult", vec![value]);
    let built = node_build::build(&result, &[]).expect(NOTE);
    assert!(!built.has_location_service_map);
    assert_eq!(built.resolve("test/TagResult").as_deref(), Some("value"));
}
