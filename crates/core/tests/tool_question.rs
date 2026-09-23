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
fn is_omitted_when_the_wildcard_question_action_is_denied() {
    assert_eq!(QuestionTool::NAME, "question");
    assert_eq!(
        QuestionTool::permission_action().unwrap(),
        QuestionTool::ACTION
    );
    assert_eq!(QuestionTool::permission_resources().unwrap(), vec!["*"]);
    assert!(QuestionTool::omitted_when_denied(true).unwrap());
    assert!(!QuestionTool::omitted_when_denied(false).unwrap());
    assert_eq!(
        QuestionTool::denied_message().unwrap(),
        "Permission denied: question"
    );
}

#[test]
fn projects_answers_and_marks_unanswered_questions() {
    assert_eq!(
        QuestionTool::format_answers(&questions(), &[vec!["Build".into()], vec![]]).unwrap(),
        "User has answered your questions: \"What should happen?\"=\"Build\", \"Which environment?\"=\"Unanswered\". You can now continue with the user's answers in mind."
    );
}
