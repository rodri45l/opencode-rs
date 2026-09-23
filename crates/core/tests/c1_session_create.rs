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
//! in-memory registry in `opencode_core::session_registry`; the fresh-target-
//! database replay and projector-defect tests are skipped.

use opencode_core::session_registry::SessionRegistry;
use serde_json::{json, Value};

fn model(id: &str, provider: &str) -> Value {
    json!({ "id": id, "providerID": provider })
}

#[test]
fn creates_a_fresh_projected_session_when_the_id_is_omitted() {
    let mut registry = SessionRegistry::new();
    let first = registry.create(None, "/project", None, None, None);
    let second = registry.create(None, "/project", None, None, None);
    assert_ne!(second.id, first.id);
    assert_eq!(registry.list().len(), 2);
}

#[test]
fn returns_the_original_session_when_the_id_is_retried() {
    let mut registry = SessionRegistry::new();
    let first = registry.create(Some("ses_retry"), "/project", None, None, None);
    let retried = registry.create(Some("ses_retry"), "/project", None, None, None);
    assert_eq!(retried, first);
    assert_eq!(registry.list().len(), 1);
}

#[test]
fn stores_supplied_immutable_create_attributes() {
    let mut registry = SessionRegistry::new();
    let created = registry.create(
        Some("ses_attrs"),
        "/project",
        Some("wrk_test"),
        Some("build"),
        Some(model("sonnet", "anthropic")),
    );
    assert_eq!(created.workspace_id.as_deref(), Some("wrk_test"));
    assert_eq!(created.agent.as_deref(), Some("build"));
    assert_eq!(created.model, Some(model("sonnet", "anthropic")));
}

#[test]
fn returns_the_existing_session_when_one_id_is_reused_with_different_create_arguments() {
    let mut registry = SessionRegistry::new();
    let created = registry.create(Some("ses_retry"), "/project", None, None, None);
    let changed = registry.create(
        Some("ses_retry"),
        "/other",
        None,
        Some("build"),
        Some(model("sonnet", "anthropic")),
    );
    assert_eq!(changed, created);
    assert_eq!(registry.list().len(), 1);
}

#[test]
fn returns_one_recorded_session_to_concurrent_exact_retries() {
    let mut registry = SessionRegistry::new();
    let first = registry.create(Some("ses_retry"), "/project", None, None, None);
    let second = registry.create(Some("ses_retry"), "/project", None, None, None);
    assert_eq!(second, first);
    assert_eq!(registry.list(), vec![first]);
}

#[test]
fn returns_the_current_session_projection_after_updates() {
    let mut registry = SessionRegistry::new();
    let created = registry.create(Some("ses_retry"), "/project", None, None, None);
    registry
        .update_agent("ses_retry", "build")
        .expect("update agent");
    let projected = registry.create(Some("ses_retry"), "/project", None, None, None);
    assert_eq!(projected.id, created.id);
    assert_eq!(projected.agent.as_deref(), Some("build"));
}

#[test]
fn switches_the_selected_agent_through_the_durable_session_event() {
    let mut registry = SessionRegistry::new();
    let created = registry.create(None, "/project", None, None, None);
    registry
        .switch_agent(&created.id, "plan")
        .expect("switch agent");
    assert_eq!(
        registry.get(&created.id).expect("get").agent.as_deref(),
        Some("plan")
    );
    assert_eq!(
        registry.last_event().expect("event"),
        &json!({ "type": "session.next.agent.switched", "data": { "agent": "plan" } })
    );
}

#[test]
fn rejects_an_agent_switch_for_a_missing_session() {
    let mut registry = SessionRegistry::new();
    let failure = registry
        .switch_agent("ses_missing_agent_switch", "plan")
        .expect_err("missing");
    assert_eq!(failure.tag(), "Session.NotFoundError");
}

#[test]
fn switches_the_selected_model_through_the_durable_session_event() {
    let mut registry = SessionRegistry::new();
    let created = registry.create(None, "/project", None, None, None);
    let selected = model("sonnet", "anthropic");
    registry
        .switch_model(&created.id, &selected)
        .expect("switch model");
    assert_eq!(
        registry.get(&created.id).expect("get").model,
        Some(selected.clone())
    );
    assert_eq!(
        registry.last_event().expect("event"),
        &json!({ "type": "session.next.model.switched", "data": { "model": selected } })
    );
}

#[test]
fn ignores_a_model_switch_when_the_selected_model_is_unchanged() {
    let mut registry = SessionRegistry::new();
    let created = registry.create(None, "/project", None, None, None);
    let selected = model("sonnet", "anthropic");
    registry
        .switch_model(&created.id, &selected)
        .expect("switch model");
    registry
        .switch_model(&created.id, &selected)
        .expect("switch model");
    assert_eq!(registry.event_count(), 2);
    assert_eq!(
        registry.get(&created.id).expect("get").model,
        Some(selected)
    );
}

#[test]
fn treats_an_omitted_variant_as_the_default_variant() {
    let mut registry = SessionRegistry::new();
    let base = model("sonnet", "anthropic");
    let created = registry.create(None, "/project", None, None, Some(base.clone()));
    let mut defaulted = base;
    defaulted["variant"] = json!("default");
    registry
        .switch_model(&created.id, &defaulted)
        .expect("switch model");
    assert_eq!(registry.event_count(), 1);
}

#[test]
fn rejects_a_model_switch_for_a_missing_session() {
    let mut registry = SessionRegistry::new();
    let failure = registry
        .switch_model("ses_missing_model_switch", &model("sonnet", "anthropic"))
        .expect_err("missing");
    assert_eq!(failure.tag(), "Session.NotFoundError");
}

#[test]
fn reports_unfinished_session_operations_as_unavailable() {
    let registry = SessionRegistry::new();
    assert_eq!(
        registry
            .shell("ses_unavailable")
            .expect_err("shell")
            .operation(),
        Some("shell")
    );
    assert_eq!(
        registry
            .skill("ses_unavailable")
            .expect_err("skill")
            .operation(),
        Some("skill")
    );
}
