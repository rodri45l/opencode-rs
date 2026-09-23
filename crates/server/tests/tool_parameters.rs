//! Port of packages/opencode/test/tool/parameters.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: each tool's parameter schema accepts the documented shape,
//! rejects missing or mistyped fields, and applies its defaults (webfetch
//! `format`). The reference JSON-Schema snapshots are wire-shape only and are
//! not reproduced here.

use opencode_server::port::tool_parameters::validate;
use serde_json::json;

fn parse(tool: &str, input: serde_json::Value) -> serde_json::Value {
    validate(tool, &input).expect("parameters accepted")
}

fn accepts(tool: &str, input: serde_json::Value) -> bool {
    validate(tool, &input).is_ok()
}

#[test]
fn apply_patch_accepts_patch_text_and_rejects_bad_input() {
    assert_eq!(
        parse(
            "apply_patch",
            json!({ "patchText": "*** Begin Patch\n*** End Patch" })
        ),
        json!({ "patchText": "*** Begin Patch\n*** End Patch" })
    );
    assert!(!accepts("apply_patch", json!({})));
    assert!(!accepts("apply_patch", json!({ "patchText": 123 })));
}

#[test]
fn shell_accepts_command_and_optionals() {
    assert_eq!(
        parse("shell", json!({ "command": "ls" })),
        json!({ "command": "ls" })
    );
    let parsed = parse(
        "shell",
        json!({ "command": "ls", "timeout": 5000, "workdir": "/tmp" }),
    );
    assert_eq!(parsed["timeout"], json!(5000));
    assert_eq!(parsed["workdir"], json!("/tmp"));
    assert!(!accepts("shell", json!({})));
}

#[test]
fn edit_accepts_all_fields_and_optional_replace_all() {
    assert_eq!(
        parse(
            "edit",
            json!({ "filePath": "/a", "oldString": "x", "newString": "y", "replaceAll": true })
        ),
        json!({ "filePath": "/a", "oldString": "x", "newString": "y", "replaceAll": true })
    );
    let parsed = parse(
        "edit",
        json!({ "filePath": "/a", "oldString": "x", "newString": "y" }),
    );
    assert!(parsed.get("replaceAll").is_none());
    assert!(!accepts(
        "edit",
        json!({ "oldString": "x", "newString": "y" })
    ));
}

#[test]
fn glob_accepts_pattern_and_optional_path() {
    assert_eq!(
        parse("glob", json!({ "pattern": "**/*.ts" })),
        json!({ "pattern": "**/*.ts" })
    );
    let parsed = parse("glob", json!({ "pattern": "**/*.ts", "path": "/tmp" }));
    assert_eq!(parsed["path"], json!("/tmp"));
    assert!(!accepts("glob", json!({})));
}

#[test]
fn grep_accepts_pattern_and_optionals() {
    assert_eq!(
        parse("grep", json!({ "pattern": "TODO" })),
        json!({ "pattern": "TODO" })
    );
    let parsed = parse(
        "grep",
        json!({ "pattern": "TODO", "path": "/tmp", "include": "*.ts" }),
    );
    assert_eq!(parsed["path"], json!("/tmp"));
    assert_eq!(parsed["include"], json!("*.ts"));
    assert!(!accepts("grep", json!({})));
}

#[test]
fn invalid_requires_tool_and_error() {
    assert_eq!(
        parse("invalid", json!({ "tool": "foo", "error": "bar" })),
        json!({ "tool": "foo", "error": "bar" })
    );
    assert!(!accepts("invalid", json!({ "tool": "foo" })));
    assert!(!accepts("invalid", json!({ "error": "bar" })));
}

#[test]
fn lsp_validates_operation_and_position() {
    let parsed = parse(
        "lsp",
        json!({ "operation": "hover", "filePath": "/a.ts", "line": 1, "character": 1 }),
    );
    assert_eq!(parsed["operation"], json!("hover"));
    assert!(!accepts(
        "lsp",
        json!({ "operation": "hover", "filePath": "/a.ts", "line": 0, "character": 1 })
    ));
    assert!(!accepts(
        "lsp",
        json!({ "operation": "hover", "filePath": "/a.ts", "line": 1, "character": 0 })
    ));
    assert!(!accepts(
        "lsp",
        json!({ "operation": "bogus", "filePath": "/a.ts", "line": 1, "character": 1 })
    ));
}

#[test]
fn plan_accepts_empty_object() {
    assert_eq!(parse("plan", json!({})), json!({}));
}

#[test]
fn question_requires_questions() {
    let parsed = parse(
        "question",
        json!({
            "questions": [{
                "question": "pick one",
                "header": "Header",
                "custom": false,
                "options": [{ "label": "a", "description": "desc" }]
            }]
        }),
    );
    assert_eq!(parsed["questions"].as_array().expect("questions").len(), 1);
    assert!(!accepts("question", json!({})));
}

#[test]
fn read_accepts_file_path_and_optionals() {
    assert_eq!(
        parse("read", json!({ "filePath": "/a" }))["filePath"],
        json!("/a")
    );
    let parsed = parse(
        "read",
        json!({ "filePath": "/a", "offset": 10, "limit": 100 }),
    );
    assert_eq!(parsed["offset"], json!(10));
    assert_eq!(parsed["limit"], json!(100));
}

#[test]
fn skill_requires_name() {
    assert_eq!(
        parse("skill", json!({ "name": "foo" }))["name"],
        json!("foo")
    );
    assert!(!accepts("skill", json!({})));
}

#[test]
fn task_accepts_required_fields_and_optional_background() {
    let parsed = parse(
        "task",
        json!({ "description": "d", "prompt": "p", "subagent_type": "general" }),
    );
    assert_eq!(parsed["subagent_type"], json!("general"));
    let parsed = parse(
        "task",
        json!({ "description": "d", "prompt": "p", "subagent_type": "general", "background": true }),
    );
    assert_eq!(parsed["background"], json!(true));
    assert!(!accepts(
        "task",
        json!({ "description": "d", "subagent_type": "general" })
    ));
}

#[test]
fn todo_requires_todos() {
    let parsed = parse(
        "todo",
        json!({ "todos": [{ "id": "t1", "content": "do x", "status": "pending", "priority": "medium" }] }),
    );
    assert_eq!(parsed["todos"].as_array().expect("todos").len(), 1);
    assert!(!accepts("todo", json!({})));
}

#[test]
fn webfetch_defaults_omitted_format_to_markdown() {
    assert_eq!(
        parse("webfetch", json!({ "url": "https://example.com" })),
        json!({ "url": "https://example.com", "format": "markdown" })
    );
    assert_eq!(
        parse(
            "webfetch",
            json!({ "url": "https://example.com", "format": null })
        ),
        json!({ "url": "https://example.com", "format": "markdown" })
    );
    assert!(!accepts(
        "webfetch",
        json!({ "url": "https://example.com", "format": "pdf" })
    ));
}

#[test]
fn websearch_accepts_query() {
    assert_eq!(
        parse("websearch", json!({ "query": "opencode" }))["query"],
        json!("opencode")
    );
}

#[test]
fn write_requires_content_and_file_path() {
    assert_eq!(
        parse("write", json!({ "content": "hi", "filePath": "/a" })),
        json!({ "content": "hi", "filePath": "/a" })
    );
    assert!(!accepts("write", json!({ "content": "hi" })));
}
