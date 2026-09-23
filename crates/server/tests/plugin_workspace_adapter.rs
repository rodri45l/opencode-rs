//! Port of packages/opencode/test/plugin/workspace-adapter.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a plugin can register a workspace adapter that rewrites
//! the requested workspace (name/branch/directory) while preserving `extra`.
//! The reference loads the adapter from a TS plugin file; this port registers
//! the equivalent adapter directly. Project id/type plumbing lives on the
//! workspace service and is out of scope for the registry.

use opencode_server::port::plugin::{
    WorkspaceAdapter, WorkspaceError, WorkspaceInput, WorkspaceRegistry,
};
use serde_json::json;
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("opencode-workspace-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[test]
fn plugin_can_install_a_workspace_adapter() -> Result<(), WorkspaceError> {
    let dir = temp_dir("adapter");
    let space = dir.join("space");
    let mark = dir.join("created.json");

    let mut registry = WorkspaceRegistry::new();
    let mark_for_adapter = mark.clone();
    let space_for_adapter = space.clone();
    registry.register(
        "plug",
        WorkspaceAdapter {
            name: "plug".to_string(),
            description: "plugin workspace adapter".to_string(),
            configure: Box::new(move |input: WorkspaceInput| {
                std::fs::write(
                    &mark_for_adapter,
                    serde_json::to_vec(&json!({
                        "name": "plug",
                        "branch": "plug/main",
                        "directory": space_for_adapter.to_string_lossy(),
                        "extra": input.extra.clone(),
                    }))
                    .expect("mark serializes"),
                )
                .expect("mark written");
                WorkspaceInput {
                    name: "plug".to_string(),
                    branch: Some("plug/main".to_string()),
                    directory: space_for_adapter.to_string_lossy().into_owned(),
                    extra: input.extra,
                }
            }),
        },
    );

    let info = registry.create(
        "plug",
        WorkspaceInput {
            name: String::new(),
            branch: None,
            directory: dir.to_string_lossy().into_owned(),
            extra: json!({ "key": "value" }),
        },
    )?;

    assert_eq!(info.name, "plug");
    assert_eq!(info.branch.as_deref(), Some("plug/main"));
    assert_eq!(info.directory, space.to_string_lossy().to_string());
    assert_eq!(info.extra, json!({ "key": "value" }));

    let saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&mark).expect("mark read")).expect("mark json");
    assert_eq!(saved["name"].as_str(), Some("plug"));
    assert_eq!(saved["branch"].as_str(), Some("plug/main"));
    assert_eq!(saved["extra"], json!({ "key": "value" }));
    Ok(())
}
