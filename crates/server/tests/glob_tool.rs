//! Port of packages/opencode/test/tool/glob.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: globbing a directory returns only matching files with a
//! count, and passing a file path to the `path` argument is rejected.

use opencode_server::port::tools::{GlobArgs, GlobTool};
use opencode_server::tools::{ToolContext, ToolError};
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("opencode-glob-{name}-{}", std::process::id()));
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

#[test]
#[ignore = "porting: tool.glob not implemented"]
fn matches_files_from_a_directory_path() -> Result<(), ToolError> {
    let dir = temp_dir("matches");
    std::fs::write(dir.join("a.ts"), "export const a = 1\n").expect("seed");
    std::fs::write(dir.join("b.txt"), "hello\n").expect("seed");

    let result = GlobTool::new().execute(
        GlobArgs {
            pattern: "*.ts".to_string(),
            path: Some(dir.to_string_lossy().into_owned()),
        },
        &mut context(&dir),
    )?;

    assert_eq!(result.metadata["count"].as_u64(), Some(1));
    assert!(result
        .output
        .contains(dir.join("a.ts").to_string_lossy().as_ref()));
    assert!(!result
        .output
        .contains(dir.join("b.txt").to_string_lossy().as_ref()));
    Ok(())
}

#[test]
#[ignore = "porting: tool.glob not implemented"]
fn rejects_exact_file_paths() {
    let dir = temp_dir("file-path");
    let file = dir.join("a.ts");
    std::fs::write(&file, "export const a = 1\n").expect("seed");

    let result = GlobTool::new().execute(
        GlobArgs {
            pattern: "*.ts".to_string(),
            path: Some(file.to_string_lossy().into_owned()),
        },
        &mut context(&dir),
    );

    let error = result.expect_err("file path rejected");
    assert!(error.to_string().contains("glob path must be a directory"));
}
