//! Port of packages/opencode/test/tool/truncation.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: byte/line truncation, head/tail direction, configured
//! overrides, the managed output file, and the Grep/Task hints. The reference
//! `models-api.json` fixture cases are re-derived with generated oversized
//! content; the cleanup and fresh-process cases are dropped (no mtime API).

use opencode_server::truncate::{Direction, Truncate, TruncateLimits, TruncateOptions};

fn default_options() -> TruncateOptions {
    TruncateOptions::default()
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn truncates_large_content_by_bytes() {
    let svc = Truncate::new();
    let content = "x".repeat(60 * 1024);
    let result = svc.output(&content, default_options());

    assert!(result.truncated);
    assert!(result.content.contains("truncated..."));
    assert!(result.output_path.is_some());
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn returns_content_unchanged_when_under_limits() {
    let svc = Truncate::new();
    let content = "line1\nline2\nline3";
    let result = svc.output(content, default_options());

    assert!(!result.truncated);
    assert_eq!(result.content, content);
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn truncates_by_line_count() {
    let svc = Truncate::new();
    let lines = (0..100)
        .map(|index| format!("line{index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = svc.output(
        &lines,
        TruncateOptions {
            max_lines: Some(10),
            ..TruncateOptions::default()
        },
    );

    assert!(result.truncated);
    assert!(result.content.contains("...90 lines truncated..."));
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn truncates_by_byte_count() {
    let svc = Truncate::new();
    let content = "a".repeat(1000);
    let result = svc.output(
        &content,
        TruncateOptions {
            max_bytes: Some(100),
            ..TruncateOptions::default()
        },
    );

    assert!(result.truncated);
    assert!(result.content.contains("truncated..."));
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn truncates_from_head_by_default() {
    let svc = Truncate::new();
    let lines = (0..10)
        .map(|index| format!("line{index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = svc.output(
        &lines,
        TruncateOptions {
            max_lines: Some(3),
            ..TruncateOptions::default()
        },
    );

    assert!(result.truncated);
    assert!(result.content.contains("line0"));
    assert!(result.content.contains("line1"));
    assert!(result.content.contains("line2"));
    assert!(!result.content.contains("line9"));
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn truncates_from_tail_when_direction_is_tail() {
    let svc = Truncate::new();
    let lines = (0..10)
        .map(|index| format!("line{index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = svc.output(
        &lines,
        TruncateOptions {
            max_lines: Some(3),
            direction: Some(Direction::Tail),
            ..TruncateOptions::default()
        },
    );

    assert!(result.truncated);
    assert!(result.content.contains("line7"));
    assert!(result.content.contains("line8"));
    assert!(result.content.contains("line9"));
    assert!(!result.content.contains("line0"));
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn uses_default_max_lines_and_max_bytes() {
    assert_eq!(Truncate::MAX_LINES, 2000);
    assert_eq!(Truncate::MAX_BYTES, 50 * 1024);
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn limits_falls_back_to_defaults() {
    let svc = Truncate::new();
    let resolved = svc.limits();
    assert_eq!(resolved.max_lines, Truncate::MAX_LINES);
    assert_eq!(resolved.max_bytes, Truncate::MAX_BYTES);
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn limits_reflects_config_overrides() {
    let svc = Truncate::with_limits(TruncateLimits {
        max_lines: 123,
        max_bytes: 456,
    });
    let resolved = svc.limits();
    assert_eq!(resolved.max_lines, 123);
    assert_eq!(resolved.max_bytes, 456);
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn output_truncates_to_configured_max_lines() {
    let svc = Truncate::with_limits(TruncateLimits {
        max_lines: 10,
        max_bytes: 1024 * 1024,
    });
    let content = (0..100)
        .map(|index| format!("line{index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = svc.output(&content, default_options());

    assert!(result.truncated);
    assert!(result.content.contains("...90 lines truncated..."));
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn output_truncates_to_configured_max_bytes() {
    let svc = Truncate::with_limits(TruncateLimits {
        max_lines: 1_000_000,
        max_bytes: 100,
    });
    let content = "a".repeat(1000);
    let result = svc.output(&content, default_options());

    assert!(result.truncated);
    assert!(result.content.contains("bytes truncated..."));
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn per_call_options_override_config() {
    let svc = Truncate::with_limits(TruncateLimits {
        max_lines: 10,
        max_bytes: 100,
    });
    let content = (0..50)
        .map(|index| format!("line{index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = svc.output(
        &content,
        TruncateOptions {
            max_lines: Some(1000),
            max_bytes: Some(1024 * 1024),
            ..TruncateOptions::default()
        },
    );

    assert!(!result.truncated);
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn large_single_line_content_truncates_with_byte_message() {
    let svc = Truncate::new();
    let content = "x".repeat(60 * 1024);
    let result = svc.output(&content, default_options());

    assert!(result.truncated);
    assert!(result.content.contains("bytes truncated..."));
    assert!(content.len() > Truncate::MAX_BYTES);
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn writes_full_output_to_file_when_truncated() {
    let svc = Truncate::new();
    let lines = (0..100)
        .map(|index| format!("line{index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = svc.output(
        &lines,
        TruncateOptions {
            max_lines: Some(10),
            ..TruncateOptions::default()
        },
    );

    assert!(result.truncated);
    assert!(result
        .content
        .contains("The tool call succeeded but the output was truncated"));
    assert!(result.content.contains("Grep"));
    let output_path = result.output_path.expect("output path");
    assert!(output_path.contains("tool_"));

    let written = std::fs::read_to_string(&output_path).expect("read full output");
    assert_eq!(written, lines);
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn suggests_task_tool_when_agent_has_task_permission() {
    let svc = Truncate::new();
    let lines = (0..100)
        .map(|index| format!("line{index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = svc.output_with_task(
        &lines,
        TruncateOptions {
            max_lines: Some(10),
            ..TruncateOptions::default()
        },
        true,
    );

    assert!(result.truncated);
    assert!(result.content.contains("Grep"));
    assert!(result.content.contains("Task tool"));
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn omits_task_tool_hint_when_agent_lacks_task_permission() {
    let svc = Truncate::new();
    let lines = (0..100)
        .map(|index| format!("line{index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = svc.output_with_task(
        &lines,
        TruncateOptions {
            max_lines: Some(10),
            ..TruncateOptions::default()
        },
        false,
    );

    assert!(result.truncated);
    assert!(result.content.contains("Grep"));
    assert!(!result.content.contains("Task tool"));
}

#[test]
#[ignore = "porting: tool.truncate not implemented"]
fn does_not_write_file_when_not_truncated() {
    let svc = Truncate::new();
    let result = svc.output("short content", default_options());

    assert!(!result.truncated);
    assert!(result.output_path.is_none());
}
