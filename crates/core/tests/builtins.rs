//! Port of packages/core/test/system-context/builtins.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: built-in context renders the location-scoped environment
//! (working directory, workspace root, git-repo flag, platform) followed by the
//! host-local date. Re-derived as a pure renderer; the `TestClock` date-reconcile
//! and instruction-composition cases are dropped.

use opencode_core::system_context::{BuiltinEnv, SystemContextBuiltIns};

fn env(is_git_repo: bool) -> BuiltinEnv {
    BuiltinEnv {
        working_directory: "/repo/packages/core".into(),
        workspace_root: "/repo".into(),
        is_git_repo,
        platform: "linux".into(),
        date: "Wed Jun 03 2026".into(),
    }
}

#[test]
fn renders_environment_and_host_local_date() {
    let baseline = SystemContextBuiltIns::render(&env(true)).unwrap();

    assert_eq!(
        baseline,
        [
            "Here is some useful information about the environment you are running in:",
            "<env>",
            "  Working directory: /repo/packages/core",
            "  Workspace root folder: /repo",
            "  Is directory a git repo: yes",
            "  Platform: linux",
            "</env>",
            "",
            "Today's date: Wed Jun 03 2026",
        ]
        .join("\n")
    );
}

#[test]
fn marks_non_git_directories() {
    let baseline = SystemContextBuiltIns::render(&env(false)).unwrap();
    assert!(baseline.contains("  Is directory a git repo: no"));
}
