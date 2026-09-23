//! Port of packages/opencode/test/acp/tool.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/acp/tool.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure tool projections `toToolKind`, `toLocations`,
//! `completedToolContent`, `pendingToolCall`, `completedToolUpdate`,
//! `completedToolRawOutput`, `extractImageAttachments`, `imageContents`, and
//! `shellOutputSnapshot`.
//! Dropped: none — every upstream case is pure. Relative workdir resolution uses
//! the platform path resolver, matching the reference comment.

use serde_json::{json, Value};
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn resolve(cwd: &str, p: &str) -> String {
    if Path::new(p).is_absolute() {
        p.to_string()
    } else {
        Path::new(cwd).join(p).to_string_lossy().into_owned()
    }
}

fn to_tool_kind(_tool: &str) -> Result<String, NotImplemented> {
    nope("acp tool")
}

fn to_locations(_tool: &str, _input: &Value, _cwd: Option<&str>) -> Result<Value, NotImplemented> {
    nope("acp tool")
}

fn completed_tool_content(_tool: &str, _state: &Value) -> Result<Value, NotImplemented> {
    nope("acp tool")
}

fn pending_tool_call(
    _tool_call_id: &str,
    _tool_name: &str,
    _input: &Value,
) -> Result<Value, NotImplemented> {
    nope("acp tool")
}

fn completed_tool_update(
    _tool_call_id: &str,
    _tool_name: &str,
    _state: &Value,
) -> Result<Value, NotImplemented> {
    nope("acp tool")
}

fn completed_tool_raw_output(_state: &Value) -> Result<Value, NotImplemented> {
    nope("acp tool")
}

fn extract_image_attachments(_attachments: &Value) -> Result<Value, NotImplemented> {
    nope("acp tool")
}

fn image_contents(_attachments: &Value) -> Result<Value, NotImplemented> {
    nope("acp tool")
}

fn shell_output_snapshot(_state: &Value) -> Result<Option<String>, NotImplemented> {
    nope("acp tool")
}

#[test]
#[ignore = "porting: acp tool not implemented"]
fn maps_opencode_tool_ids_to_acp_tool_kinds() {
    let cases = [
        ("bash", "execute"),
        ("shell", "execute"),
        ("webfetch", "fetch"),
        ("edit", "edit"),
        ("apply_patch", "edit"),
        ("patch", "edit"),
        ("write", "edit"),
        ("grep", "search"),
        ("glob", "search"),
        ("context7_resolve_library_id", "search"),
        ("context7_get_library_docs", "search"),
        ("read", "read"),
        ("task", "think"),
        ("custom_tool", "other"),
    ];
    for (tool, kind) in cases {
        assert_eq!(to_tool_kind(tool).unwrap(), kind, "tool {tool}");
    }
}

#[test]
#[ignore = "porting: acp tool not implemented"]
fn extracts_file_locations_from_tool_input() {
    assert_eq!(
        to_locations("read", &json!({ "filePath": "/tmp/a.ts" }), None).unwrap(),
        json!([{ "path": "/tmp/a.ts" }])
    );
    assert_eq!(
        to_locations("edit", &json!({ "filePath": "/tmp/b.ts" }), None).unwrap(),
        json!([{ "path": "/tmp/b.ts" }])
    );
    assert_eq!(
        to_locations("write", &json!({ "filePath": "/tmp/c.ts" }), None).unwrap(),
        json!([{ "path": "/tmp/c.ts" }])
    );
    assert_eq!(
        to_locations("grep", &json!({ "path": "/repo/src" }), None).unwrap(),
        json!([{ "path": "/repo/src" }])
    );
    assert_eq!(
        to_locations("glob", &json!({ "path": "/repo/test" }), None).unwrap(),
        json!([{ "path": "/repo/test" }])
    );
    assert_eq!(
        to_locations(
            "context7_get_library_docs",
            &json!({ "path": "/docs" }),
            None
        )
        .unwrap(),
        json!([{ "path": "/docs" }])
    );
    assert_eq!(
        to_locations(
            "external_directory",
            &json!({ "directories": ["/tmp/outside"], "patterns": ["/tmp/outside/*"] }),
            None
        )
        .unwrap(),
        json!([{ "path": "/tmp/outside" }])
    );
    assert_eq!(
        to_locations("bash", &json!({ "cmd": "pwd" }), Some("/workspace")).unwrap(),
        json!([{ "path": "/workspace" }])
    );
    assert_eq!(
        to_locations(
            "bash",
            &json!({ "command": "pwd", "workdir": "subdir" }),
            Some("/workspace")
        )
        .unwrap(),
        json!([{ "path": resolve("/workspace", "subdir") }])
    );
    assert_eq!(
        to_locations(
            "bash",
            &json!({ "command": "pwd", "workdir": "/abs/dir" }),
            Some("/workspace")
        )
        .unwrap(),
        json!([{ "path": "/abs/dir" }])
    );
    assert_eq!(
        to_locations("bash", &json!({ "command": "printf hello" }), None).unwrap(),
        json!([])
    );
    assert_eq!(
        to_locations(
            "read",
            &json!({ "path": "/tmp/missing-file-path.ts" }),
            None
        )
        .unwrap(),
        json!([])
    );
}

#[test]
#[ignore = "porting: acp tool not implemented"]
fn builds_completed_content_with_text_edit_diffs_and_image_attachments() {
    let image = "aW1hZ2UtZGF0YQ==";
    let result = completed_tool_content(
        "edit",
        &json!({
            "status": "completed",
            "input": { "filePath": "/tmp/file.ts", "oldString": "before", "newString": "after" },
            "output": "edited /tmp/file.ts",
            "attachments": [
                { "type": "file", "mime": "image/png", "filename": "image.png", "url": format!("data:image/png;base64,{image}") },
                { "type": "file", "mime": "text/plain", "filename": "note.txt", "url": "data:text/plain;base64,bm90ZQ==" }
            ]
        }),
    )
    .unwrap();
    assert_eq!(
        result,
        json!([
            { "type": "content", "content": { "type": "text", "text": "edited /tmp/file.ts" } },
            { "type": "diff", "path": "/tmp/file.ts", "oldText": "before", "newText": "after" },
            { "type": "content", "content": { "type": "image", "mimeType": "image/png", "data": image } }
        ])
    );
}

#[test]
#[ignore = "porting: acp tool not implemented"]
fn omits_edit_diffs_until_old_and_new_text_fields_exist() {
    assert_eq!(
        completed_tool_content(
            "write",
            &json!({
                "status": "completed",
                "input": { "filePath": "/tmp/file.ts", "content": "created" },
                "output": "wrote /tmp/file.ts"
            })
        )
        .unwrap(),
        json!([{ "type": "content", "content": { "type": "text", "text": "wrote /tmp/file.ts" } }])
    );
}

#[test]
#[ignore = "porting: acp tool not implemented"]
fn sends_completed_tool_calls_as_partial_updates() {
    let pending = pending_tool_call(
        "tool-1",
        "edit",
        &json!({ "filePath": "/tmp/file.ts", "oldString": "before", "newString": "after" }),
    )
    .unwrap();
    assert_eq!(pending["kind"], json!("edit"));
    assert_eq!(pending["locations"], json!([{ "path": "/tmp/file.ts" }]));
    assert_eq!(
        pending["rawInput"],
        json!({ "filePath": "/tmp/file.ts", "oldString": "before", "newString": "after" })
    );

    let completed = completed_tool_update(
        "tool-1",
        "edit",
        &json!({
            "status": "completed",
            "input": { "filePath": "/tmp/file.ts", "oldString": "before", "newString": "after" },
            "output": "Edit applied successfully."
        }),
    )
    .unwrap();
    assert_eq!(
        completed,
        json!({
            "toolCallId": "tool-1",
            "status": "completed",
            "content": [
                { "type": "content", "content": { "type": "text", "text": "Edit applied successfully." } },
                { "type": "diff", "path": "/tmp/file.ts", "oldText": "before", "newText": "after" }
            ],
            "rawOutput": { "output": "Edit applied successfully." }
        })
    );

    let titled = completed_tool_update(
        "tool-1",
        "edit",
        &json!({
            "status": "completed",
            "input": { "filePath": "/tmp/file.ts", "oldString": "before", "newString": "after" },
            "title": "file.ts",
            "output": "Edit applied successfully."
        }),
    )
    .unwrap();
    assert_eq!(titled["toolCallId"], json!("tool-1"));
    assert_eq!(titled["status"], json!("completed"));
    assert_eq!(titled["title"], json!("file.ts"));
}

#[test]
#[ignore = "porting: acp tool not implemented"]
fn uses_clean_read_display_text_for_completed_content() {
    let output = [
        "<path>/tmp/file.ts</path>",
        "<type>file</type>",
        "<content>",
        "7: first",
        "8: second",
        "",
        "(End of file - total 8 lines)",
        "</content>",
    ]
    .join("\n");
    let metadata = json!({
        "display": {
            "type": "file",
            "path": "/tmp/file.ts",
            "text": "first\nsecond",
            "lineStart": 7,
            "lineEnd": 8,
            "totalLines": 8,
            "truncated": false
        }
    });
    let state = json!({
        "status": "completed",
        "input": { "filePath": "/tmp/file.ts" },
        "output": output,
        "metadata": metadata
    });

    assert_eq!(
        completed_tool_content("read", &state).unwrap(),
        json!([{ "type": "content", "content": { "type": "text", "text": "first\nsecond" } }])
    );
    assert_eq!(
        completed_tool_raw_output(&state).unwrap(),
        json!({ "output": output, "metadata": metadata })
    );
}

#[test]
#[ignore = "porting: acp tool not implemented"]
fn builds_completed_raw_output_with_optional_metadata_and_attachments() {
    let attachments = json!([
        { "type": "file", "mime": "image/jpeg", "filename": "photo.jpg", "url": "data:image/jpeg;base64,AAAA" }
    ]);
    assert_eq!(
        completed_tool_raw_output(&json!({
            "status": "completed",
            "input": {},
            "output": "done",
            "metadata": { "exit": 0 },
            "attachments": attachments
        }))
        .unwrap(),
        json!({ "output": "done", "metadata": { "exit": 0 }, "attachments": attachments })
    );
    assert_eq!(
        completed_tool_raw_output(&json!({ "status": "completed", "input": {}, "output": "done" }))
            .unwrap(),
        json!({ "output": "done" })
    );
}

#[test]
#[ignore = "porting: acp tool not implemented"]
fn extracts_image_attachments_only_from_data_urls() {
    let attachments = json!([
        { "mime": "image/webp", "url": "data:image/webp;charset=utf-8;base64,AAAA" },
        { "mime": "image/png", "url": "https://example.com/image.png" },
        { "mime": "text/plain", "url": "data:text/plain;base64,BBBB" }
    ]);
    assert_eq!(
        extract_image_attachments(&attachments).unwrap(),
        json!([{ "mimeType": "image/webp", "data": "AAAA" }])
    );
    assert_eq!(
        image_contents(&attachments).unwrap(),
        json!([{ "type": "content", "content": { "type": "image", "mimeType": "image/webp", "data": "AAAA" } }])
    );
}

#[test]
#[ignore = "porting: acp tool not implemented"]
fn reads_shell_output_snapshot_from_string_metadata_output() {
    assert_eq!(
        shell_output_snapshot(&json!({ "metadata": { "output": "line 1\nline 2" } })).unwrap(),
        Some("line 1\nline 2".to_string())
    );
    assert_eq!(
        shell_output_snapshot(&json!({ "metadata": { "output": 42 } })).unwrap(),
        None
    );
    assert_eq!(
        shell_output_snapshot(&json!({ "metadata": null })).unwrap(),
        None
    );
}
