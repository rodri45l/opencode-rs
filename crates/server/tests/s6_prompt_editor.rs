//! Port of packages/opencode/test/cli/run/prompt.editor.test.ts (upstream 18ef3cc).
//!
//! RED-first: the run external-editor helpers in `cli/cmd/run/prompt.editor` are
//! not implemented in this crate. Reference behaviour is pinned against local
//! typed stubs.

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
enum RunPromptPart {
    File {
        mime: String,
        filename: String,
        url: String,
        path: String,
        text_start: usize,
        text_end: usize,
        text_value: String,
    },
    Agent {
        name: String,
        source_start: usize,
        source_end: usize,
        source_value: String,
    },
}

fn file_part() -> RunPromptPart {
    RunPromptPart::File {
        mime: "text/plain".to_string(),
        filename: "src/app.ts".to_string(),
        url: "file:///src/app.ts".to_string(),
        path: "src/app.ts".to_string(),
        text_start: 0,
        text_end: 11,
        text_value: "@src/app.ts".to_string(),
    }
}

fn agent_part() -> RunPromptPart {
    RunPromptPart::Agent {
        name: "helper".to_string(),
        source_start: 12,
        source_end: 19,
        source_value: "@helper".to_string(),
    }
}

fn resolve_editor_slash_value(_value: &str) -> String {
    String::new()
}

fn realign_editor_prompt_parts(_text: &str, _parts: &[RunPromptPart]) -> Vec<RunPromptPart> {
    Vec::new()
}

#[test]
#[ignore = "porting: cli run prompt editor not implemented"]
fn strips_the_local_editor_command_from_the_initial_editor_text() {
    assert_eq!(resolve_editor_slash_value("/editor"), "");
    assert_eq!(
        resolve_editor_slash_value("/editor draft message"),
        "draft message"
    );
    assert_eq!(
        resolve_editor_slash_value("/editor first line\nsecond line"),
        "first line\nsecond line"
    );
}

#[test]
#[ignore = "porting: cli run prompt editor not implemented"]
fn realigns_file_and_agent_parts_after_external_editing() {
    let parts = vec![file_part(), agent_part()];
    let expected = vec![
        RunPromptPart::File {
            mime: "text/plain".to_string(),
            filename: "src/app.ts".to_string(),
            url: "file:///src/app.ts".to_string(),
            path: "src/app.ts".to_string(),
            text_start: 28,
            text_end: 39,
            text_value: "@src/app.ts".to_string(),
        },
        RunPromptPart::Agent {
            name: "helper".to_string(),
            source_start: 13,
            source_end: 20,
            source_value: "@helper".to_string(),
        },
    ];
    assert_eq!(
        realign_editor_prompt_parts("Please check @helper before @src/app.ts", &parts),
        expected
    );
}

#[test]
#[ignore = "porting: cli run prompt editor not implemented"]
fn drops_parts_whose_virtual_text_was_deleted() {
    let parts = vec![file_part(), agent_part()];
    let expected = vec![RunPromptPart::Agent {
        name: "helper".to_string(),
        source_start: 5,
        source_end: 12,
        source_value: "@helper".to_string(),
    }];
    assert_eq!(
        realign_editor_prompt_parts("Only @helper remains", &parts),
        expected
    );
}
