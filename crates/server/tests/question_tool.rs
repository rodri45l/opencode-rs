//! Port of packages/opencode/test/tool/question.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a successfully answered question reports its title and
//! formats the answer into the model-facing output. The reference drives the
//! interactive reply queue; this port asserts the settled tool result (the
//! reply step is the tool's own responsibility once implemented). The removed
//! zod-validation cases stay removed upstream and are not ported.

use opencode_server::port::tools::{QuestionArgs, QuestionOption, QuestionSpec, QuestionTool};
use opencode_server::tools::{ToolContext, ToolError, ToolResult};

fn context() -> ToolContext {
    ToolContext {
        session_id: "ses_test-session".to_string(),
        message_id: "msg_test-message".to_string(),
        call_id: "test-call".to_string(),
        agent: "test-agent".to_string(),
        ..ToolContext::default()
    }
}

#[test]
fn executes_with_valid_question_parameters() -> Result<(), ToolError> {
    let args = QuestionArgs {
        questions: vec![QuestionSpec {
            question: "What is your favorite color?".to_string(),
            header: "Color".to_string(),
            options: vec![
                QuestionOption {
                    label: "Red".to_string(),
                    description: "The color of passion".to_string(),
                },
                QuestionOption {
                    label: "Blue".to_string(),
                    description: "The color of sky".to_string(),
                },
            ],
            multiple: Some(false),
        }],
    };

    let result: ToolResult = QuestionTool::new().execute(args, &mut context())?;
    assert_eq!(result.title, "Asked 1 question");
    Ok(())
}

#[test]
fn passes_with_a_header_longer_than_twelve_but_less_than_thirty_chars() -> Result<(), ToolError> {
    let args = QuestionArgs {
        questions: vec![QuestionSpec {
            question: "What is your favorite animal?".to_string(),
            header: "This Header is Over 12".to_string(),
            options: vec![QuestionOption {
                label: "Dog".to_string(),
                description: "Man's best friend".to_string(),
            }],
            multiple: None,
        }],
    };

    let result = QuestionTool::new().execute(args, &mut context())?;
    assert!(result
        .output
        .contains("\"What is your favorite animal?\"=\"Dog\""));
    Ok(())
}
