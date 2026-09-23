//! Contract tests for the route inventory.
//!
//! Every route the Rust server claims to implement must exist in the reference
//! `openapi.json` with the same method, path, and `operationId`.

use opencode_protocol::IMPLEMENTED_ROUTES;
use serde_json::Value;

fn fixture() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/routes.json"
    );
    let raw = std::fs::read_to_string(path).expect("read routes fixture");
    serde_json::from_str(&raw).expect("parse routes fixture")
}

#[test]
fn implemented_routes_exist_in_the_contract() {
    let fx = fixture();
    let routes = fx["routes"].as_array().unwrap();
    for route in IMPLEMENTED_ROUTES {
        let found = routes.iter().any(|r| {
            r["method"].as_str() == Some(route.method)
                && r["path"].as_str() == Some(route.path)
                && r["operation_id"].as_str() == Some(route.operation_id)
        });
        assert!(
            found,
            "route {} {} ({}) is not in the reference contract",
            route.method, route.path, route.operation_id
        );
    }
}

#[test]
fn fixture_is_well_formed() {
    let fx = fixture();
    let routes = fx["routes"].as_array().unwrap();
    assert_eq!(fx["route_count"].as_u64().unwrap() as usize, routes.len());
    assert!(!routes.is_empty());
}
