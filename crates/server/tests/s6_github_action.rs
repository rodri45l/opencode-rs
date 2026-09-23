//! Port of packages/opencode/test/cli/github-action.test.ts (upstream 18ef3cc).
//!
//! Only the parts of the reference suite that are pure data projection are in
//! scope: `extractResponseText` and `formatPromptTooLargeError`. RED-first: the
//! CLI github helpers are not implemented in this crate, so the reference
//! behaviour is pinned against local typed stubs.

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
enum ToolStatus {
    Completed,
    Running,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Part {
    Text(String),
    Reasoning(String),
    Tool(ToolStatus),
    StepStart,
    StepFinish,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PromptFile {
    filename: String,
    content: String,
}

fn file(filename: &str, len: usize) -> PromptFile {
    PromptFile {
        filename: filename.to_string(),
        content: "x".repeat(len),
    }
}

/// Stub for `extractResponseText`. `Err` models the reference `throw` on an
/// empty part list; `Ok(None)` models a null result that signals a summary.
fn extract_response_text(_parts: &[Part]) -> Result<Option<String>, String> {
    Ok(None)
}

/// Stub for `formatPromptTooLargeError`.
fn format_prompt_too_large_error(_files: &[PromptFile]) -> String {
    String::new()
}

const TOO_LARGE: &str = "PROMPT_TOO_LARGE: The prompt exceeds the model's context limit.";

#[test]
#[ignore = "porting: cli github extractResponseText not implemented"]
fn returns_text_from_text_part() {
    let parts = [Part::Text("Hello world".to_string())];
    assert_eq!(
        extract_response_text(&parts),
        Ok(Some("Hello world".to_string()))
    );
}

#[test]
#[ignore = "porting: cli github extractResponseText not implemented"]
fn returns_last_text_part_when_multiple_exist() {
    let parts = [
        Part::Text("First".to_string()),
        Part::Text("Last".to_string()),
    ];
    assert_eq!(extract_response_text(&parts), Ok(Some("Last".to_string())));
}

#[test]
#[ignore = "porting: cli github extractResponseText not implemented"]
fn returns_text_even_when_tool_parts_follow() {
    let parts = [
        Part::Text("I'll help with that.".to_string()),
        Part::Tool(ToolStatus::Completed),
    ];
    assert_eq!(
        extract_response_text(&parts),
        Ok(Some("I'll help with that.".to_string()))
    );
}

#[test]
#[ignore = "porting: cli github extractResponseText not implemented"]
fn returns_null_for_reasoning_only_response() {
    let parts = [Part::Reasoning("Let me think...".to_string())];
    assert_eq!(extract_response_text(&parts), Ok(None));
}

#[test]
#[ignore = "porting: cli github extractResponseText not implemented"]
fn returns_null_for_tool_only_response() {
    let parts = [Part::Tool(ToolStatus::Completed)];
    assert_eq!(extract_response_text(&parts), Ok(None));
}

#[test]
#[ignore = "porting: cli github extractResponseText not implemented"]
fn returns_null_for_multiple_completed_tools() {
    let parts = [
        Part::Tool(ToolStatus::Completed),
        Part::Tool(ToolStatus::Completed),
        Part::Tool(ToolStatus::Completed),
    ];
    assert_eq!(extract_response_text(&parts), Ok(None));
}

#[test]
#[ignore = "porting: cli github extractResponseText not implemented"]
fn returns_null_for_running_tool_parts() {
    let parts = [Part::Tool(ToolStatus::Running)];
    assert_eq!(extract_response_text(&parts), Ok(None));
}

#[test]
#[ignore = "porting: cli github extractResponseText not implemented"]
fn throws_on_empty_array() {
    assert!(extract_response_text(&[]).is_err());
}

#[test]
#[ignore = "porting: cli github extractResponseText not implemented"]
fn returns_null_for_step_start_only() {
    assert_eq!(extract_response_text(&[Part::StepStart]), Ok(None));
}

#[test]
#[ignore = "porting: cli github extractResponseText not implemented"]
fn returns_null_for_step_finish_only() {
    assert_eq!(extract_response_text(&[Part::StepFinish]), Ok(None));
}

#[test]
#[ignore = "porting: cli github extractResponseText not implemented"]
fn returns_null_for_step_start_and_finish() {
    assert_eq!(
        extract_response_text(&[Part::StepStart, Part::StepFinish]),
        Ok(None)
    );
}

#[test]
#[ignore = "porting: cli github extractResponseText not implemented"]
fn returns_text_from_multi_step_response() {
    let parts = [
        Part::StepStart,
        Part::Tool(ToolStatus::Completed),
        Part::Text("Done".to_string()),
        Part::StepFinish,
    ];
    assert_eq!(extract_response_text(&parts), Ok(Some("Done".to_string())));
}

#[test]
#[ignore = "porting: cli github extractResponseText not implemented"]
fn prefers_text_over_reasoning() {
    let parts = [
        Part::Reasoning("Internal thinking...".to_string()),
        Part::Text("Final answer".to_string()),
    ];
    assert_eq!(
        extract_response_text(&parts),
        Ok(Some("Final answer".to_string()))
    );
}

#[test]
#[ignore = "porting: cli github extractResponseText not implemented"]
fn prefers_text_over_tools() {
    let parts = [
        Part::Tool(ToolStatus::Completed),
        Part::Text("Here's what I found".to_string()),
    ];
    assert_eq!(
        extract_response_text(&parts),
        Ok(Some("Here's what I found".to_string()))
    );
}

#[test]
#[ignore = "porting: cli github formatPromptTooLargeError not implemented"]
fn formats_error_without_files() {
    assert_eq!(format_prompt_too_large_error(&[]), TOO_LARGE);
}

#[test]
#[ignore = "porting: cli github formatPromptTooLargeError not implemented"]
fn formats_error_with_base64_files() {
    let files = [
        file("screenshot.png", 400 * 1024),
        file("diagram.png", 200 * 1024),
    ];
    let result = format_prompt_too_large_error(&files);
    assert!(result.starts_with(TOO_LARGE));
    assert!(result.contains("Files in prompt:"));
    assert!(result.contains("screenshot.png (300 KB)"));
    assert!(result.contains("diagram.png (150 KB)"));
}

#[test]
#[ignore = "porting: cli github formatPromptTooLargeError not implemented"]
fn lists_all_files_when_multiple_present() {
    let files = [
        file("img1.png", 4 * 1024),
        file("img2.jpg", 8 * 1024),
        file("img3.gif", 12 * 1024),
    ];
    let result = format_prompt_too_large_error(&files);
    assert!(result.contains("img1.png (3 KB)"));
    assert!(result.contains("img2.jpg (6 KB)"));
    assert!(result.contains("img3.gif (9 KB)"));
}
