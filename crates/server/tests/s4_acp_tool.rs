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

#[allow(dead_code)]
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

fn to_tool_kind(tool: &str) -> Result<String, NotImplemented> {
    Ok(match tool {
        "bash" | "shell" => "execute",
        "webfetch" => "fetch",
        "edit" | "apply_patch" | "patch" | "write" => "edit",
        "read" => "read",
        "task" => "think",
        _ if tool == "grep" || tool == "glob" || tool.starts_with("context7_") => "search",
        _ => "other",
    }
    .to_string())
}

fn to_locations(tool: &str, input: &Value, cwd: Option<&str>) -> Result<Value, NotImplemented> {
    let string = |key: &str| input.get(key).and_then(Value::as_str);
    let single = |path: &str| json!([{ "path": path }]);
    Ok(match tool {
        "read" | "edit" | "write" => string("filePath").map(single).unwrap_or_else(|| json!([])),
        "grep" | "glob" | "context7_get_library_docs" => {
            string("path").map(single).unwrap_or_else(|| json!([]))
        }
        "external_directory" => input
            .get("directories")
            .and_then(Value::as_array)
            .and_then(|dirs| dirs.first())
            .and_then(Value::as_str)
            .map(single)
            .unwrap_or_else(|| json!([])),
        "bash" | "shell" => {
            if let Some(workdir) = string("workdir") {
                single(&resolve(cwd.unwrap_or(""), workdir))
            } else if let Some(cwd) = cwd {
                single(cwd)
            } else {
                json!([])
            }
        }
        _ => json!([]),
    })
}

fn image_attachments(attachments: &Value) -> Vec<Value> {
    attachments
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let mime = item.get("mime").and_then(Value::as_str)?;
                    let url = item.get("url").and_then(Value::as_str)?;
                    if !mime.starts_with("image/") || !url.starts_with("data:") {
                        return None;
                    }
                    let data = url.split(";base64,").nth(1)?;
                    Some(json!({ "mimeType": mime, "data": data }))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn completed_tool_content(tool: &str, state: &Value) -> Result<Value, NotImplemented> {
    let mut blocks = Vec::new();
    let display = state
        .pointer("/metadata/display/text")
        .and_then(Value::as_str)
        .filter(|_| tool == "read")
        .map(str::to_string);
    let text = display.or_else(|| {
        state
            .get("output")
            .and_then(Value::as_str)
            .map(str::to_string)
    });
    if let Some(text) = text {
        blocks.push(json!({ "type": "content", "content": { "type": "text", "text": text } }));
    }
    let input = &state["input"];
    if let (Some(path), Some(old), Some(new)) = (
        input.get("filePath").and_then(Value::as_str),
        input.get("oldString").and_then(Value::as_str),
        input.get("newString").and_then(Value::as_str),
    ) {
        blocks.push(json!({ "type": "diff", "path": path, "oldText": old, "newText": new }));
    }
    for image in image_attachments(&state["attachments"]) {
        let mut content = image;
        content["type"] = json!("image");
        blocks.push(json!({ "type": "content", "content": content }));
    }
    Ok(Value::Array(blocks))
}

fn pending_tool_call(
    tool_call_id: &str,
    tool_name: &str,
    input: &Value,
) -> Result<Value, NotImplemented> {
    Ok(json!({
        "toolCallId": tool_call_id,
        "kind": to_tool_kind(tool_name)?,
        "status": "pending",
        "rawInput": input,
        "locations": to_locations(tool_name, input, None)?,
    }))
}

fn completed_tool_raw_output(state: &Value) -> Result<Value, NotImplemented> {
    let mut output = serde_json::Map::new();
    output.insert(
        "output".to_string(),
        state.get("output").cloned().unwrap_or(Value::Null),
    );
    if let Some(metadata) = state.get("metadata").filter(|value| !value.is_null()) {
        output.insert("metadata".to_string(), metadata.clone());
    }
    if let Some(attachments) = state.get("attachments").filter(|value| !value.is_null()) {
        output.insert("attachments".to_string(), attachments.clone());
    }
    Ok(Value::Object(output))
}

fn completed_tool_update(
    tool_call_id: &str,
    tool_name: &str,
    state: &Value,
) -> Result<Value, NotImplemented> {
    let mut update = serde_json::Map::new();
    update.insert("toolCallId".to_string(), json!(tool_call_id));
    update.insert("status".to_string(), json!("completed"));
    update.insert(
        "content".to_string(),
        completed_tool_content(tool_name, state)?,
    );
    update.insert("rawOutput".to_string(), completed_tool_raw_output(state)?);
    if let Some(title) = state.get("title") {
        update.insert("title".to_string(), title.clone());
    }
    Ok(Value::Object(update))
}

fn extract_image_attachments(attachments: &Value) -> Result<Value, NotImplemented> {
    Ok(Value::Array(image_attachments(attachments)))
}

fn image_contents(attachments: &Value) -> Result<Value, NotImplemented> {
    let contents: Vec<Value> = image_attachments(attachments)
        .into_iter()
        .map(|image| {
            let mut content = image;
            content["type"] = json!("image");
            json!({ "type": "content", "content": content })
        })
        .collect();
    Ok(Value::Array(contents))
}

fn shell_output_snapshot(state: &Value) -> Result<Option<String>, NotImplemented> {
    Ok(state
        .get("metadata")
        .and_then(|metadata| metadata.get("output"))
        .and_then(Value::as_str)
        .map(str::to_string))
}

#[test]
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
