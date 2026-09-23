//! Port of packages/core/test/snapshot.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: capture treats a non-Git directory as unavailable, and
//! capture/files/restore round-trip Location-scoped changes without touching
//! files outside the scope. Re-derived: the `Global`/`Location`/Effect wiring and
//! the legacy-worktree-index cases are replaced by direct helpers over a real
//! temporary Git repository.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use opencode_core::snapshot::Snapshot;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn tmp() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-snapshot-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("tmpdir");
    dir
}

fn git(directory: &str, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(directory)
        .status()
        .expect("git");
    assert!(status.success(), "git {args:?}");
}

fn init_repo(directory: &Path) {
    let path = directory.to_str().expect("utf8");
    git(path, &["init"]);
    git(path, &["config", "core.fsmonitor", "false"]);
    git(path, &["config", "commit.gpgsign", "false"]);
    git(path, &["config", "user.email", "test@opencode.test"]);
    git(path, &["config", "user.name", "Test"]);
    fs::write(directory.join("tracked.txt"), "one\n").expect("write");
    git(path, &["add", "."]);
    git(path, &["commit", "-m", "initial"]);
}

#[test]
fn treats_capture_outside_git_as_unavailable() {
    let directory = tmp();
    let path = directory.to_str().expect("utf8");
    assert_eq!(Snapshot::capture(path).unwrap(), None);
}

#[test]
fn captures_and_restores_location_scoped_changes() {
    let directory = tmp();
    init_repo(&directory);
    let path = directory.to_str().expect("utf8");

    let before = Snapshot::capture(path).unwrap().unwrap();
    fs::write(directory.join("tracked.txt"), "two\n").expect("write");
    fs::write(directory.join("added.txt"), "added\n").expect("write");
    let after = Snapshot::capture(path).unwrap().unwrap();

    let files = Snapshot::files(path, &before, &after).unwrap();
    assert_eq!(files, vec!["added.txt", "tracked.txt"]);
    assert_eq!(
        Snapshot::preview(path, "tracked.txt", &before).unwrap(),
        "one\n"
    );

    Snapshot::restore(path, &[("tracked.txt".into(), before)]).unwrap();
    assert_eq!(
        fs::read_to_string(directory.join("tracked.txt")).expect("read"),
        "one\n"
    );
    assert_eq!(
        fs::read_to_string(directory.join("added.txt")).expect("read"),
        "added\n"
    );
}
