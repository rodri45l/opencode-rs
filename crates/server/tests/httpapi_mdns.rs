//! Port of packages/opencode/test/server/httpapi-mdns.test.ts (upstream 18ef3cc).
//!
//! Subset: the publish decision and advertised name are ported green against
//! `opencode_server::mdns`. The lifecycle cases (publish on listen, unpublish on
//! stop, scope finalizer) need the Bonjour transport and the listener API and
//! are left to the listen phase.

use opencode_server::mdns::{service_name, should_publish};

#[test]
fn skips_publish_for_loopback_hostnames() {
    assert!(!should_publish(true, "127.0.0.1", 4096));
    assert!(!should_publish(true, "localhost", 4096));
    assert!(!should_publish(true, "::1", 4096));
}

#[test]
fn skips_publish_when_mdns_is_disabled() {
    assert!(!should_publish(false, "0.0.0.0", 4096));
}

#[test]
fn skips_publish_without_a_bound_port() {
    assert!(!should_publish(true, "0.0.0.0", 0));
}

#[test]
fn publishes_for_non_loopback_hostnames() {
    assert!(should_publish(true, "0.0.0.0", 4096));
}

#[test]
fn advertises_opencode_prefixed_port() {
    assert_eq!(service_name(4096), "opencode-4096");
}
