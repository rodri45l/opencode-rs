//! Port of packages/core/test/integration.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: integrations register through a scoped editor, a closed
//! scope reverts to the previous registration (and to absent when there was
//! none), methods register and override independently of the integration name,
//! and connections project the active credential followed by the declared env
//! names that are actually present.
//!
//! Re-derived (dropped): the `Integration`/`Credential`/`EventV2` service
//! wiring, `Scope`/`Fiber`/`Stream`/`TestClock` effects, and the OAuth attempt
//! lifecycle (code-required, cancel, background auto-complete, 10-minute
//! expiry) which is driven by the live credential store and clock.

#![allow(dead_code)]

use opencode_core::integration::{project_connections, Connection, Info, Method, Registry};

#[test]
fn registers_integrations_through_the_editor() {
    let mut registry = Registry::new().expect("registry");
    let scope = registry.open_scope().expect("scope");

    registry.update(scope, "openai", "OpenAI").expect("update");
    assert_eq!(
        registry.get("openai").expect("get"),
        Some(Info {
            id: "openai".into(),
            name: "OpenAI".into(),
            methods: vec![],
            connections: vec![],
        })
    );

    registry.close_scope(scope).expect("close");
    assert!(registry.get("openai").expect("get").is_none());
}

#[test]
fn reveals_the_previous_registration_when_an_override_closes() {
    let mut registry = Registry::new().expect("registry");
    let first = registry.open_scope().expect("scope");
    let second = registry.open_scope().expect("scope");

    registry.update(first, "openai", "OpenAI").expect("update");
    registry
        .update(second, "openai", "OpenAI Override")
        .expect("update");
    assert_eq!(
        registry
            .get("openai")
            .expect("get")
            .expect("registered")
            .name,
        "OpenAI Override"
    );

    registry.close_scope(second).expect("close");
    assert_eq!(
        registry
            .get("openai")
            .expect("get")
            .expect("registered")
            .name,
        "OpenAI"
    );
    assert_eq!(
        registry
            .list()
            .expect("list")
            .into_iter()
            .map(|info| info.id)
            .collect::<Vec<_>>(),
        vec!["openai"]
    );
}

#[test]
fn registers_and_overrides_methods_independently() {
    let method = |label: &str| Method {
        id: Some("chatgpt".into()),
        kind: "oauth",
        label: label.into(),
    };

    let mut registry = Registry::new().expect("registry");
    let first = registry.open_scope().expect("scope");
    let second = registry.open_scope().expect("scope");

    registry
        .update_method(first, "openai", method("ChatGPT"))
        .expect("method");
    registry
        .update_method(second, "openai", method("ChatGPT Override"))
        .expect("method");

    assert_eq!(
        registry
            .get("openai")
            .expect("get")
            .expect("registered")
            .methods[0]
            .label,
        "ChatGPT Override"
    );

    registry.close_scope(second).expect("close");
    assert_eq!(
        registry
            .get("openai")
            .expect("get")
            .expect("registered")
            .methods[0]
            .label,
        "ChatGPT"
    );
}

#[test]
fn projects_the_active_credential_and_present_env_names() {
    let connections = project_connections(
        Some(("cred_personal", "Personal")),
        &[
            ("INTEGRATION_TEST_ACME_KEY", true),
            ("INTEGRATION_TEST_ACME_MISSING", false),
        ],
    )
    .expect("connections");

    assert_eq!(
        connections,
        vec![
            Connection::Credential {
                id: "cred_personal".into(),
                label: "Personal".into(),
            },
            Connection::Env {
                name: "INTEGRATION_TEST_ACME_KEY".into(),
            },
        ]
    );
}
