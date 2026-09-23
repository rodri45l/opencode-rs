//! Port of packages/core/test/project-directories.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: directory list input/output schemas decode plain data,
//! `create` inserts once and ignores a conflicting insert, and an explicit
//! `replace` behavior updates the stored strategy while a plain replace without
//! a strategy clears it. Re-derived: the Database/Effect service wiring is
//! replaced by an in-memory store.

use opencode_core::project_directories::{
    CreateBehavior, CreateInput, DirectoryEntry, ProjectDirectories,
};
use serde_json::json;

const NOTE: &str = "porting: project directories not implemented";

const PROJECT_ID: &str = "project-directories";
const DIRECTORY: &str = "/tmp/project-directories";

fn create(strategy: Option<&str>, behavior: CreateBehavior) -> CreateInput {
    CreateInput {
        project_id: PROJECT_ID.into(),
        directory: DIRECTORY.into(),
        strategy: strategy.map(str::to_string),
        behavior,
    }
}

#[test]
#[ignore = "porting: project directories not implemented"]
fn decodes_directory_schemas() {
    let input =
        ProjectDirectories::decode_list_input(&json!({ "projectID": PROJECT_ID })).expect(NOTE);
    assert_eq!(input.project_id, PROJECT_ID);

    let output =
        ProjectDirectories::decode_list_output(&json!([{ "directory": DIRECTORY }])).expect(NOTE);
    assert_eq!(
        output,
        vec![DirectoryEntry {
            directory: DIRECTORY.into(),
            strategy: None,
        }]
    );
}

#[test]
#[ignore = "porting: project directories not implemented"]
fn creates_once_and_ignores_conflicts() {
    let mut directories = ProjectDirectories::new();
    assert!(directories
        .create(create(None, CreateBehavior::Ignore))
        .expect(NOTE));
    assert!(!directories
        .create(create(Some("git_worktree"), CreateBehavior::Ignore))
        .expect(NOTE));
    assert_eq!(
        directories.list(PROJECT_ID).expect(NOTE),
        vec![DirectoryEntry {
            directory: DIRECTORY.into(),
            strategy: None,
        }]
    );
}

#[test]
#[ignore = "porting: project directories not implemented"]
fn replaces_the_strategy_when_requested() {
    let mut directories = ProjectDirectories::new();
    directories
        .create(create(Some("old/strategy"), CreateBehavior::Ignore))
        .expect(NOTE);

    assert!(directories
        .create(create(Some("new/strategy"), CreateBehavior::Replace))
        .expect(NOTE));
    assert!(!directories
        .create(create(Some("new/strategy"), CreateBehavior::Replace))
        .expect(NOTE));
    assert!(directories
        .create(create(None, CreateBehavior::Replace))
        .expect(NOTE));
    assert!(!directories
        .create(create(None, CreateBehavior::Replace))
        .expect(NOTE));
    assert!(directories
        .create(create(Some("new/strategy"), CreateBehavior::Replace))
        .expect(NOTE));
    assert_eq!(
        directories.list(PROJECT_ID).expect(NOTE),
        vec![DirectoryEntry {
            directory: DIRECTORY.into(),
            strategy: Some("new/strategy".into()),
        }]
    );
}
