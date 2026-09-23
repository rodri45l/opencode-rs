//! Port of packages/tui/test/cli/tui/inline-tool-wrap-snapshot.test.tsx
//! (upstream 18ef3cc). Behaviour pinned by packages/tui/src/routes/session/index.tsx;
//! see docs/TEST-PORT.md. Snapshot/wrapping/layout assertions are visual
//! (human-verified) and omitted.
#![allow(dead_code)]

use serde_json::Value;

const TOOL_DISPLAYS: &[&str] = &[
    "bash",
    "glob",
    "read",
    "grep",
    "webfetch",
    "websearch",
    "write",
    "edit",
    "task",
    "apply_patch",
    "todowrite",
    "question",
    "skill",
    "execute",
];

fn tool_display(tool: &str) -> &str {
    if TOOL_DISPLAYS.contains(&tool) {
        tool
    } else {
        "generic"
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ApplyPatchFile {
    r#type: String,
    relative_path: String,
    file_path: String,
    patch: String,
    deletions: i64,
    move_path: Option<String>,
}

fn record(value: &Value) -> Option<&serde_json::Map<String, Value>> {
    value.as_object()
}

fn string_value(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(str::to_string)
}

fn number_value(value: Option<&Value>) -> Option<i64> {
    value
        .and_then(Value::as_f64)
        .filter(|n| n.is_finite())
        .map(|n| n as i64)
}

fn parse_apply_patch_files(value: &Value) -> Vec<ApplyPatchFile> {
    let items = match value.as_array() {
        Some(items) => items,
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    for item in items {
        let Some(file) = record(item) else { continue };
        let (Some(kind), Some(relative_path), Some(file_path)) = (
            string_value(file.get("type")),
            string_value(file.get("relativePath")),
            string_value(file.get("filePath")),
        ) else {
            continue;
        };
        let Some(patch) = string_value(file.get("patch")) else {
            continue;
        };
        let Some(deletions) = number_value(file.get("deletions")) else {
            continue;
        };
        out.push(ApplyPatchFile {
            r#type: kind,
            relative_path,
            file_path,
            patch,
            deletions,
            move_path: string_value(file.get("movePath")),
        });
    }
    out
}

#[derive(Debug, Clone, PartialEq)]
struct Todo {
    status: String,
    content: String,
}

fn parse_todos(value: &Value) -> Vec<Todo> {
    let items = match value.as_array() {
        Some(items) => items,
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    for item in items {
        let (Some(status), Some(content)) = (
            string_value(item.get("status")),
            string_value(item.get("content")),
        ) else {
            continue;
        };
        out.push(Todo { status, content });
    }
    out
}

#[derive(Debug, Clone, PartialEq)]
struct Question {
    question: String,
}

fn parse_questions(value: &Value) -> Vec<Question> {
    let items = match value.as_array() {
        Some(items) => items,
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    for item in items {
        if let Some(question) = string_value(item.get("question")) {
            out.push(Question { question });
        }
    }
    out
}

fn parse_question_answers(value: &Value) -> Option<Vec<Vec<String>>> {
    let items = value.as_array()?;
    Some(
        items
            .iter()
            .map(|answer| match answer.as_array() {
                Some(list) => list
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect(),
                None => Vec::new(),
            })
            .collect(),
    )
}

#[derive(Debug, Clone, PartialEq)]
struct Position {
    line: i64,
    character: i64,
}

#[derive(Debug, Clone, PartialEq)]
struct DiagnosticRange {
    start: Position,
}

#[derive(Debug, Clone, PartialEq)]
struct Diagnostic {
    range: DiagnosticRange,
    message: String,
}

fn parse_diagnostics(value: &Value, file_path: &str) -> Vec<Diagnostic> {
    let Some(diagnostics) = value.get(file_path).and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in diagnostics {
        let Some(diagnostic) = record(item) else {
            continue;
        };
        let start = diagnostic
            .get("range")
            .and_then(record)
            .and_then(|range| range.get("start"))
            .and_then(record);
        let line = number_value(start.and_then(|s| s.get("line")));
        let character = number_value(start.and_then(|s| s.get("character")));
        let message = string_value(diagnostic.get("message"));
        if diagnostic.get("severity").and_then(Value::as_i64) != Some(1)
            || line.is_none()
            || character.is_none()
            || message.is_none()
        {
            continue;
        }
        out.push(Diagnostic {
            range: DiagnosticRange {
                start: Position {
                    line: line.unwrap(),
                    character: character.unwrap(),
                },
            },
            message: message.unwrap(),
        });
    }
    out.truncate(3);
    out
}

fn format_subagent_toolcalls(count: i64) -> String {
    format!("{count} toolcall{}", if count == 1 { "" } else { "s" })
}

fn format_subagent_title(agent: &str, description: &str, background: bool) -> String {
    format!(
        "{agent} Task{} — {description}",
        if background { " (background)" } else { "" }
    )
}

fn format_subagent_retry(attempt: i64, message: &str) -> String {
    format!("Retrying (attempt {attempt}) · {message}")
}

fn format_completed_subagent_detail(toolcalls: i64, duration: &str) -> String {
    if toolcalls == 0 {
        return duration.to_string();
    }
    format!("{} · {duration}", format_subagent_toolcalls(toolcalls))
}

#[test]
fn falls_back_for_unknown_tool_names() {
    assert_eq!(tool_display("bash"), "bash");
    assert_eq!(tool_display("plugin_tool"), "generic");
}

#[test]
fn filters_malformed_nested_tool_wire_data() {
    use serde_json::json;

    assert_eq!(
        parse_apply_patch_files(&json!([
            null,
            { "type": "add" },
            { "type": "add", "relativePath": "a.ts", "filePath": "a.ts", "patch": "diff", "deletions": 0 }
        ])),
        vec![ApplyPatchFile {
            r#type: "add".to_string(),
            relative_path: "a.ts".to_string(),
            file_path: "a.ts".to_string(),
            patch: "diff".to_string(),
            deletions: 0,
            move_path: None,
        }]
    );
    assert_eq!(
        parse_todos(
            &json!([null, { "status": "pending" }, { "status": "pending", "content": "Safe" }])
        ),
        vec![Todo {
            status: "pending".to_string(),
            content: "Safe".to_string(),
        }]
    );
    assert_eq!(
        parse_questions(&json!([{}, { "question": 1 }, { "question": "Continue?" }])),
        vec![Question {
            question: "Continue?".to_string(),
        }]
    );
    assert_eq!(
        parse_question_answers(&json!([null, ["yes", 1], "no"])),
        Some(vec![Vec::new(), vec!["yes".to_string()], Vec::new()])
    );
    assert_eq!(parse_question_answers(&serde_json::json!({})), None);
}

#[test]
fn ignores_diagnostics_with_malformed_nested_ranges() {
    use serde_json::json;

    assert_eq!(
        parse_diagnostics(
            &json!({
                "a.ts": [
                    { "severity": 1, "message": "missing range" },
                    { "severity": 1, "message": "bad line", "range": { "start": { "line": "0", "character": 1 } } },
                    { "severity": 1, "message": "valid", "range": { "start": { "line": 2, "character": 3 } } }
                ]
            }),
            "a.ts"
        ),
        vec![Diagnostic {
            range: DiagnosticRange {
                start: Position {
                    line: 2,
                    character: 3,
                },
            },
            message: "valid".to_string(),
        }]
    );
}

#[test]
fn formats_completed_subagent_toolcall_details() {
    assert_eq!(format_completed_subagent_detail(0, "501ms"), "501ms");
    assert_eq!(
        format_completed_subagent_detail(1, "501ms"),
        "1 toolcall · 501ms"
    );
    assert_eq!(
        format_completed_subagent_detail(2, "501ms"),
        "2 toolcalls · 501ms"
    );
    assert_eq!(format_subagent_toolcalls(0), "0 toolcalls");
}

#[test]
fn keeps_background_state_attached_to_the_subagent_identity() {
    assert_eq!(
        format_subagent_title("Explore", "Inspect renderer", false),
        "Explore Task — Inspect renderer"
    );
    assert_eq!(
        format_subagent_title("Explore", "Inspect renderer", true),
        "Explore Task (background) — Inspect renderer"
    );
}

#[test]
fn keeps_retry_status_ahead_of_wrapping_messages() {
    assert_eq!(
        format_subagent_retry(2, "Rate limited by provider"),
        "Retrying (attempt 2) · Rate limited by provider"
    );
}
