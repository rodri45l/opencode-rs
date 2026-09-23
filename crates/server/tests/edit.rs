//! Port of packages/opencode/test/tool/edit.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: creating files, replacing text, replaceAll, BOM
//! preservation, identical/not-found/anchor-mismatch failures, directory
//! rejection, diff statistics, and line-ending preservation. The watcher
//! event cases and the concurrent-editing case are dropped (they need the
//! event bus and fiber scheduling).

use opencode_server::port::tools::{EditArgs, EditTool};
use opencode_server::tools::{ToolContext, ToolError, ToolResult};
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("opencode-edit-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn context(directory: &Path) -> ToolContext {
    ToolContext {
        session_id: "ses_test-edit-session".to_string(),
        message_id: "msg_test".to_string(),
        agent: "build".to_string(),
        directory: directory.to_path_buf(),
        ..ToolContext::default()
    }
}

fn run(
    dir: &Path,
    file: &Path,
    old_string: &str,
    new_string: &str,
    replace_all: Option<bool>,
) -> Result<ToolResult, ToolError> {
    EditTool::new().execute(
        EditArgs {
            file_path: file.to_string_lossy().into_owned(),
            old_string: old_string.to_string(),
            new_string: new_string.to_string(),
            replace_all,
        },
        &mut context(dir),
    )
}

#[test]
fn creates_new_file_when_old_string_is_empty() -> Result<(), ToolError> {
    let dir = temp_dir("create");
    let file = dir.join("newfile.txt");

    let result = run(&dir, &file, "", "new content", None)?;
    assert!(result.metadata["diff"]
        .as_str()
        .expect("diff string")
        .contains("new content"));
    assert_eq!(std::fs::read_to_string(&file).expect("read"), "new content");
    Ok(())
}

#[test]
fn rejects_empty_old_string_on_existing_files() {
    let dir = temp_dir("create-existing");
    let file = dir.join("existing.cs");
    let bom = '\u{feff}';
    let original = format!("{bom}using System;\n");
    std::fs::write(&file, &original).expect("seed");

    let error = run(&dir, &file, "", "using Up;\n", None).expect_err("should reject");
    assert!(error.to_string().contains("oldString cannot be empty"));
    assert_eq!(std::fs::read_to_string(&file).expect("read"), original);
}

#[test]
fn creates_new_file_with_nested_directories() -> Result<(), ToolError> {
    let dir = temp_dir("nested");
    let file = dir.join("nested").join("dir").join("file.txt");

    run(&dir, &file, "", "nested file", None)?;
    assert_eq!(std::fs::read_to_string(&file).expect("read"), "nested file");
    Ok(())
}

#[test]
fn replaces_text_in_existing_file() -> Result<(), ToolError> {
    let dir = temp_dir("replace");
    let file = dir.join("existing.txt");
    std::fs::write(&file, "old content here").expect("seed");

    let result = run(&dir, &file, "old content", "new content", None)?;
    assert!(result.output.contains("Edit applied successfully"));
    assert_eq!(
        std::fs::read_to_string(&file).expect("read"),
        "new content here"
    );
    Ok(())
}

#[test]
fn replaces_the_first_visible_line_in_bom_files() -> Result<(), ToolError> {
    let dir = temp_dir("bom");
    let file = dir.join("existing.cs");
    let bom = '\u{feff}';
    std::fs::write(&file, format!("{bom}using System;\nclass Test {{}}\n")).expect("seed");

    let result = run(&dir, &file, "using System;", "using Up;", None)?;
    let diff = result.metadata["diff"].as_str().expect("diff string");
    assert!(diff.contains("-using System;"));
    assert!(diff.contains("+using Up;"));
    assert!(!diff.contains(bom));

    let content = std::fs::read_to_string(&file).expect("read");
    assert_eq!(content.chars().next(), Some(bom));
    assert_eq!(&content[bom.len_utf8()..], "using Up;\nclass Test {}\n");
    Ok(())
}

#[test]
fn throws_when_file_does_not_exist() {
    let dir = temp_dir("missing");
    let file = dir.join("nonexistent.txt");
    let error = run(&dir, &file, "old", "new", None).expect_err("missing file");
    assert!(error.to_string().contains("not found"));
}

#[test]
fn throws_when_old_string_equals_new_string() {
    let dir = temp_dir("identical");
    let file = dir.join("file.txt");
    std::fs::write(&file, "content").expect("seed");
    let error = run(&dir, &file, "same", "same", None).expect_err("identical");
    assert!(error.to_string().contains("identical"));
}

#[test]
fn throws_when_old_string_not_found_in_file() {
    let dir = temp_dir("not-found");
    let file = dir.join("file.txt");
    std::fs::write(&file, "actual content").expect("seed");
    assert!(run(&dir, &file, "not in file", "replacement", None).is_err());
}

#[test]
fn rejects_loose_block_anchor_matches() {
    let dir = temp_dir("anchor");
    let file = dir.join("file.ts");
    let original = [
        "function configure() {",
        "  keepImportantState()",
        "  removeAllUserData()",
        "  archiveBackups()",
        "  auditLog()",
        "}",
    ]
    .join("\n");
    std::fs::write(&file, &original).expect("seed");

    let error = run(
        &dir,
        &file,
        "function configure() {\n  const enabled = true\n}",
        "function configure() {\n  const enabled = false\n}",
        None,
    )
    .expect_err("anchor mismatch");
    assert!(error.to_string().contains("Could not find oldString"));
    assert_eq!(std::fs::read_to_string(&file).expect("read"), original);
}

#[test]
fn replaces_all_occurrences_with_replace_all_option() -> Result<(), ToolError> {
    let dir = temp_dir("replace-all");
    let file = dir.join("file.txt");
    std::fs::write(&file, "foo bar foo baz foo").expect("seed");

    run(&dir, &file, "foo", "qux", Some(true))?;
    assert_eq!(
        std::fs::read_to_string(&file).expect("read"),
        "qux bar qux baz qux"
    );
    Ok(())
}

#[test]
fn handles_multiline_replacements() -> Result<(), ToolError> {
    let dir = temp_dir("multiline");
    let file = dir.join("file.txt");
    std::fs::write(&file, "line1\nline2\nline3").expect("seed");

    run(&dir, &file, "line2", "new line 2\nextra line", None)?;
    assert_eq!(
        std::fs::read_to_string(&file).expect("read"),
        "line1\nnew line 2\nextra line\nline3"
    );
    Ok(())
}

#[test]
fn handles_crlf_line_endings() -> Result<(), ToolError> {
    let dir = temp_dir("crlf");
    let file = dir.join("file.txt");
    std::fs::write(&file, "line1\r\nold\r\nline3").expect("seed");

    run(&dir, &file, "old", "new", None)?;
    assert_eq!(
        std::fs::read_to_string(&file).expect("read"),
        "line1\r\nnew\r\nline3"
    );
    Ok(())
}

#[test]
fn throws_when_path_is_a_directory() {
    let dir = temp_dir("directory");
    let target = dir.join("adir");
    std::fs::create_dir_all(&target).expect("dir");
    let error = run(&dir, &target, "old", "new", None).expect_err("directory");
    assert!(error.to_string().contains("directory"));
}

#[test]
fn tracks_file_diff_statistics() -> Result<(), ToolError> {
    let dir = temp_dir("stats");
    let file = dir.join("file.txt");
    std::fs::write(&file, "line1\nline2\nline3").expect("seed");

    let result = run(&dir, &file, "line2", "new line a\nnew line b", None)?;
    assert_eq!(
        result.metadata["filediff"]["file"].as_str(),
        Some(file.to_string_lossy().as_ref())
    );
    assert!(
        result.metadata["filediff"]["additions"]
            .as_u64()
            .unwrap_or(0)
            > 0
    );
    Ok(())
}

#[test]
fn preserves_lf_when_new_string_uses_crlf() -> Result<(), ToolError> {
    let dir = temp_dir("lf-crlf");
    let file = dir.join("test.txt");
    std::fs::write(&file, "alpha\nbeta\ngamma\n").expect("seed");

    run(
        &dir,
        &file,
        "alpha\nbeta\ngamma",
        "alpha\nbeta-updated\ngamma",
        None,
    )?;

    let output = std::fs::read_to_string(&file).expect("read");
    assert_eq!(output, "alpha\nbeta-updated\ngamma\n");
    assert!(!output.contains("\r\n"));
    Ok(())
}

#[test]
fn preserves_crlf_when_new_string_uses_lf() -> Result<(), ToolError> {
    let dir = temp_dir("crlf-lf");
    let file = dir.join("test.txt");
    std::fs::write(&file, "alpha\r\nbeta\r\ngamma\r\n").expect("seed");

    run(
        &dir,
        &file,
        "alpha\r\nbeta\r\ngamma",
        "alpha\r\nbeta-updated\r\ngamma",
        None,
    )?;

    let output = std::fs::read_to_string(&file).expect("read");
    assert_eq!(output, "alpha\r\nbeta-updated\r\ngamma\r\n");
    assert!(output.contains("\r\n"));
    Ok(())
}
