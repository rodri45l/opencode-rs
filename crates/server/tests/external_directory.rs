//! Port of packages/opencode/test/tool/external-directory.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the external-directory guard no-ops for empty/inside
//! targets and bypass, and otherwise asks once with a canonical directory glob.
//! The Windows-only path normalization cases are dropped (no Windows runner).

use opencode_server::tools::{
    assert_external_directory, ExternalDirectoryOptions, ToolContext, ToolError,
};
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("opencode-extdir-{name}"));
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

fn glob(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[test]
#[ignore = "porting: tool.assertExternalDirectory not implemented"]
fn no_ops_for_empty_target() -> Result<(), ToolError> {
    let dir = temp_dir("empty");
    let mut ctx = context(&dir);
    assert_external_directory(&mut ctx, None, &ExternalDirectoryOptions::default())?;
    assert!(ctx.requests.is_empty());
    Ok(())
}

#[test]
#[ignore = "porting: tool.assertExternalDirectory not implemented"]
fn no_ops_for_paths_inside_the_instance_directory() -> Result<(), ToolError> {
    let dir = temp_dir("inside");
    let mut ctx = context(&dir);
    let target = dir.join("file.txt");
    assert_external_directory(
        &mut ctx,
        Some(target.to_string_lossy().as_ref()),
        &ExternalDirectoryOptions::default(),
    )?;
    assert!(ctx.requests.is_empty());
    Ok(())
}

#[test]
#[ignore = "porting: tool.assertExternalDirectory not implemented"]
fn asks_with_a_single_canonical_glob() -> Result<(), ToolError> {
    let base = temp_dir("outside");
    let dir = base.join("project");
    std::fs::create_dir_all(&dir).expect("project dir");
    let target = base.join("outside").join("file.txt");
    let expected = glob(&base.join("outside").join("*"));

    let mut ctx = context(&dir);
    assert_external_directory(
        &mut ctx,
        Some(target.to_string_lossy().as_ref()),
        &ExternalDirectoryOptions::default(),
    )?;

    let request = ctx
        .requests
        .iter()
        .find(|request| request.permission == "external_directory")
        .expect("external_directory request");
    assert_eq!(request.patterns, vec![expected.clone()]);
    assert_eq!(request.always, vec![expected]);
    Ok(())
}

#[test]
#[ignore = "porting: tool.assertExternalDirectory not implemented"]
fn uses_target_directory_when_kind_is_directory() -> Result<(), ToolError> {
    let base = temp_dir("dir-kind");
    let dir = base.join("project");
    std::fs::create_dir_all(&dir).expect("project dir");
    let target = base.join("outside");
    let expected = glob(&target.join("*"));

    let mut ctx = context(&dir);
    assert_external_directory(
        &mut ctx,
        Some(target.to_string_lossy().as_ref()),
        &ExternalDirectoryOptions {
            kind: Some("directory".to_string()),
            bypass: false,
        },
    )?;

    let request = ctx
        .requests
        .iter()
        .find(|request| request.permission == "external_directory")
        .expect("external_directory request");
    assert_eq!(request.patterns, vec![expected.clone()]);
    assert_eq!(request.always, vec![expected]);
    Ok(())
}

#[test]
#[ignore = "porting: tool.assertExternalDirectory not implemented"]
fn skips_prompting_when_bypass_is_true() -> Result<(), ToolError> {
    let dir = temp_dir("bypass");
    let mut ctx = context(&dir);
    assert_external_directory(
        &mut ctx,
        Some("/tmp/outside/file.txt"),
        &ExternalDirectoryOptions {
            kind: None,
            bypass: true,
        },
    )?;
    assert!(ctx.requests.is_empty());
    Ok(())
}
