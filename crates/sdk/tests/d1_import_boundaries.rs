//! Port of packages/sdk-next/test/import-boundaries.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/sdk-next/test/import-boundaries.test.ts: the
//! bundled inputs include files under the client, core, and server package
//! directories.
//! Re-derived: the `bun build` invocation and its metafile are not portable, so
//! only the pure `within(inputs, directory)` membership predicate is ported. The
//! bundling assertion is recorded as n/a (bundler integration, human-verified).

use opencode_sdk::import_boundaries::within_directory;

#[test]
fn matches_inputs_at_or_below_the_directory() {
    let inputs = vec![
        "/repo/packages/client/a.ts".to_string(),
        "/repo/packages/core/b.ts".to_string(),
        "/repo/packages/client".to_string(),
        "/repo/packages/client-next/c.ts".to_string(),
        "/repo/other/d.ts".to_string(),
    ];

    assert_eq!(
        within_directory(&inputs, "/repo/packages/client"),
        vec![
            "/repo/packages/client/a.ts".to_string(),
            "/repo/packages/client".to_string(),
        ]
    );
}
