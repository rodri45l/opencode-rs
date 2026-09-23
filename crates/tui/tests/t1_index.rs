//! Port of packages/tui/test/index.test.tsx (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/index.ts; see docs/TEST-PORT.md.
//! The lifecycle entrypoint itself is exercised by app-lifecycle (visual).

/// Local stand-in for the canonical application lifecycle entrypoint
/// (`run` re-exported from the package root).
fn run() {}

#[test]
#[ignore = "porting: tui package lifecycle entrypoint not implemented"]
fn exports_the_canonical_application_lifecycle() {
    let entrypoint: fn() = run;
    assert!(entrypoint as usize != 0);
}
