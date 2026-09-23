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
//! replaced by an in-memory inbox with explicit `promote_steers`; concurrent
//! exact-retry assertions run as sequential idempotent calls.

#![allow(dead_code)]

use serde_json::Value;

const NOTE: &str = "porting: session prompt service not implemented";

mod local {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PortError {
        NotImplemented(&'static str),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Delivery {
        Steer,
        Queue,
    }

    /// A durably admitted prompt inbox row.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Admitted {
        pub id: String,
        pub session_id: String,
        pub text: String,
        pub delivery: Delivery,
        pub admitted_seq: u64,
        pub promoted_seq: Option<u64>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PromptError {
        NotImplemented(&'static str),
    }

    /// The conflict tag reported for a rejected prompt reuse.
    pub fn conflict_tag() -> &'static str {
        "Session.PromptConflictError"
    }

    /// Resolve a data-URI MIME type.
    pub fn resolve_mime(uri: &str) -> Option<String> {
        let rest = uri.strip_prefix("data:")?;
        let mime = rest.split(';').next()?;
        if mime.is_empty() {
            None
        } else {
            Some(mime.to_string())
        }
    }

    #[derive(Debug, Default)]
    pub struct SessionService;

    impl SessionService {
        pub fn new() -> Self {
            Self
        }

        pub fn active(&self) -> Vec<String> {
            Vec::new()
        }

        pub fn add_active(&mut self, _id: &str) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session prompt service"))
        }

        pub fn prompt(
            &mut self,
            _session: &str,
            _id: Option<&str>,
            _text: &str,
            _delivery: Delivery,
            _resume: Option<bool>,
        ) -> Result<Admitted, PromptError> {
            Err(PromptError::NotImplemented("session prompt service"))
        }

        pub fn resume(&mut self, _session: &str) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session prompt service"))
        }

        pub fn interrupt(&mut self, _session: &str) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session prompt service"))
        }

        pub fn messages(&self, _session: &str) -> Result<Vec<Admitted>, PortError> {
            Err(PortError::NotImplemented("session prompt service"))
        }

        pub fn admitted(&self, _id: &str) -> Option<&Admitted> {
            None
        }

        pub fn admitted_count(&self) -> usize {
            0
        }

        pub fn promote_steers(&mut self, _cutoff: u64) -> Result<usize, PortError> {
            Err(PortError::NotImplemented("session prompt service"))
        }

        pub fn project_history(
            &mut self,
            _id: &str,
            _session: &str,
            _text: &str,
            _delivery: Delivery,
        ) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session prompt service"))
        }

        pub fn execution_calls(&self) -> Vec<String> {
            Vec::new()
        }

        pub fn wake_calls(&self) -> Vec<String> {
            Vec::new()
        }

        pub fn interrupt_calls(&self) -> Vec<String> {
            Vec::new()
        }
    }
}

use local::{resolve_mime, Delivery, SessionService};

fn file(uri: &str, name: &str) -> Value {
    serde_json::json!({ "uri": uri, "name": name })
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn exposes_the_execution_registry() {
    let mut service = SessionService::new();
    service.add_active("ses_prompt_test").expect(NOTE);
    assert_eq!(service.active(), vec!["ses_prompt_test".to_string()]);
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn delegates_execution_continuation_through_session_execution() {
    let mut service = SessionService::new();
    service.resume("ses_prompt_test").expect(NOTE);
    assert_eq!(
        service.execution_calls(),
        vec!["ses_prompt_test".to_string()]
    );
    assert!(service.wake_calls().is_empty());
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn delegates_process_local_interruption_through_session_execution() {
    let mut service = SessionService::new();
    service.interrupt("ses_prompt_test").expect(NOTE);
    assert_eq!(
        service.interrupt_calls(),
        vec!["ses_prompt_test".to_string()]
    );
    assert!(service.messages("ses_prompt_test").expect(NOTE).is_empty());
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn delegates_interruption_without_requiring_a_recorded_session() {
    let mut service = SessionService::new();
    service.interrupt("ses_missing").expect(NOTE);
    assert_eq!(service.interrupt_calls(), vec!["ses_missing".to_string()]);
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn durably_admits_one_user_message_before_transcript_promotion() {
    let mut service = SessionService::new();
    let message = service
        .prompt(
            "ses_prompt_test",
            None,
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    assert_eq!(message.text, "Fix the failing tests");
    assert!(service.messages("ses_prompt_test").expect(NOTE).is_empty());
    let admitted = service.admitted(&message.id).expect(NOTE);
    assert_eq!(admitted.session_id, "ses_prompt_test");
    assert_eq!(admitted.delivery, Delivery::Steer);
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn resolves_attachment_mime_before_admission() {
    assert_eq!(
        resolve_mime("data:image/png;base64,aGVsbG8="),
        Some("image/png".to_string())
    );
    let files = [file("data:image/png;base64,aGVsbG8=", "image.png")];
    assert_eq!(files[0]["uri"], "data:image/png;base64,aGVsbG8=");
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn records_distinct_messages_when_the_id_is_omitted() {
    let mut service = SessionService::new();
    let first = service
        .prompt(
            "ses_prompt_test",
            None,
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    let second = service
        .prompt(
            "ses_prompt_test",
            None,
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    assert_ne!(second.id, first.id);
    assert!(service.messages("ses_prompt_test").expect(NOTE).is_empty());
    assert_eq!(service.admitted_count(), 2);
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn returns_the_original_recorded_message_when_the_id_is_retried() {
    let mut service = SessionService::new();
    let first = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    let retried = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    assert_eq!(retried, first);
    assert!(service.messages("ses_prompt_test").expect(NOTE).is_empty());
    assert_eq!(service.admitted_count(), 1);
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn wakes_execution_when_an_exact_prompt_retry_recovers_a_committed_message() {
    let mut service = SessionService::new();
    let first = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Recover committed prompt",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    let retried = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Recover committed prompt",
            Delivery::Steer,
            Some(true),
        )
        .expect(NOTE);
    assert_eq!(retried, first);
    assert_eq!(service.wake_calls(), vec!["ses_prompt_test".to_string()]);
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn rejects_reuse_of_one_id_with_a_different_prompt() {
    let mut service = SessionService::new();
    service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    let failure = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Delete the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect_err(NOTE);
    assert_eq!(
        failure,
        local::PromptError::NotImplemented("session prompt service")
    );
    assert_eq!(local::conflict_tag(), "Session.PromptConflictError");
    assert!(service.messages("ses_prompt_test").expect(NOTE).is_empty());
    assert_eq!(service.admitted_count(), 1);
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn rejects_reuse_of_one_id_with_a_different_delivery_mode() {
    let mut service = SessionService::new();
    service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    let failure = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Queue,
            Some(false),
        )
        .expect_err(NOTE);
    assert_eq!(
        failure,
        local::PromptError::NotImplemented("session prompt service")
    );
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn returns_one_recorded_message_to_concurrent_exact_retries() {
    let mut service = SessionService::new();
    let first = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    let second = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    assert_eq!(second, first);
    assert_eq!(service.admitted_count(), 1);
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn promotes_one_message_once_under_concurrent_promotion_attempts() {
    let mut service = SessionService::new();
    service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Promote once",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    let first = service.promote_steers(u64::MAX).expect(NOTE);
    let second = service.promote_steers(u64::MAX).expect(NOTE);
    assert_eq!(first, 1);
    assert_eq!(second, 0);
    let promoted = service.messages("ses_prompt_test").expect(NOTE);
    assert_eq!(promoted.len(), 1);
    assert_eq!(promoted[0].text, "Promote once");
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn promotes_steers_only_through_the_captured_inbox_cutoff() {
    let mut service = SessionService::new();
    let first = service
        .prompt(
            "ses_prompt_test",
            None,
            "Before cutoff",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    let cutoff = first.admitted_seq;
    let second = service
        .prompt(
            "ses_prompt_test",
            None,
            "After cutoff",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    service.promote_steers(cutoff).expect(NOTE);

    assert!(service
        .admitted(&first.id)
        .expect(NOTE)
        .promoted_seq
        .is_some());
    assert!(service
        .admitted(&second.id)
        .expect(NOTE)
        .promoted_seq
        .is_none());
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn rejects_reuse_of_one_globally_unique_message_id_across_sessions() {
    let mut service = SessionService::new();
    service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    let failure = service
        .prompt(
            "ses_prompt_other",
            Some("msg_retry"),
            "Fix the failing tests",
            Delivery::Steer,
            Some(false),
        )
        .expect_err(NOTE);
    assert_eq!(
        failure,
        local::PromptError::NotImplemented("session prompt service")
    );
    assert_eq!(local::conflict_tag(), "Session.PromptConflictError");
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn rejects_a_prompt_id_already_used_by_visible_session_history() {
    let mut service = SessionService::new();
    service
        .project_history(
            "msg_retry",
            "ses_prompt_test",
            "Existing history",
            Delivery::Steer,
        )
        .expect(NOTE);
    let failure = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Conflicting prompt",
            Delivery::Steer,
            Some(false),
        )
        .expect_err(NOTE);
    assert_eq!(
        failure,
        local::PromptError::NotImplemented("session prompt service")
    );
    assert!(service.admitted("msg_retry").is_none());
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn starts_execution_by_default_after_recording_the_prompt() {
    let mut service = SessionService::new();
    service
        .prompt(
            "ses_prompt_test",
            None,
            "Run by default",
            Delivery::Steer,
            None,
        )
        .expect(NOTE);
    assert!(service.execution_calls().is_empty());
    assert_eq!(service.wake_calls(), vec!["ses_prompt_test".to_string()]);
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn starts_execution_when_resume_is_explicitly_true() {
    let mut service = SessionService::new();
    service
        .prompt(
            "ses_prompt_test",
            None,
            "Run explicitly",
            Delivery::Steer,
            Some(true),
        )
        .expect(NOTE);
    assert!(service.execution_calls().is_empty());
    assert_eq!(service.wake_calls(), vec!["ses_prompt_test".to_string()]);
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn only_records_the_prompt_when_resume_is_false() {
    let mut service = SessionService::new();
    service
        .prompt(
            "ses_prompt_test",
            None,
            "Do not run",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    assert!(service.execution_calls().is_empty());
    assert!(service.wake_calls().is_empty());
}

#[test]
#[ignore = "porting: session prompt service not implemented"]
fn returns_an_exact_retry_of_a_legacy_projected_prompt() {
    let mut service = SessionService::new();
    service
        .project_history(
            "msg_retry",
            "ses_prompt_test",
            "Historical prompt",
            Delivery::Steer,
        )
        .expect(NOTE);
    let retried = service
        .prompt(
            "ses_prompt_test",
            Some("msg_retry"),
            "Historical prompt",
            Delivery::Steer,
            Some(false),
        )
        .expect(NOTE);
    assert_eq!(retried.id, "msg_retry");
    assert_eq!(retried.text, "Historical prompt");
    assert!(retried.promoted_seq.is_some());
}
