//! Port of packages/core/test/util/which.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `which` returns `None` for a missing command, finds a
//! command from a PATH override, uses the first PATH match, and ignores
//! non-executable files on unix. Dropped: the Windows PATHEXT/Path-casing cases.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use opencode_core::which::Which;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-rs-which-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn cmd(dir: &std::path::Path, name: &str, exec: bool) -> PathBuf {
    let file = dir.join(name);
    std::fs::write(&file, "#!/bin/sh\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if exec { 0o755 } else { 0o644 };
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(mode)).unwrap();
    }
    let _ = exec;
    file
}

#[test]
fn returns_none_when_command_is_missing() {
    assert_eq!(
        Which::which("opencode-missing-command-for-test", "", None).unwrap(),
        None
    );
}

#[test]
fn finds_a_command_from_path_override() {
    let dir = scratch();
    let bin = dir.join("bin");
    std::fs::create_dir(&bin).unwrap();
    let file = cmd(&bin, "tool", true);

    assert_eq!(
        Which::which("tool", &bin.to_string_lossy(), None).unwrap(),
        Some(file)
    );
}

#[test]
fn uses_first_path_match() {
    let dir = scratch();
    let a = dir.join("a");
    let b = dir.join("b");
    std::fs::create_dir(&a).unwrap();
    std::fs::create_dir(&b).unwrap();
    let first = cmd(&a, "dupe", true);
    cmd(&b, "dupe", true);

    let path = std::env::join_paths([&a, &b]).unwrap();
    assert_eq!(
        Which::which("dupe", &path.to_string_lossy(), None).unwrap(),
        Some(first)
    );
}

#[cfg(unix)]
#[test]
fn returns_none_for_non_executable_file_on_unix() {
    let dir = scratch();
    let bin = dir.join("bin");
    std::fs::create_dir(&bin).unwrap();
    cmd(&bin, "noexec", false);

    assert_eq!(
        Which::which("noexec", &bin.to_string_lossy(), None).unwrap(),
        None
    );
}
