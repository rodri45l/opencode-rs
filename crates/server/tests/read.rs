//! Port of packages/opencode/test/tool/read.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: reading inside the project, external-directory asks,
//! line truncation, empty-file handling, long-line truncation, image
//! attachments, binary rejection, and nearby instruction loading. The
//! Windows-only normalisation case, the live-offset streaming counter, and
//! the `models-api.json` byte-cap fixture are dropped (platform-specific or
//! need the reference fixture).

use opencode_server::port::tools::{ReadArgs, ReadTool};
use opencode_server::tools::{ToolContext, ToolError, ToolResult};
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("opencode-read-{name}-{}", std::process::id()));
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

fn read(dir: &Path, file: &Path) -> Result<ToolResult, ToolError> {
    ReadTool::new().execute(
        ReadArgs {
            file_path: file.to_string_lossy().into_owned(),
            offset: None,
            limit: None,
        },
        &mut context(dir),
    )
}

#[test]
fn allows_reading_absolute_path_inside_project_directory() -> Result<(), ToolError> {
    let dir = temp_dir("inside");
    let file = dir.join("test.txt");
    std::fs::write(&file, "hello world").expect("seed");

    let result = read(&dir, &file)?;
    assert!(result.output.contains("hello world"));
    Ok(())
}

#[test]
fn allows_reading_file_in_subdirectory_inside_project_directory() -> Result<(), ToolError> {
    let dir = temp_dir("subdir");
    let file = dir.join("subdir").join("test.txt");
    std::fs::create_dir_all(file.parent().expect("parent")).expect("dirs");
    std::fs::write(&file, "nested content").expect("seed");

    let result = read(&dir, &file)?;
    assert!(result.output.contains("nested content"));
    Ok(())
}

#[test]
fn asks_for_external_directory_permission_when_reading_absolute_path_outside_project() {
    let outer = temp_dir("outer");
    let dir = temp_dir("project");
    std::fs::write(outer.join("secret.txt"), "secret data").expect("seed");

    let mut ctx = context(&dir);
    let _ = ReadTool::new().execute(
        ReadArgs {
            file_path: outer.join("secret.txt").to_string_lossy().into_owned(),
            offset: None,
            limit: None,
        },
        &mut ctx,
    );

    let request = ctx
        .requests
        .iter()
        .find(|request| request.permission == "external_directory");
    assert!(request.is_some());
    assert!(request
        .expect("external request")
        .patterns
        .iter()
        .any(|pattern| pattern.ends_with('*')));
}

#[test]
fn does_not_ask_for_external_directory_permission_when_reading_inside_project() {
    let dir = temp_dir("internal");
    let file = dir.join("internal.txt");
    std::fs::write(&file, "internal content").expect("seed");

    let mut ctx = context(&dir);
    let _ = ReadTool::new().execute(
        ReadArgs {
            file_path: file.to_string_lossy().into_owned(),
            offset: None,
            limit: None,
        },
        &mut ctx,
    );

    assert!(ctx
        .requests
        .iter()
        .all(|request| request.permission != "external_directory"));
}

#[test]
fn truncates_by_line_count_when_limit_is_specified() -> Result<(), ToolError> {
    let dir = temp_dir("limit");
    let file = dir.join("many-lines.txt");
    let lines = (0..100)
        .map(|i| format!("line{i}"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&file, lines).expect("seed");

    let result = ReadTool::new().execute(
        ReadArgs {
            file_path: file.to_string_lossy().into_owned(),
            offset: None,
            limit: Some(10),
        },
        &mut context(&dir),
    )?;

    assert_eq!(result.metadata["truncated"].as_bool(), Some(true));
    assert!(result.output.contains("Showing lines 1-10 of 100"));
    assert!(result.output.contains("Use offset=11"));
    assert!(result.output.contains("line0"));
    assert!(result.output.contains("line9"));
    assert!(!result.output.contains("line10"));
    Ok(())
}

#[test]
fn does_not_truncate_small_file() -> Result<(), ToolError> {
    let dir = temp_dir("small");
    let file = dir.join("small.txt");
    std::fs::write(&file, "hello world").expect("seed");

    let result = read(&dir, &file)?;
    assert_eq!(result.metadata["truncated"].as_bool(), Some(false));
    assert!(result.output.contains("End of file"));
    assert_eq!(result.metadata["display"]["type"].as_str(), Some("file"));
    assert_eq!(
        result.metadata["display"]["text"].as_str(),
        Some("hello world")
    );
    assert_eq!(result.metadata["display"]["lineStart"].as_u64(), Some(1));
    assert_eq!(result.metadata["display"]["lineEnd"].as_u64(), Some(1));
    assert_eq!(result.metadata["display"]["totalLines"].as_u64(), Some(1));
    Ok(())
}

#[test]
fn respects_offset_parameter() -> Result<(), ToolError> {
    let dir = temp_dir("offset");
    let file = dir.join("offset.txt");
    let lines = (1..=20)
        .map(|i| format!("line{i}"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&file, lines).expect("seed");

    let result = ReadTool::new().execute(
        ReadArgs {
            file_path: file.to_string_lossy().into_owned(),
            offset: Some(10),
            limit: Some(5),
        },
        &mut context(&dir),
    )?;

    assert!(result.output.contains("10: line10"));
    assert!(result.output.contains("14: line14"));
    assert!(!result.output.contains("9: line10"));
    assert!(!result.output.contains("15: line15"));
    Ok(())
}

#[test]
fn throws_when_offset_is_beyond_end_of_file() {
    let dir = temp_dir("offset-beyond");
    let file = dir.join("short.txt");
    std::fs::write(&file, "line1\nline2\nline3").expect("seed");

    let result = ReadTool::new().execute(
        ReadArgs {
            file_path: file.to_string_lossy().into_owned(),
            offset: Some(4),
            limit: Some(5),
        },
        &mut context(&dir),
    );

    let error = result.expect_err("offset out of range");
    assert!(error
        .to_string()
        .contains("Offset 4 is out of range for this file (3 lines)"));
}

#[test]
fn allows_reading_empty_file_at_default_offset() -> Result<(), ToolError> {
    let dir = temp_dir("empty");
    let file = dir.join("empty.txt");
    std::fs::write(&file, "").expect("seed");

    let result = read(&dir, &file)?;
    assert_eq!(result.metadata["truncated"].as_bool(), Some(false));
    assert!(result.output.contains("End of file - total 0 lines"));
    Ok(())
}

#[test]
fn throws_when_offset_greater_than_one_for_empty_file() {
    let dir = temp_dir("empty-offset");
    let file = dir.join("empty.txt");
    std::fs::write(&file, "").expect("seed");

    let result = ReadTool::new().execute(
        ReadArgs {
            file_path: file.to_string_lossy().into_owned(),
            offset: Some(2),
            limit: None,
        },
        &mut context(&dir),
    );

    let error = result.expect_err("offset out of range");
    assert!(error
        .to_string()
        .contains("Offset 2 is out of range for this file (0 lines)"));
}

#[test]
fn truncates_long_lines() -> Result<(), ToolError> {
    let dir = temp_dir("long-line");
    let file = dir.join("long-line.txt");
    std::fs::write(&file, "x".repeat(3000)).expect("seed");

    let result = read(&dir, &file)?;
    assert!(result.output.contains("(line truncated to 2000 chars)"));
    assert!(result.output.len() < 3000);
    Ok(())
}

#[test]
fn image_files_set_truncated_to_false() -> Result<(), ToolError> {
    let dir = temp_dir("image");
    let file = dir.join("image.png");
    let png: &[u8] = &[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6,
        0, 0, 0, 31, 21, 196, 137,
    ];
    std::fs::write(&file, png).expect("seed");

    let result = read(&dir, &file)?;
    assert_eq!(result.metadata["truncated"].as_bool(), Some(false));
    let attachments = result.attachments.expect("attachments");
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].mime, "image/png");
    Ok(())
}

#[test]
fn detects_attachment_media_from_file_contents() -> Result<(), ToolError> {
    let dir = temp_dir("jpeg");
    let file = dir.join("image.bin");
    let jpeg: &[u8] = &[
        0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01,
    ];
    std::fs::write(&file, jpeg).expect("seed");

    let result = read(&dir, &file)?;
    assert_eq!(result.output, "Image read successfully");
    let attachments = result.attachments.expect("attachments");
    assert_eq!(attachments[0].mime, "image/jpeg");
    assert!(attachments[0].url.starts_with("data:image/jpeg;base64,"));
    Ok(())
}

#[test]
fn flatbuffers_files_are_read_as_text() -> Result<(), ToolError> {
    let dir = temp_dir("fbs");
    let file = dir.join("schema.fbs");
    let fbs = "namespace MyGame;\n\ntable Monster {\n  pos:Vec3;\n}\n\nroot_type Monster;";
    std::fs::write(&file, fbs).expect("seed");

    let result = read(&dir, &file)?;
    assert!(result.attachments.is_none());
    assert!(result.output.contains("namespace MyGame"));
    assert!(result.output.contains("table Monster"));
    Ok(())
}

#[test]
fn unsupported_image_mime_types_fall_through_to_text() -> Result<(), ToolError> {
    let dir = temp_dir("unsupported-image");
    for (name, content) in [
        ("image.bmp", "BM text content"),
        ("photo.tiff", "II text content"),
    ] {
        let file = dir.join(name);
        std::fs::write(&file, content).expect("seed");
        let result = read(&dir, &file)?;
        assert!(result.attachments.is_none());
        assert!(result.output.contains(content));
    }
    Ok(())
}

#[test]
fn loads_agents_md_from_parent_directory_and_includes_it_in_metadata() -> Result<(), ToolError> {
    let dir = temp_dir("instructions");
    let agents = dir.join("subdir").join("AGENTS.md");
    std::fs::create_dir_all(agents.parent().expect("parent")).expect("dirs");
    std::fs::write(&agents, "# Test Instructions\nDo something special.").expect("seed");
    let nested = dir.join("subdir").join("nested").join("test.txt");
    std::fs::create_dir_all(nested.parent().expect("parent")).expect("dirs");
    std::fs::write(&nested, "test content").expect("seed");

    let result = read(&dir, &nested)?;
    assert!(result.output.contains("test content"));
    assert!(result.output.contains("system-reminder"));
    assert!(result.output.contains("Test Instructions"));
    assert!(result.metadata["loaded"]
        .as_array()
        .expect("loaded array")
        .iter()
        .any(|value| value.as_str() == Some(agents.to_string_lossy().as_ref())));
    Ok(())
}

#[test]
fn rejects_text_extension_files_with_null_bytes() {
    let dir = temp_dir("null-byte");
    let file = dir.join("null-byte.txt");
    std::fs::write(&file, [0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0x77]).expect("seed");

    let error = read(&dir, &file).expect_err("binary file rejected");
    assert!(error.to_string().contains("Cannot read binary file"));
}

#[test]
fn rejects_known_binary_extensions() {
    let dir = temp_dir("wasm");
    let file = dir.join("module.wasm");
    std::fs::write(&file, "not really wasm").expect("seed");

    let error = read(&dir, &file).expect_err("binary file rejected");
    assert!(error.to_string().contains("Cannot read binary file"));
}
