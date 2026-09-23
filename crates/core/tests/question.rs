//! Port of packages/core/test/question.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `ask` registers a pending request with a `que_` id and
//! `list` returns it, `reply` settles the request and clears it from `list`, and
//! replying to or rejecting an unknown id returns a not-found error. Re-derived:
//! the Effect `Deferred`/fiber/event and location-scope isolation cases are
//! replaced by synchronous service calls.

use opencode_core::question::{QuestionInfo, QuestionOption, QuestionV2};

const NOTE: &str = "porting: question service not implemented";

fn question() -> QuestionInfo {
    QuestionInfo {
        question: "Which option?".into(),
        header: "Option".into(),
        options: vec![QuestionOption {
            label: "One".into(),
            description: "First option".into(),
        }],
    }
}

#[test]
#[ignore = "porting: question service not implemented"]
fn registers_a_pending_request_and_settles_a_reply() {
    let mut service = QuestionV2::new();
    let request = service
        .ask("ses_question_test", vec![question()])
        .expect(NOTE);

    assert!(request.id.starts_with("que_"));
    assert_eq!(service.list().expect(NOTE), vec![request.clone()]);

    service
        .reply(&request.id, vec![vec!["One".into()]])
        .expect(NOTE);
    assert!(service.list().expect(NOTE).is_empty());
}

#[test]
#[ignore = "porting: question service not implemented"]
fn rejects_unknown_request_ids() {
    let mut service = QuestionV2::new();
    assert!(service.reply("que_unknown", vec![]).is_err());
    assert!(service.reject("que_unknown").is_err());
}

#[test]
#[ignore = "porting: question service not implemented"]
fn rejects_a_pending_request() {
    let mut service = QuestionV2::new();
    let request = service
        .ask("ses_question_test", vec![question()])
        .expect(NOTE);
    service.reject(&request.id).expect(NOTE);
    assert!(service.list().expect(NOTE).is_empty());
}
