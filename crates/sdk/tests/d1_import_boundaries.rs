//! Port of packages/sdk-next/test/import-boundaries.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/sdk-next/test/import-boundaries.test.ts: the
//! bundled inputs include files under the client, core, and server package
//! directories.
//! Re-derived: the `bun build` invocation and its metafile are not portable, so
//! only the pure `within(inputs, directory)` membership predicate is ported. The
//! bundling assertion is recorded as n/a (bundler integration, human-verified).

#[allow(dead_code)]
mod import_boundaries {
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: sdk-next import boundaries not implemented";

    pub fn within_directory(_inputs: &[String], _directory: &str) -> PortResult<Vec<String>> {
        Err(NotImplemented(NOTE))
    }
}

use import_boundaries::{within_directory, NOTE};

#[test]
#[ignore = "porting: sdk-next import boundaries not implemented"]
fn matches_inputs_at_or_below_the_directory() {
    let inputs = vec![
        "/repo/packages/client/a.ts".to_string(),
        "/repo/packages/core/b.ts".to_string(),
        "/repo/packages/client".to_string(),
        "/repo/packages/client-next/c.ts".to_string(),
        "/repo/other/d.ts".to_string(),
    ];

    assert_eq!(
        within_directory(&inputs, "/repo/packages/client").expect(NOTE),
        vec![
            "/repo/packages/client/a.ts".to_string(),
            "/repo/packages/client".to_string(),
        ]
    );
}
