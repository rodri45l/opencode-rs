//! Port of packages/core/test/session-create.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: an omitted ID creates a fresh projected Session, an exact ID
//! retry returns the original record and ignores differing create arguments,
//! supplied location/agent/model attributes are stored, a reused ID returns the
//! current projection after updates, agent and model switches publish durable
//! Session events (an unchanged or default-variant model switch publishes
//! nothing), a missing Session reports `Session.NotFoundError`, and unfinished
//! shell/skill operations report their operation name.
//! Re-derived: the Database/EventV2/projector and Effect layers are replaced by an
//! in-memory registry with an event log; the fresh-target-database replay and
//! projector-defect tests are skipped (they need the live event store).

#![allow(dead_code)]

use serde_json::{json, Value};

const NOTE: &str = "porting: session create service not implemented";

mod local {
    use serde_json::Value;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PortError {
        NotImplemented(&'static str),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Session {
        pub id: String,
        pub project_id: String,
        pub directory: String,
        pub workspace_id: Option<String>,
        pub agent: Option<String>,
        pub model: Option<Value>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum CreateError {
        NotFound,
        OperationUnavailable { operation: String },
    }

    impl CreateError {
        pub fn tag(&self) -> &'static str {
            match self {
                CreateError::NotFound => "Session.NotFoundError",
                CreateError::OperationUnavailable { .. } => "Session.OperationUnavailableError",
            }
        }

        pub fn operation(&self) -> Option<&str> {
            match self {
                CreateError::OperationUnavailable { operation } => Some(operation.as_str()),
                CreateError::NotFound => None,
            }
        }
    }

    #[derive(Debug, Default)]
    pub struct SessionRegistry {
        events: Vec<Value>,
    }

    impl SessionRegistry {
        pub fn new() -> Self {
            Self::default()
        }

        #[allow(clippy::too_many_arguments)]
        pub fn create(
            &mut self,
            _id: Option<&str>,
            _directory: &str,
            _workspace_id: Option<&str>,
            _agent: Option<&str>,
            _model: Option<Value>,
        ) -> Result<Session, PortError> {
            Err(PortError::NotImplemented("session create service"))
        }

        pub fn list(&self) -> Vec<Session> {
            Vec::new()
        }

        pub fn get(&self, _id: &str) -> Option<&Session> {
            None
        }

        pub fn update_agent(&mut self, _id: &str, _agent: &str) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session create service"))
        }

        pub fn switch_agent(&mut self, _id: &str, _agent: &str) -> Result<(), CreateError> {
            Err(CreateError::NotFound)
        }

        pub fn switch_model(&mut self, _id: &str, _model: &Value) -> Result<(), CreateError> {
            Err(CreateError::NotFound)
        }

        pub fn shell(&self, _id: &str) -> Result<(), CreateError> {
            Err(CreateError::OperationUnavailable {
                operation: "shell".to_string(),
            })
        }

        pub fn skill(&self, _id: &str) -> Result<(), CreateError> {
            Err(CreateError::OperationUnavailable {
                operation: "skill".to_string(),
            })
        }

        pub fn event_count(&self) -> usize {
            self.events.len()
        }

        pub fn last_event(&self) -> Option<&Value> {
            self.events.last()
        }
    }
}

use local::SessionRegistry;

fn model(id: &str, provider: &str) -> Value {
    json!({ "id": id, "providerID": provider })
}

#[test]
#[ignore = "porting: session create service not implemented"]
fn creates_a_fresh_projected_session_when_the_id_is_omitted() {
    let mut registry = SessionRegistry::new();
    let first = registry
        .create(None, "/project", None, None, None)
        .expect(NOTE);
    let second = registry
        .create(None, "/project", None, None, None)
        .expect(NOTE);
    assert_ne!(second.id, first.id);
    assert_eq!(registry.list().len(), 2);
}

#[test]
#[ignore = "porting: session create service not implemented"]
fn returns_the_original_session_when_the_id_is_retried() {
    let mut registry = SessionRegistry::new();
    let first = registry
        .create(Some("ses_retry"), "/project", None, None, None)
        .expect(NOTE);
    let retried = registry
        .create(Some("ses_retry"), "/project", None, None, None)
        .expect(NOTE);
    assert_eq!(retried, first);
    assert_eq!(registry.list().len(), 1);
}

#[test]
#[ignore = "porting: session create service not implemented"]
fn stores_supplied_immutable_create_attributes() {
    let mut registry = SessionRegistry::new();
    let created = registry
        .create(
            Some("ses_attrs"),
            "/project",
            Some("wrk_test"),
            Some("build"),
            Some(model("sonnet", "anthropic")),
        )
        .expect(NOTE);
    assert_eq!(created.workspace_id.as_deref(), Some("wrk_test"));
    assert_eq!(created.agent.as_deref(), Some("build"));
    assert_eq!(created.model, Some(model("sonnet", "anthropic")));
}

#[test]
#[ignore = "porting: session create service not implemented"]
fn returns_the_existing_session_when_one_id_is_reused_with_different_create_arguments() {
    let mut registry = SessionRegistry::new();
    let created = registry
        .create(Some("ses_retry"), "/project", None, None, None)
        .expect(NOTE);
    let changed = registry
        .create(
            Some("ses_retry"),
            "/other",
            None,
            Some("build"),
            Some(model("sonnet", "anthropic")),
        )
        .expect(NOTE);
    assert_eq!(changed, created);
    assert_eq!(registry.list().len(), 1);
}

#[test]
#[ignore = "porting: session create service not implemented"]
fn returns_one_recorded_session_to_concurrent_exact_retries() {
    let mut registry = SessionRegistry::new();
    let first = registry
        .create(Some("ses_retry"), "/project", None, None, None)
        .expect(NOTE);
    let second = registry
        .create(Some("ses_retry"), "/project", None, None, None)
        .expect(NOTE);
    assert_eq!(second, first);
    assert_eq!(registry.list(), vec![first]);
}

#[test]
#[ignore = "porting: session create service not implemented"]
fn returns_the_current_session_projection_after_updates() {
    let mut registry = SessionRegistry::new();
    let created = registry
        .create(Some("ses_retry"), "/project", None, None, None)
        .expect(NOTE);
    registry.update_agent("ses_retry", "build").expect(NOTE);
    let projected = registry
        .create(Some("ses_retry"), "/project", None, None, None)
        .expect(NOTE);
    assert_eq!(projected.id, created.id);
    assert_eq!(projected.agent.as_deref(), Some("build"));
}

#[test]
#[ignore = "porting: session create service not implemented"]
fn switches_the_selected_agent_through_the_durable_session_event() {
    let mut registry = SessionRegistry::new();
    let created = registry
        .create(None, "/project", None, None, None)
        .expect(NOTE);
    registry.switch_agent(&created.id, "plan").expect(NOTE);
    assert_eq!(
        registry.get(&created.id).expect(NOTE).agent.as_deref(),
        Some("plan")
    );
    assert_eq!(
        registry.last_event().expect(NOTE),
        &json!({ "type": "session.next.agent.switched", "data": { "agent": "plan" } })
    );
}

#[test]
#[ignore = "porting: session create service not implemented"]
fn rejects_an_agent_switch_for_a_missing_session() {
    let mut registry = SessionRegistry::new();
    let failure = registry
        .switch_agent("ses_missing_agent_switch", "plan")
        .expect_err(NOTE);
    assert_eq!(failure.tag(), "Session.NotFoundError");
}

#[test]
#[ignore = "porting: session create service not implemented"]
fn switches_the_selected_model_through_the_durable_session_event() {
    let mut registry = SessionRegistry::new();
    let created = registry
        .create(None, "/project", None, None, None)
        .expect(NOTE);
    let selected = model("sonnet", "anthropic");
    registry.switch_model(&created.id, &selected).expect(NOTE);
    assert_eq!(
        registry.get(&created.id).expect(NOTE).model,
        Some(selected.clone())
    );
    assert_eq!(
        registry.last_event().expect(NOTE),
        &json!({ "type": "session.next.model.switched", "data": { "model": selected } })
    );
}

#[test]
#[ignore = "porting: session create service not implemented"]
fn ignores_a_model_switch_when_the_selected_model_is_unchanged() {
    let mut registry = SessionRegistry::new();
    let created = registry
        .create(None, "/project", None, None, None)
        .expect(NOTE);
    let selected = model("sonnet", "anthropic");
    registry.switch_model(&created.id, &selected).expect(NOTE);
    registry.switch_model(&created.id, &selected).expect(NOTE);
    assert_eq!(registry.event_count(), 2);
    assert_eq!(registry.get(&created.id).expect(NOTE).model, Some(selected));
}

#[test]
#[ignore = "porting: session create service not implemented"]
fn treats_an_omitted_variant_as_the_default_variant() {
    let mut registry = SessionRegistry::new();
    let base = model("sonnet", "anthropic");
    let created = registry
        .create(None, "/project", None, None, Some(base.clone()))
        .expect(NOTE);
    let mut defaulted = base;
    defaulted["variant"] = json!("default");
    registry.switch_model(&created.id, &defaulted).expect(NOTE);
    assert_eq!(registry.event_count(), 1);
}

#[test]
#[ignore = "porting: session create service not implemented"]
fn rejects_a_model_switch_for_a_missing_session() {
    let mut registry = SessionRegistry::new();
    let failure = registry
        .switch_model("ses_missing_model_switch", &model("sonnet", "anthropic"))
        .expect_err(NOTE);
    assert_eq!(failure.tag(), "Session.NotFoundError");
}

#[test]
#[ignore = "porting: session create service not implemented"]
fn reports_unfinished_session_operations_as_unavailable() {
    let registry = SessionRegistry::new();
    assert_eq!(
        registry
            .shell("ses_unavailable")
            .expect_err(NOTE)
            .operation(),
        Some("shell")
    );
    assert_eq!(
        registry
            .skill("ses_unavailable")
            .expect_err(NOTE)
            .operation(),
        Some("skill")
    );
}
