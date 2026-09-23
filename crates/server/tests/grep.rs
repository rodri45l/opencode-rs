//! Port of packages/opencode/test/tool/grep.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: ripgrep-backed search output, truncation messaging, exact
//! file paths, and the external-directory alias rule. The reference `rooted`
//! repo-root search is re-derived against a scoped temp directory.

use opencode_server::tools::{GrepArgs, GrepTool, ToolContext, ToolError};
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("opencode-grep-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn context() -> ToolContext {
    ToolContext {
        session_id: "ses_test".to_string(),
        message_id: "msg_test".to_string(),
        agent: "build".to_string(),
        ..ToolContext::default()
    }
}

fn grep() -> GrepTool {
    GrepTool::new()
}

#[test]
#[ignore = "porting: tool.grep not implemented"]
fn basic_search() -> Result<(), ToolError> {
    let dir = temp_dir("basic");
    std::fs::write(dir.join("a.ts"), "export const a = 1\n").expect("write");
    std::fs::write(dir.join("b.txt"), "no exports here\n").expect("write");

    let result = grep().execute(
        GrepArgs {
            pattern: "export".to_string(),
            path: Some(dir.to_string_lossy().into_owned()),
            include: Some("*.ts".to_string()),
        },
        &mut context(),
    )?;

    assert!(result.metadata["matches"].as_u64().unwrap_or(0) > 0);
    assert!(result.output.contains("Found"));
    Ok(())
}

#[test]
#[ignore = "porting: tool.grep not implemented"]
fn no_matches_returns_correct_output() -> Result<(), ToolError> {
    let dir = temp_dir("none");
    std::fs::write(dir.join("test.txt"), "hello world").expect("write");

    let result = grep().execute(
        GrepArgs {
            pattern: "xyznonexistentpatternxyz123".to_string(),
            path: Some(dir.to_string_lossy().into_owned()),
            include: None,
        },
        &mut context(),
    )?;

    assert_eq!(result.metadata["matches"].as_u64(), Some(0));
    assert_eq!(result.output, "No files found");
    Ok(())
}

#[test]
#[ignore = "porting: tool.grep not implemented"]
fn finds_matches_in_tmp_instance() -> Result<(), ToolError> {
    let dir = temp_dir("matches");
    std::fs::write(dir.join("test.txt"), "line1\nline2\nline3").expect("write");

    let result = grep().execute(
        GrepArgs {
            pattern: "line".to_string(),
            path: Some(dir.to_string_lossy().into_owned()),
            include: None,
        },
        &mut context(),
    )?;

    assert!(result.metadata["matches"].as_u64().unwrap_or(0) > 0);
    Ok(())
}

#[test]
#[ignore = "porting: tool.grep not implemented"]
fn does_not_report_an_unknown_total_when_results_are_truncated() -> Result<(), ToolError> {
    let dir = temp_dir("truncated");
    for index in 0..101 {
        std::fs::write(dir.join(format!("match-{index}.txt")), "needle").expect("write");
    }

    let result = grep().execute(
        GrepArgs {
            pattern: "needle".to_string(),
            path: Some(dir.to_string_lossy().into_owned()),
            include: Some("*.txt".to_string()),
        },
        &mut context(),
    )?;

    assert!(result
        .output
        .contains("(Results truncated. Consider using a more specific path or pattern.)"));
    assert!(!result.output.contains("showing 1 of 101 matches"));
    Ok(())
}

#[test]
#[ignore = "porting: tool.grep not implemented"]
fn supports_exact_file_paths() -> Result<(), ToolError> {
    let dir = temp_dir("exact");
    let file = dir.join("test.txt");
    std::fs::write(&file, "line1\nline2\nline3").expect("write");

    let result = grep().execute(
        GrepArgs {
            pattern: "line2".to_string(),
            path: Some(file.to_string_lossy().into_owned()),
            include: None,
        },
        &mut context(),
    )?;

    assert_eq!(result.metadata["matches"].as_u64(), Some(1));
    assert!(result.output.contains(&file.to_string_lossy().into_owned()));
    assert!(result.output.contains("Line 2: line2"));
    Ok(())
}

#[cfg(unix)]
#[test]
#[ignore = "porting: tool.grep not implemented"]
fn does_not_ask_for_external_directory_when_alias_path_is_allowed() -> Result<(), ToolError> {
    let base = temp_dir("alias");
    let real = base.join("real");
    let alias = base.join("alias");
    std::fs::create_dir_all(&real).expect("real dir");
    std::os::unix::fs::symlink(&real, &alias).expect("symlink");
    std::fs::write(real.join("test.txt"), "needle").expect("write");

    let mut ctx = context();
    let result = grep().execute(
        GrepArgs {
            pattern: "needle".to_string(),
            path: Some(alias.to_string_lossy().into_owned()),
            include: Some("*.txt".to_string()),
        },
        &mut ctx,
    )?;

    assert_eq!(result.metadata["matches"].as_u64(), Some(1));
    assert!(result
        .output
        .contains(&alias.join("test.txt").to_string_lossy().into_owned()));
    assert!(ctx
        .requests
        .iter()
        .all(|request| request.permission != "external_directory"));
    Ok(())
}
