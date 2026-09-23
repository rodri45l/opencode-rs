//! Port of packages/core/test/location.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a bound location resolves its directory, workspace,
//! enclosing project, and git VCS metadata. Re-derived: the reference injects a
//! `Project` layer; here the project root is discovered by walking up to the
//! enclosing `.git`, against a scratch directory.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use opencode_core::location::{Location, ProjectId, Vcs};
use opencode_core::path::AbsolutePath;
use opencode_schema::WorkspaceId;

const NOTE: &str = "porting: location not implemented";

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-rs-location-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn resolves_the_current_project_and_vcs_information() {
    let root = scratch();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    let app = root.join("packages").join("app");
    std::fs::create_dir_all(&app).unwrap();

    let workspace_id = WorkspaceId::parse("wrk_test").expect("valid workspace id");
    let directory = AbsolutePath::new(app.clone());

    let location = Location::resolve(&directory, workspace_id.clone()).expect(NOTE);

    let expected_id = root.file_name().unwrap().to_string_lossy().to_string();
    assert_eq!(location.directory, AbsolutePath::new(app));
    assert_eq!(location.workspace_id, workspace_id);
    assert_eq!(location.project.id, ProjectId::make(expected_id));
    assert_eq!(location.project.directory, AbsolutePath::new(root.clone()));
    assert_eq!(
        location.vcs,
        Some(Vcs::Git {
            store: AbsolutePath::new(root.join(".git")),
        })
    );
}
