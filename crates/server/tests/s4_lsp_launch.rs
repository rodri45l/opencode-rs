//! Port of packages/opencode/test/lsp/launch.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/lsp/launch.ts; see docs/TEST-PORT.md.
//!
//! Ported: `spawn` for a `.cmd` script whose path contains spaces, asserting the
//! process exits with the script's code.
//! Dropped: none. The upstream case returns early on non-Windows hosts; the Rust
//! test is a no-op guard there, mirroring that behaviour once the module lands.

use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

#[allow(dead_code)]
fn spawn(_command: &Path, _args: &[&str]) -> Result<i32, NotImplemented> {
    nope("lsp launch")
}

#[cfg(windows)]
#[test]
#[ignore = "porting: lsp launch not implemented"]
fn spawns_cmd_scripts_with_spaces_on_windows() {
    let file = Path::new("C:/tmp/with space/echo cmd.cmd");
    assert_eq!(spawn(file, &["--stdio"]).unwrap(), 0);
}
