//! Port of packages/core/test/move-session.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: moving a session to a destination directory records the
//! destination and the path relative to its worktree when no changes are moved,
//! and transfers in-scope changes when requested. Re-derived: the Database/
//! Project/Event wiring is replaced by direct calls over temporary directories;
//! the nested-checkout and staged-file cases are noted as dropped.

use std::fs;
use std::path::PathBuf;

use opencode_core::move_session::{MoveDestination, MoveSession};

const NOTE: &str = "porting: move session not implemented";

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("opencode-move-{}-{tag}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("tmpdir");
    dir
}

#[test]
#[ignore = "porting: move session not implemented"]
fn moves_within_a_checkout_without_transferring_existing_changes() {
    let source = tmp("source");
    let destination = source.join("packages");
    fs::create_dir_all(&destination).expect("packages");
    fs::write(source.join("tracked.txt"), "changed\n").expect("write");

    let moved = MoveSession::move_session(
        "ses_move_nested",
        source.to_str().expect("utf8"),
        &MoveDestination {
            directory: destination.to_str().expect("utf8").to_string(),
        },
        true,
    )
    .expect(NOTE);

    assert_eq!(moved.directory, destination.to_str().expect("utf8"));
    assert_eq!(moved.path, "packages");
    assert_eq!(
        fs::read_to_string(source.join("tracked.txt")).expect("read"),
        "changed\n"
    );
}

#[test]
#[ignore = "porting: move session not implemented"]
fn moves_session_changes_to_another_project_directory() {
    let source = tmp("source-transfer");
    let destination = tmp("destination-transfer");
    fs::write(source.join("tracked.txt"), "changed\n").expect("write");
    fs::write(source.join("untracked.txt"), "new\n").expect("write");

    let moved = MoveSession::move_session(
        "ses_move",
        source.to_str().expect("utf8"),
        &MoveDestination {
            directory: destination.to_str().expect("utf8").to_string(),
        },
        true,
    )
    .expect(NOTE);

    assert_eq!(moved.directory, destination.to_str().expect("utf8"));
    assert_eq!(moved.path, "");
    assert_eq!(
        fs::read_to_string(destination.join("tracked.txt")).expect("read"),
        "changed\n"
    );
    assert_eq!(
        fs::read_to_string(destination.join("untracked.txt")).expect("read"),
        "new\n"
    );
}
