//! Port of packages/core/test/session-prompt.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the execution registry and delegation (resume/interrupt)
//! surface through `SessionExecution`, a prompt durably admits one inbox message
//! before promotion, attachment MIME is resolved before admission, exact ID
//! retries return the original record while conflicting text/delivery/session
//! reuse fails with `Session.PromptConflictError`, a prompt ID already used by
//! visible history is rejected, steers promote once through the captured inbox
//! cutoff, and resume defaults to true (wake) unless explicitly false.
//! Re-derived: the Database/EventV2/projector and Effect-concurrency wiring are
//! replaced by an in-memory inbox in `opencode_core::session_prompt`.

use opencode_core::session_prompt::{conflict_tag, resolve_mime, Delivery, SessionPromptService};
use serde_json::Value;

fn file(uri: &str, name: &str) -> Value {
    serde_json::json!({ "uri": uri, "name": name })
}

#[test]
fn exposes_the_execution_registry() {
    let mut service = SessionPromptService::new();
    service.add_active("ses_prompt_test");
    assert_eq!(service.active(), vec!["ses_prompt_test".to_string()]);
}

#[test]
fn delegates_execution_continuation_through_session_execution() {
    let mut service = SessionPromptService::new();
    service.resume("ses_prompt_test");
    assert_eq!(
        service.execution_calls(),
        vec!["ses_prompt_test".to_string()]
    );
    assert!(service.wake_calls().is_empty());
}

#[test]
fn delegates_process_local_interruption_through_session_execution() {
    let mut service = SessionPromptService::new();
    service.interrupt("ses_prompt_test");
    assert_eq!(
        service.interrupt_calls(),
        vec!["ses_prompt_test".to_string()]
    );
    assert!(service.messages("ses_prompt_test").is_empty());
}

#[test]
fn delegates_interruption_without_requiring_a_recorded_session() {
    let mut service = SessionPromptService::new();
    service.interrupt("ses_missing");
    assert_eq!(service.interrupt_calls(), vec!["ses_missing".to_string()]);
}

#[test]
fn durably_admits_one_user_message_before_transcript_promotion() {
    let mut service = SessionPromptService::new();
    let message = service
        .prompt(
            "ses_prompt_test",
            None,
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    assert_eq!(message.text, "Fix the failing tests");
    assert!(service.messages("ses_prompt_test").is_empty());
    let admitted = service.admitted(&message.id).expect("admitted");
    assert_eq!(admitted.session_id, "ses_prompt_test");
    assert_eq!(admitted.delivery, Delivery::Steer);
}

#[test]
fn resolves_attachment_mime_before_admission() {
    assert_eq!(
        resolve_mime("data:image/png;base64,aGVsbG8="),
        Some("image/png".to_string())
    );
    let files = [file("data:image/png;base64,aGVsbG8=", "image.png")];
    assert_eq!(files[0]["uri"], "data:image/png;base64,aGVsbG8=");
}

#[test]
fn records_distinct_messages_when_the_id_is_omitted() {
    let mut service = SessionPromptService::new();
    let first = service
        .prompt(
            "ses_prompt_test",
            None,
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    let second = service
        .prompt(
            "ses_prompt_test",
            None,
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    assert_ne!(second.id, first.id);
    assert!(service.messages("ses_prompt_test").is_empty());
    assert_eq!(service.admitted_count(), 2);
}

#[test]
fn returns_the_original_recorded_message_when_the_id_is_retried() {
    let mut service = SessionPromptService::new();
    let first = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    let retried = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    assert_eq!(retried, first);
    assert!(service.messages("ses_prompt_test").is_empty());
    assert_eq!(service.admitted_count(), 1);
}

#[test]
fn wakes_execution_when_an_exact_prompt_retry_recovers_a_committed_message() {
    let mut service = SessionPromptService::new();
    let first = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Recover committed prompt",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    let retried = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Recover committed prompt",
            Delivery::Steer,
            Some(true),
        )
        .expect("prompt");
    assert_eq!(retried, first);
    assert_eq!(service.wake_calls(), vec!["ses_prompt_test".to_string()]);
}

#[test]
fn rejects_reuse_of_one_id_with_a_different_prompt() {
    let mut service = SessionPromptService::new();
    service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    let failure = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Delete the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect_err("conflict");
    assert_eq!(failure.tag(), conflict_tag());
    assert_eq!(conflict_tag(), "Session.PromptConflictError");
    assert!(service.messages("ses_prompt_test").is_empty());
    assert_eq!(service.admitted_count(), 1);
}

#[test]
fn rejects_reuse_of_one_id_with_a_different_delivery_mode() {
    let mut service = SessionPromptService::new();
    service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    let failure = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Queue,
            Some(false),
        )
        .expect_err("conflict");
    assert_eq!(failure.tag(), conflict_tag());
}

#[test]
fn returns_one_recorded_message_to_concurrent_exact_retries() {
    let mut service = SessionPromptService::new();
    let first = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    let second = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    assert_eq!(second, first);
    assert_eq!(service.admitted_count(), 1);
}

#[test]
fn promotes_one_message_once_under_concurrent_promotion_attempts() {
    let mut service = SessionPromptService::new();
    service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Promote once",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    let first = service.promote_steers(u64::MAX);
    let second = service.promote_steers(u64::MAX);
    assert_eq!(first, 1);
    assert_eq!(second, 0);
    let promoted = service.messages("ses_prompt_test");
    assert_eq!(promoted.len(), 1);
    assert_eq!(promoted[0].text, "Promote once");
}

#[test]
fn promotes_steers_only_through_the_captured_inbox_cutoff() {
    let mut service = SessionPromptService::new();
    let first = service
        .prompt(
            "ses_prompt_test",
            None,
            "Before cutoff",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    let cutoff = first.admitted_seq;
    let second = service
        .prompt(
            "ses_prompt_test",
            None,
            "After cutoff",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    service.promote_steers(cutoff);

    assert!(service
        .admitted(&first.id)
        .expect("admitted")
        .promoted_seq
        .is_some());
    assert!(service
        .admitted(&second.id)
        .expect("admitted")
        .promoted_seq
        .is_none());
}

#[test]
fn rejects_reuse_of_one_globally_unique_message_id_across_sessions() {
    let mut service = SessionPromptService::new();
    service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    let failure = service
        .prompt(
            "ses_prompt_other",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect_err("conflict");
    assert_eq!(failure.tag(), conflict_tag());
    assert_eq!(conflict_tag(), "Session.PromptConflictError");
}

#[test]
fn rejects_a_prompt_id_already_used_by_visible_session_history() {
    let mut service = SessionPromptService::new();
    service.project_history(
        "msg_retry",
        "ses_prompt_test",
        "Existing history",
        Delivery::Steer,
    );
    let failure = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Conflicting prompt",
            Delivery::Steer,
            Some(false),
        )
        .expect_err("conflict");
    assert_eq!(failure.tag(), conflict_tag());
    assert!(service.admitted("msg_retry").is_none());
}

#[test]
fn starts_execution_by_default_after_recording_the_prompt() {
    let mut service = SessionPromptService::new();
    service
        .prompt(
            "ses_prompt_test",
            None,
            "Run by default",
            Delivery::Steer,
            None,
        )
        .expect("prompt");
    assert!(service.execution_calls().is_empty());
    assert_eq!(service.wake_calls(), vec!["ses_prompt_test".to_string()]);
}

#[test]
fn starts_execution_when_resume_is_explicitly_true() {
    let mut service = SessionPromptService::new();
    service
        .prompt(
            "ses_prompt_test",
            None,
            "Run explicitly",
            Delivery::Steer,
            Some(true),
        )
        .expect("prompt");
    assert!(service.execution_calls().is_empty());
    assert_eq!(service.wake_calls(), vec!["ses_prompt_test".to_string()]);
}

#[test]
fn only_records_the_prompt_when_resume_is_false() {
    let mut service = SessionPromptService::new();
    service
        .prompt(
            "ses_prompt_test",
            None,
            "Do not run",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    assert!(service.execution_calls().is_empty());
    assert!(service.wake_calls().is_empty());
}

#[test]
fn returns_an_exact_retry_of_a_legacy_projected_prompt() {
    let mut service = SessionPromptService::new();
    service.project_history(
        "msg_retry",
        "ses_prompt_test",
        "Historical prompt",
        Delivery::Steer,
    );
    let retried = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Historical prompt",
            Delivery::Steer,
            Some(false),
        )
        .expect("prompt");
    assert_eq!(retried.id, "msg_retry");
    assert_eq!(retried.text, "Historical prompt");
    assert!(retried.promoted_seq.is_some());
}
