//! Port of packages/opencode/test/tool/lsp.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the `lsp` permission metadata keeps cursor details for
//! position-based operations, drops them for `documentSymbol`, and keeps only
//! the operation for `workspaceSymbol`; the title mirrors the same shape. The
//! query-passing case observes a fake LSP client and is dropped here.

use opencode_server::port::tools::{LspArgs, LspTool};
use opencode_server::tools::{ToolContext, ToolError, ToolResult};
use serde_json::json;
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("opencode-lsp-{name}-{}", std::process::id()));
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

fn setup(name: &str) -> (PathBuf, PathBuf) {
    let dir = temp_dir(name);
    let file = dir.join("test.ts");
    std::fs::write(&file, "export const x = 1\n").expect("seed");
    (dir, file)
}

#[test]
#[ignore = "porting: tool.lsp not implemented"]
fn keeps_cursor_details_for_position_based_operations() -> Result<(), ToolError> {
    let (dir, file) = setup("position");
    let mut ctx = context(&dir);
    let result: ToolResult = LspTool::new().execute(
        LspArgs {
            operation: "goToDefinition".to_string(),
            file_path: file.to_string_lossy().into_owned(),
            line: Some(3),
            character: Some(7),
            query: None,
        },
        &mut ctx,
    )?;

    let request = ctx
        .requests
        .iter()
        .find(|request| request.permission == "lsp")
        .expect("lsp permission");
    assert_eq!(
        request.metadata,
        json!({
            "operation": "goToDefinition",
            "filePath": file.to_string_lossy(),
            "line": 3,
            "character": 7,
        })
    );
    assert_eq!(result.title, "goToDefinition test.ts:3:7");
    Ok(())
}

#[test]
#[ignore = "porting: tool.lsp not implemented"]
fn omits_cursor_details_for_document_symbol() -> Result<(), ToolError> {
    let (dir, file) = setup("document-symbol");
    let mut ctx = context(&dir);
    let result = LspTool::new().execute(
        LspArgs {
            operation: "documentSymbol".to_string(),
            file_path: file.to_string_lossy().into_owned(),
            line: Some(3),
            character: Some(7),
            query: None,
        },
        &mut ctx,
    )?;

    let request = ctx
        .requests
        .iter()
        .find(|request| request.permission == "lsp")
        .expect("lsp permission");
    assert_eq!(
        request.metadata,
        json!({
            "operation": "documentSymbol",
            "filePath": file.to_string_lossy(),
        })
    );
    assert_eq!(result.title, "documentSymbol test.ts");
    Ok(())
}

#[test]
#[ignore = "porting: tool.lsp not implemented"]
fn omits_file_and_cursor_details_for_workspace_symbol() -> Result<(), ToolError> {
    let (dir, file) = setup("workspace-symbol");
    let mut ctx = context(&dir);
    let result = LspTool::new().execute(
        LspArgs {
            operation: "workspaceSymbol".to_string(),
            file_path: file.to_string_lossy().into_owned(),
            line: Some(3),
            character: Some(7),
            query: None,
        },
        &mut ctx,
    )?;

    let request = ctx
        .requests
        .iter()
        .find(|request| request.permission == "lsp")
        .expect("lsp permission");
    assert_eq!(request.metadata, json!({ "operation": "workspaceSymbol" }));
    assert_eq!(result.title, "workspaceSymbol");
    Ok(())
}
