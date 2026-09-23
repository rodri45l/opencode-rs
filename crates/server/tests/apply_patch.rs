//! Port of packages/opencode/test/tool/apply_patch.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: freeform patch validation, add/update/delete/move, hunk
//! application, BOM handling, trailing newline normalisation, and permission
//! metadata for UI rendering. The remaining reference cases (delete-target
//! directory rejection, invalid hunk shapes) are covered by the tool's own
//! unit surface once it lands.

use opencode_server::port::tools::{ApplyPatchArgs, ApplyPatchTool};
use opencode_server::tools::{ToolContext, ToolError, ToolResult};
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("opencode-patch-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn context(directory: &Path) -> ToolContext {
    ToolContext {
        session_id: "ses_test".to_string(),
        message_id: "msg_test".to_string(),
        agent: "build".to_string(),
        directory: directory.to_path_buf(),
        ..ToolContext::default()
    }
}

fn run(dir: &Path, patch_text: &str) -> Result<ToolResult, ToolError> {
    ApplyPatchTool::new().execute(
        ApplyPatchArgs {
            patch_text: patch_text.to_string(),
        },
        &mut context(dir),
    )
}

#[test]
#[ignore = "porting: tool.apply_patch not implemented"]
fn requires_patch_text() {
    let dir = temp_dir("requires");
    let error = run(&dir, "").expect_err("empty patch rejected");
    assert!(error.to_string().contains("patchText is required"));
}

#[test]
#[ignore = "porting: tool.apply_patch not implemented"]
fn rejects_invalid_patch_format() {
    let dir = temp_dir("invalid");
    let error = run(&dir, "invalid patch").expect_err("invalid rejected");
    assert!(error
        .to_string()
        .contains("apply_patch verification failed"));
}

#[test]
#[ignore = "porting: tool.apply_patch not implemented"]
fn rejects_empty_patch() {
    let dir = temp_dir("empty");
    let error = run(&dir, "*** Begin Patch\n*** End Patch").expect_err("empty rejected");
    assert!(error.to_string().contains("patch rejected: empty patch"));
}

#[test]
#[ignore = "porting: tool.apply_patch not implemented"]
fn applies_add_update_and_delete_in_one_patch() -> Result<(), ToolError> {
    let dir = temp_dir("combined");
    std::fs::write(dir.join("modify.txt"), "line1\nline2\n").expect("seed");
    std::fs::write(dir.join("delete.txt"), "obsolete\n").expect("seed");

    let patch_text = "*** Begin Patch\n*** Add File: nested/new.txt\n+created\n*** Delete File: delete.txt\n*** Update File: modify.txt\n@@\n-line2\n+changed\n*** End Patch";
    let mut ctx = context(&dir);
    let result = ApplyPatchTool::new().execute(
        ApplyPatchArgs {
            patch_text: patch_text.to_string(),
        },
        &mut ctx,
    )?;

    assert!(result
        .output
        .contains("Success. Updated the following files"));
    assert!(result.output.contains("nested/new.txt"));
    assert!(result.output.contains("delete.txt"));
    assert!(result.output.contains("modify.txt"));
    assert!(result.metadata["diff"]
        .as_str()
        .expect("diff string")
        .contains("Index:"));
    assert_eq!(ctx.requests.len(), 1);
    let files = ctx.requests[0].metadata["files"]
        .as_array()
        .expect("files array");
    assert_eq!(files.len(), 3);
    assert_eq!(
        std::fs::read_to_string(dir.join("nested").join("new.txt")).expect("read"),
        "created\n"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("modify.txt")).expect("read"),
        "line1\nchanged\n"
    );
    assert!(!dir.join("delete.txt").exists());
    Ok(())
}

#[test]
#[ignore = "porting: tool.apply_patch not implemented"]
fn permission_metadata_includes_move_file_info() -> Result<(), ToolError> {
    let dir = temp_dir("move-info");
    let original = dir.join("old").join("name.txt");
    std::fs::create_dir_all(original.parent().expect("parent")).expect("dirs");
    std::fs::write(&original, "old content\n").expect("seed");

    let patch_text = "*** Begin Patch\n*** Update File: old/name.txt\n*** Move to: renamed/dir/name.txt\n@@\n-old content\n+new content\n*** End Patch";
    let mut ctx = context(&dir);
    let _ = ApplyPatchTool::new().execute(
        ApplyPatchArgs {
            patch_text: patch_text.to_string(),
        },
        &mut ctx,
    )?;

    assert_eq!(ctx.requests.len(), 1);
    let files = ctx.requests[0].metadata["files"].as_array().expect("files");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["type"].as_str(), Some("move"));
    assert_eq!(
        files[0]["relativePath"].as_str(),
        Some("renamed/dir/name.txt")
    );
    Ok(())
}

#[test]
#[ignore = "porting: tool.apply_patch not implemented"]
fn applies_multiple_hunks_to_one_file() -> Result<(), ToolError> {
    let dir = temp_dir("hunks");
    let target = dir.join("multi.txt");
    std::fs::write(&target, "line1\nline2\nline3\nline4\n").expect("seed");

    let patch_text = "*** Begin Patch\n*** Update File: multi.txt\n@@\n-line2\n+changed2\n@@\n-line4\n+changed4\n*** End Patch";
    run(&dir, patch_text)?;

    assert_eq!(
        std::fs::read_to_string(&target).expect("read"),
        "line1\nchanged2\nline3\nchanged4\n"
    );
    Ok(())
}

#[test]
#[ignore = "porting: tool.apply_patch not implemented"]
fn does_not_invent_a_first_line_diff_for_bom_files() -> Result<(), ToolError> {
    let dir = temp_dir("bom");
    let target = dir.join("example.cs");
    let bom = '\u{feff}';
    std::fs::write(&target, format!("{bom}using System;\n\nclass Test {{}}\n")).expect("seed");

    let patch_text = "*** Begin Patch\n*** Update File: example.cs\n@@\n class Test {}\n+class Next {}\n*** End Patch";
    let mut ctx = context(&dir);
    let _ = ApplyPatchTool::new().execute(
        ApplyPatchArgs {
            patch_text: patch_text.to_string(),
        },
        &mut ctx,
    )?;

    assert_eq!(ctx.requests.len(), 1);
    let shown = ctx.requests[0].metadata["files"][0]["patch"]
        .as_str()
        .unwrap_or("")
        .to_string();
    assert!(!shown.contains(bom));
    assert!(!shown.contains("-using System;"));
    assert!(!shown.contains("+using System;"));
    Ok(())
}

#[test]
#[ignore = "porting: tool.apply_patch not implemented"]
fn inserts_lines_with_insert_only_hunk() -> Result<(), ToolError> {
    let dir = temp_dir("insert");
    let target = dir.join("insert_only.txt");
    std::fs::write(&target, "alpha\nomega\n").expect("seed");

    let patch_text = "*** Begin Patch\n*** Update File: insert_only.txt\n@@\n alpha\n+beta\n omega\n*** End Patch";
    run(&dir, patch_text)?;

    assert_eq!(
        std::fs::read_to_string(&target).expect("read"),
        "alpha\nbeta\nomega\n"
    );
    Ok(())
}

#[test]
#[ignore = "porting: tool.apply_patch not implemented"]
fn appends_trailing_newline_on_update() -> Result<(), ToolError> {
    let dir = temp_dir("trailing");
    let target = dir.join("no_newline.txt");
    std::fs::write(&target, "no newline at end").expect("seed");

    let patch_text = "*** Begin Patch\n*** Update File: no_newline.txt\n@@\n-no newline at end\n+first line\n+second line\n*** End Patch";
    run(&dir, patch_text)?;

    assert_eq!(
        std::fs::read_to_string(&target).expect("read"),
        "first line\nsecond line\n"
    );
    Ok(())
}

#[test]
#[ignore = "porting: tool.apply_patch not implemented"]
fn moves_file_to_a_new_directory() -> Result<(), ToolError> {
    let dir = temp_dir("move");
    let original = dir.join("old").join("name.txt");
    std::fs::create_dir_all(original.parent().expect("parent")).expect("dirs");
    std::fs::write(&original, "old content\n").expect("seed");

    let patch_text = "*** Begin Patch\n*** Update File: old/name.txt\n*** Move to: renamed/dir/name.txt\n@@\n-old content\n+new content\n*** End Patch";
    run(&dir, patch_text)?;

    let moved = dir.join("renamed").join("dir").join("name.txt");
    assert!(!original.exists());
    assert_eq!(
        std::fs::read_to_string(&moved).expect("read"),
        "new content\n"
    );
    Ok(())
}

#[test]
#[ignore = "porting: tool.apply_patch not implemented"]
fn adds_file_overwriting_existing_file() -> Result<(), ToolError> {
    let dir = temp_dir("overwrite");
    let target = dir.join("duplicate.txt");
    std::fs::write(&target, "old content\n").expect("seed");

    let patch_text = "*** Begin Patch\n*** Add File: duplicate.txt\n+new content\n*** End Patch";
    run(&dir, patch_text)?;

    assert_eq!(
        std::fs::read_to_string(&target).expect("read"),
        "new content\n"
    );
    Ok(())
}

#[test]
#[ignore = "porting: tool.apply_patch not implemented"]
fn rejects_update_when_target_file_is_missing() {
    let dir = temp_dir("missing-update");
    let patch_text =
        "*** Begin Patch\n*** Update File: missing.txt\n@@\n-nope\n+better\n*** End Patch";
    let error = run(&dir, patch_text).expect_err("missing target");
    assert!(error
        .to_string()
        .contains("apply_patch verification failed: Failed to read file to update"));
}

#[test]
#[ignore = "porting: tool.apply_patch not implemented"]
fn rejects_delete_when_file_is_missing() {
    let dir = temp_dir("missing-delete");
    let patch_text = "*** Begin Patch\n*** Delete File: missing.txt\n*** End Patch";
    assert!(run(&dir, patch_text).is_err());
}
