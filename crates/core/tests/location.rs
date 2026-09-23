//! Port of packages/core/test/location.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a bound location resolves its directory, workspace,
//! enclosing project, and git VCS metadata.

use opencode_core::location::{Location, ProjectId, Vcs};
use opencode_core::path::AbsolutePath;
use opencode_schema::WorkspaceId;

const NOTE: &str = "porting: location not implemented";

#[test]
#[ignore = "porting: location not implemented"]
fn resolves_the_current_project_and_vcs_information() {
    let workspace_id = WorkspaceId::parse("wrk_test").expect("valid workspace id");
    let directory = AbsolutePath::new("/repo/packages/app");

    let location = Location::resolve(&directory, workspace_id.clone()).expect(NOTE);

    assert_eq!(location.directory, AbsolutePath::new("/repo/packages/app"));
    assert_eq!(location.workspace_id, workspace_id);
    assert_eq!(location.project.id, ProjectId::make("project"));
    assert_eq!(location.project.directory, AbsolutePath::new("/repo"));
    assert_eq!(
        location.vcs,
        Some(Vcs::Git {
            store: AbsolutePath::new("/repo/.git"),
        })
    );
}
