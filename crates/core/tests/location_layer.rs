//! Port of packages/core/test/location-layer.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a directly constructed and a decoded location reference are
//! equal and hash the same, the service map returns one context identity for
//! equal refs, location state is isolated per directory, and the shared policy
//! can deny a configured provider by resource. Re-derived: Effect `Context`/
//! `Layer`/`Scope` wiring is replaced by an explicit service map.

use opencode_core::location_layer::{LocationRef, LocationServiceMap};
use serde_json::json;

const NOTE: &str = "porting: location service map not implemented";

#[test]
#[ignore = "porting: location service map not implemented"]
fn reuses_cached_services_for_constructed_and_decoded_refs() {
    let constructed = LocationRef::new("/tmp/location");
    let decoded = LocationRef::decode(&json!({ "directory": "/tmp/location" })).expect(NOTE);

    assert_eq!(constructed, decoded);
    assert_eq!(constructed.workspace_id, None);

    let locations = LocationServiceMap::new();
    assert_eq!(
        locations.context_effect(&constructed).expect(NOTE),
        locations.context_effect(&decoded).expect(NOTE)
    );
}

#[test]
#[ignore = "porting: location service map not implemented"]
fn isolates_location_state_while_sharing_policy() {
    let blocked = LocationRef::new("/tmp/blocked");
    let allowed = LocationRef::new("/tmp/allowed");

    let mut locations = LocationServiceMap::new();
    locations
        .add_policy(json!({ "effect": "deny", "action": "provider.use", "resource": "test" }))
        .expect(NOTE);
    locations
        .configure(&blocked, vec!["test".into()])
        .expect(NOTE);
    locations
        .configure(&allowed, vec!["test".into()])
        .expect(NOTE);

    assert!(locations.provider_ids(&blocked).expect(NOTE).is_empty());
    assert_eq!(locations.provider_ids(&allowed).expect(NOTE), vec!["test"]);
}
