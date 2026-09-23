//! Port of packages/sdk-next/test/embedded.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the embedded SDK host: sessions can be created (with an
//! explicit or generated id) and switched to a model/agent; the session list,
//! active set, prompt admission, and context projections are observable; missing
//! sessions and messages report `SessionNotFoundError`/`MessageNotFoundError`;
//! events are per-host and never shared between independent hosts; and the host
//! is available as a layer service.
//! Re-derived: the live SQLite/Effect router and streaming events are replaced by
//! an in-memory host contract; timers and `Stream`/`Deferred` plumbing are
//! dropped.

#[allow(dead_code)]
mod embedded {
    use std::collections::BTreeMap;
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub enum HostError {
        SessionNotFound,
        MessageNotFound,
        NotImplemented(&'static str),
    }

    impl fmt::Display for HostError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                HostError::SessionNotFound => f.write_str("SessionNotFoundError"),
                HostError::MessageNotFound => f.write_str("MessageNotFoundError"),
                HostError::NotImplemented(note) => f.write_str(note),
            }
        }
    }

    impl std::error::Error for HostError {}

    pub type PortResult<T> = Result<T, HostError>;

    pub const NOTE: &str = "porting: sdk-next embedded host not implemented";

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ModelRef {
        pub id: String,
        pub provider_id: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Session {
        pub id: String,
        pub model: Option<ModelRef>,
        pub agent: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ContextMessage {
        pub id: String,
        pub message_type: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PromptResult {
        pub session_id: String,
        pub message_id: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Event {
        pub event_type: String,
        pub session_id: Option<String>,
        pub seq: Option<i64>,
    }

    pub struct OpenCodeHost;

    impl OpenCodeHost {
        pub fn create(&mut self) -> PortResult<Session> {
            Err(HostError::NotImplemented(NOTE))
        }

        pub fn create_with(&mut self, _id: &str) -> PortResult<Session> {
            Err(HostError::NotImplemented(NOTE))
        }

        pub fn switch_model(&mut self, _session_id: &str, _model: ModelRef) -> PortResult<()> {
            Err(HostError::NotImplemented(NOTE))
        }

        pub fn switch_agent(&mut self, _session_id: &str, _agent: &str) -> PortResult<()> {
            Err(HostError::NotImplemented(NOTE))
        }

        pub fn get(&self, _session_id: &str) -> PortResult<Session> {
            Err(HostError::NotImplemented(NOTE))
        }

        pub fn list(&self) -> PortResult<Vec<Session>> {
            Err(HostError::NotImplemented(NOTE))
        }

        pub fn active(&self) -> PortResult<BTreeMap<String, String>> {
            Err(HostError::NotImplemented(NOTE))
        }

        pub fn prompt(&mut self, _session_id: &str, _text: &str) -> PortResult<PromptResult> {
            Err(HostError::NotImplemented(NOTE))
        }

        pub fn context(&self, _session_id: &str) -> PortResult<Vec<ContextMessage>> {
            Err(HostError::NotImplemented(NOTE))
        }

        pub fn interrupt(&mut self, _session_id: &str) -> PortResult<()> {
            Err(HostError::NotImplemented(NOTE))
        }

        pub fn message(&self, _session_id: &str, _message_id: &str) -> PortResult<ContextMessage> {
            Err(HostError::NotImplemented(NOTE))
        }

        pub fn events(&self) -> PortResult<Vec<Event>> {
            Err(HostError::NotImplemented(NOTE))
        }
    }

    pub fn opencode_create() -> PortResult<OpenCodeHost> {
        Ok(OpenCodeHost)
    }

    pub fn opencode_layer_create() -> PortResult<OpenCodeHost> {
        Err(HostError::NotImplemented(NOTE))
    }
}

use embedded::{opencode_create, opencode_layer_create, Event, HostError, ModelRef, NOTE};

fn model() -> ModelRef {
    ModelRef {
        id: "embedded".to_string(),
        provider_id: "test".to_string(),
    }
}

#[test]
#[ignore = "porting: sdk-next embedded host not implemented"]
fn embedded_client_uses_the_real_router_and_handlers() {
    let mut host = opencode_create().expect(NOTE);

    let created = host.create_with("ses_embedded_1").expect(NOTE);
    assert_eq!(created.id, "ses_embedded_1");

    host.switch_model("ses_embedded_1", model()).expect(NOTE);
    let selected = host.get("ses_embedded_1").expect(NOTE);
    assert_eq!(
        selected.model.as_ref().map(|value| value.id.clone()),
        Some(model().id)
    );
    assert_eq!(
        selected
            .model
            .as_ref()
            .map(|value| value.provider_id.clone()),
        Some(model().provider_id)
    );

    let page = host.list().expect(NOTE);
    assert!(page.iter().any(|session| session.id == "ses_embedded_1"));
    assert_eq!(
        host.active().expect(NOTE),
        std::collections::BTreeMap::new()
    );

    let admitted = host.prompt("ses_embedded_1", "Do not run").expect(NOTE);
    assert_eq!(admitted.session_id, "ses_embedded_1");

    let context = host.context("ses_embedded_1").expect(NOTE);
    assert!(context
        .iter()
        .any(|message| message.message_type == "model-switched"));

    let wake = host
        .prompt("ses_embedded_1", "Promote this input")
        .expect(NOTE);
    let wake_context = host.context("ses_embedded_1").expect(NOTE);
    assert!(wake_context
        .iter()
        .any(|message| message.id == wake.message_id && message.message_type == "user"));

    let model_message = context
        .iter()
        .find(|message| message.message_type == "model-switched")
        .expect("model-switched message");
    let message = host
        .message("ses_embedded_1", &model_message.id)
        .expect(NOTE);
    assert_eq!(message, *model_message);

    host.interrupt("ses_embedded_1").expect(NOTE);

    assert_eq!(
        host.get("ses_missing").unwrap_err(),
        HostError::SessionNotFound
    );
    assert_eq!(
        host.message("ses_embedded_1", "msg_missing").unwrap_err(),
        HostError::MessageNotFound
    );
}

#[test]
#[ignore = "porting: sdk-next embedded host not implemented"]
fn location_owned_runner_events_reach_the_ready_global_client() {
    let host = opencode_create().expect(NOTE);
    let events: Vec<Event> = host.events().expect(NOTE);

    assert!(events
        .iter()
        .any(|event| event.event_type == "server.connected"));
    assert!(events.iter().any(|event| {
        event.event_type == "session.next.prompted"
            && event.session_id.as_deref() == Some("ses_embedded_events")
            && event.seq.is_some()
    }));
}

#[test]
#[ignore = "porting: sdk-next embedded host not implemented"]
fn independent_embedded_hosts_do_not_share_live_notifications() {
    let mut first = opencode_create().expect(NOTE);
    let second = opencode_create().expect(NOTE);

    first.create_with("ses_shared").expect(NOTE);
    first.switch_agent("ses_shared", "plan").expect(NOTE);

    let first_events = first.events().expect(NOTE);
    assert!(first_events
        .iter()
        .any(|event| event.event_type == "session.next.agent.switched"));

    let second_events = second.events().expect(NOTE);
    assert!(second_events
        .iter()
        .all(|event| event.event_type == "server.connected"));
}

#[test]
#[ignore = "porting: sdk-next embedded host not implemented"]
fn embedded_client_is_available_as_a_layer_service() {
    let mut host = opencode_layer_create().expect(NOTE);
    let created = host.create_with("ses_embedded_layer").expect(NOTE);
    assert_eq!(created.id, "ses_embedded_layer");
}
