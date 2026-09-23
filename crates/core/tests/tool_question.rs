//! Port of packages/core/test/tool-question.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the tool is omitted entirely when a deny rule matches the
//! wildcard `question` action, settles a denied call with
//! `Permission denied: question`, otherwise registers as `question`, asserts the
//! wildcard permission, and projects answers into the fixed sentence with
//! unanswered questions rendered as `Unanswered`. Re-derived: the `QuestionV2`/
//! `Permission`/`ToolRegistry` service wiring, captured ask inputs, and fiber
//! settlement are dropped.

use opencode_core::tool_question::{Question, QuestionTool};

const NOTE: &str = "porting: question tool not implemented";

fn questions() -> Vec<Question> {
    vec![
        Question {
            question: "What should happen?".into(),
            header: "Action".into(),
            options: vec!["Build".into()],
        },
        Question {
            question: "Which environment?".into(),
            header: "Environment".into(),
            options: vec!["Dev".into()],
        },
    ]
}

#[test]
#[ignore = "porting: question tool not implemented"]
fn is_omitted_when_the_wildcard_question_action_is_denied() {
    assert_eq!(QuestionTool::NAME, "question");
    assert_eq!(
        QuestionTool::permission_action().expect(NOTE),
        QuestionTool::ACTION
    );
    assert_eq!(QuestionTool::permission_resources().expect(NOTE), vec!["*"]);
    assert!(QuestionTool::omitted_when_denied(true).expect(NOTE));
    assert!(!QuestionTool::omitted_when_denied(false).expect(NOTE));
    assert_eq!(
        QuestionTool::denied_message().expect(NOTE),
        "Permission denied: question"
    );
}

#[test]
#[ignore = "porting: question tool not implemented"]
fn projects_answers_and_marks_unanswered_questions() {
    assert_eq!(
        QuestionTool::format_answers(&questions(), &[vec!["Build".into()], vec![]]).expect(NOTE),
        "User has answered your questions: \"What should happen?\"=\"Build\", \"Which environment?\"=\"Unanswered\". You can now continue with the user's answers in mind."
    );
}
