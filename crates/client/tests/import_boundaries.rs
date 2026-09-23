//! Port of packages/client/test/import-boundaries.test.ts (upstream 18ef3cc).
//!
//! The reference bundles the JS entrypoints and asserts the public
//! `@opencode-ai/client` boundary pulls in no Effect/Schema/Protocol/Core/Server
//! modules, while `@opencode-ai/client/effect` pulls in Effect/Schema/Protocol
//! but not Core/Server. Rust has no equivalent dual entrypoint or bundler, so
//! this is re-derived as a manifest-level boundary check: the `opencode-client`
//! crate must depend only on the wire layer (`opencode-schema`) and never on the
//! higher `opencode-protocol`/`opencode-core`/`opencode-server` layers.

const CLIENT_MANIFEST: &str = include_str!("../Cargo.toml");

#[test]
fn public_entrypoint_does_not_depend_on_higher_layers() {
    for forbidden in ["opencode-protocol", "opencode-core", "opencode-server"] {
        assert!(
            !CLIENT_MANIFEST.contains(forbidden),
            "opencode-client must not depend on {forbidden}"
        );
    }
}

#[test]
fn wire_layer_dependency_is_the_only_shared_layer() {
    assert!(CLIENT_MANIFEST.contains("opencode-schema.workspace = true"));
}
