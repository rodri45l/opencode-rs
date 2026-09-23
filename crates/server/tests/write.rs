//! Port of packages/opencode/test/tool/write.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: new-file creation, parent directories, relative path
//! resolution, overwrite + BOM preservation, content types, title generation,
//! and write failures. The formatter BOM-restore case is dropped (it shells
//! out to an external formatter).

use opencode_server::tools::{ToolContext, ToolError, WriteArgs, WriteTool};
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("opencode-write-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn context(directory: &Path) -> ToolContext {
    ToolContext {
        session_id: "ses_test-write-session".to_string(),
        message_id: "msg_test".to_string(),
        agent: "build".to_string(),
        directory: directory.to_path_buf(),
        ..ToolContext::default()
    }
}

fn run(
    directory: &Path,
    file_path: &Path,
    content: &str,
) -> Result<opencode_server::tools::ToolResult, ToolError> {
    WriteTool::new().execute(
        WriteArgs {
            file_path: file_path.to_string_lossy().into_owned(),
            content: content.to_string(),
        },
        &mut context(directory),
    )
}

#[test]
#[ignore = "porting: tool.write not implemented"]
fn writes_content_to_new_file() -> Result<(), ToolError> {
    let dir = temp_dir("new");
    let filepath = dir.join("newfile.txt");
    let result = run(&dir, &filepath, "Hello, World!")?;

    assert!(result.output.contains("Wrote file successfully"));
    assert_eq!(result.metadata["exists"].as_bool(), Some(false));
    assert_eq!(
        std::fs::read_to_string(&filepath).expect("read"),
        "Hello, World!"
    );
    Ok(())
}

#[test]
#[ignore = "porting: tool.write not implemented"]
fn creates_parent_directories_if_needed() -> Result<(), ToolError> {
    let dir = temp_dir("nested");
    let filepath = dir.join("nested").join("deep").join("file.txt");
    run(&dir, &filepath, "nested content")?;

    assert_eq!(
        std::fs::read_to_string(&filepath).expect("read"),
        "nested content"
    );
    Ok(())
}

#[test]
#[ignore = "porting: tool.write not implemented"]
fn handles_relative_paths_by_resolving_to_instance_directory() -> Result<(), ToolError> {
    let dir = temp_dir("relative");
    run(&dir, Path::new("relative.txt"), "relative content")?;

    assert_eq!(
        std::fs::read_to_string(dir.join("relative.txt")).expect("read"),
        "relative content"
    );
    Ok(())
}

#[test]
#[ignore = "porting: tool.write not implemented"]
fn overwrites_existing_file_content() -> Result<(), ToolError> {
    let dir = temp_dir("overwrite");
    let filepath = dir.join("existing.txt");
    std::fs::write(&filepath, "old content").expect("seed");
    let result = run(&dir, &filepath, "new content")?;

    assert!(result.output.contains("Wrote file successfully"));
    assert_eq!(result.metadata["exists"].as_bool(), Some(true));
    assert_eq!(
        std::fs::read_to_string(&filepath).expect("read"),
        "new content"
    );
    Ok(())
}

#[test]
#[ignore = "porting: tool.write not implemented"]
fn preserves_bom_when_overwriting_existing_files() -> Result<(), ToolError> {
    let dir = temp_dir("bom");
    let filepath = dir.join("existing.cs");
    let mut original = vec![0xEF, 0xBB, 0xBF];
    original.extend_from_slice(b"using System;\n");
    std::fs::write(&filepath, &original).expect("seed");

    run(&dir, &filepath, "using Up;\n")?;

    let content = std::fs::read(&filepath).expect("read");
    assert_eq!(&content[..3], &[0xEF, 0xBB, 0xBF]);
    assert_eq!(&content[3..], b"using Up;\n");
    Ok(())
}

#[test]
#[ignore = "porting: tool.write not implemented"]
fn returns_metadata_for_existing_files() -> Result<(), ToolError> {
    let dir = temp_dir("metadata");
    let filepath = dir.join("file.txt");
    std::fs::write(&filepath, "old").expect("seed");
    let result = run(&dir, &filepath, "new")?;

    assert_eq!(
        result.metadata["filepath"].as_str(),
        Some(filepath.to_string_lossy().as_ref())
    );
    assert_eq!(result.metadata["exists"].as_bool(), Some(true));
    Ok(())
}

#[cfg(unix)]
#[test]
#[ignore = "porting: tool.write not implemented"]
fn sets_file_permissions_when_writing_sensitive_data() -> Result<(), ToolError> {
    use std::os::unix::fs::PermissionsExt;

    let dir = temp_dir("permissions");
    let filepath = dir.join("sensitive.json");
    run(
        &dir,
        &filepath,
        &serde_json::json!({ "secret": "data" }).to_string(),
    )?;

    let mode = std::fs::metadata(&filepath)
        .expect("metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o644);
    Ok(())
}

#[test]
#[ignore = "porting: tool.write not implemented"]
fn writes_json_content() -> Result<(), ToolError> {
    let dir = temp_dir("json");
    let filepath = dir.join("data.json");
    let data = serde_json::json!({ "key": "value", "nested": { "array": [1, 2, 3] } });
    run(
        &dir,
        &filepath,
        &serde_json::to_string_pretty(&data).expect("json"),
    )?;

    let content: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&filepath).expect("read")).expect("parse");
    assert_eq!(content, data);
    Ok(())
}

#[test]
#[ignore = "porting: tool.write not implemented"]
fn writes_binary_safe_content() -> Result<(), ToolError> {
    let dir = temp_dir("binary");
    let filepath = dir.join("binary.bin");
    let content = "Hello\u{0}World\u{1}\u{2}\u{3}";
    run(&dir, &filepath, content)?;

    let read = std::fs::read_to_string(&filepath).expect("read");
    assert_eq!(read, content);
    Ok(())
}

#[test]
#[ignore = "porting: tool.write not implemented"]
fn writes_empty_content() -> Result<(), ToolError> {
    let dir = temp_dir("empty");
    let filepath = dir.join("empty.txt");
    run(&dir, &filepath, "")?;

    assert_eq!(std::fs::read_to_string(&filepath).expect("read"), "");
    assert_eq!(std::fs::metadata(&filepath).expect("metadata").len(), 0);
    Ok(())
}

#[test]
#[ignore = "porting: tool.write not implemented"]
fn writes_multi_line_content() -> Result<(), ToolError> {
    let dir = temp_dir("multiline");
    let filepath = dir.join("multiline.txt");
    let lines = ["Line 1", "Line 2", "Line 3", ""].join("\n");
    run(&dir, &filepath, &lines)?;

    assert_eq!(std::fs::read_to_string(&filepath).expect("read"), lines);
    Ok(())
}

#[test]
#[ignore = "porting: tool.write not implemented"]
fn handles_different_line_endings() -> Result<(), ToolError> {
    let dir = temp_dir("crlf");
    let filepath = dir.join("crlf.txt");
    let content = "Line 1\r\nLine 2\r\nLine 3";
    run(&dir, &filepath, content)?;

    assert_eq!(std::fs::read_to_string(&filepath).expect("read"), content);
    Ok(())
}

#[cfg(unix)]
#[test]
#[ignore = "porting: tool.write not implemented"]
fn fails_when_os_denies_write_access() {
    use std::os::unix::fs::PermissionsExt;

    let dir = temp_dir("readonly");
    let filepath = dir.join("readonly.txt");
    std::fs::write(&filepath, "test").expect("seed");
    std::fs::set_permissions(&filepath, std::fs::Permissions::from_mode(0o444)).expect("chmod");

    let result = run(&dir, &filepath, "new content");
    assert!(result.is_err());
}

#[test]
#[ignore = "porting: tool.write not implemented"]
fn returns_relative_path_as_title() -> Result<(), ToolError> {
    let dir = temp_dir("title");
    let filepath = dir.join("src").join("components").join("Button.tsx");
    std::fs::create_dir_all(filepath.parent().expect("parent")).expect("dirs");

    let result = run(&dir, &filepath, "export const Button = () => {}")?;
    let expected = PathBuf::from("src").join("components").join("Button.tsx");
    assert!(result
        .title
        .ends_with(&expected.to_string_lossy().into_owned()));
    Ok(())
}
