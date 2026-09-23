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

const NOTE: &str = "porting: integration registry not implemented";

#[derive(Debug, PartialEq, Eq)]
enum IntegrationError {
    NotImplemented,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Method {
    id: Option<String>,
    kind: &'static str,
    label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Connection {
    Credential { id: String, label: String },
    Env { name: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Info {
    id: String,
    name: String,
    methods: Vec<Method>,
    connections: Vec<Connection>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ScopeId(u64);

#[derive(Default)]
struct Registry;

impl Registry {
    fn new() -> Result<Self, IntegrationError> {
        Err(IntegrationError::NotImplemented)
    }

    fn open_scope(&mut self) -> Result<ScopeId, IntegrationError> {
        Err(IntegrationError::NotImplemented)
    }

    fn close_scope(&mut self, _scope: ScopeId) -> Result<(), IntegrationError> {
        Err(IntegrationError::NotImplemented)
    }

    fn update(&mut self, _scope: ScopeId, _id: &str, _name: &str) -> Result<(), IntegrationError> {
        Err(IntegrationError::NotImplemented)
    }

    fn update_method(
        &mut self,
        _scope: ScopeId,
        _integration_id: &str,
        _method: Method,
    ) -> Result<(), IntegrationError> {
        Err(IntegrationError::NotImplemented)
    }

    fn get(&self, _id: &str) -> Result<Option<Info>, IntegrationError> {
        Err(IntegrationError::NotImplemented)
    }

    fn list(&self) -> Result<Vec<Info>, IntegrationError> {
        Err(IntegrationError::NotImplemented)
    }
}

fn project_connections(
    _active: Option<(&str, &str)>,
    _env: &[(&str, bool)],
) -> Result<Vec<Connection>, IntegrationError> {
    Err(IntegrationError::NotImplemented)
}

#[test]
#[ignore = "porting: integration registry not implemented"]
fn registers_integrations_through_the_editor() {
    let mut registry = Registry::new().expect(NOTE);
    let scope = registry.open_scope().expect(NOTE);

    registry.update(scope, "openai", "OpenAI").expect(NOTE);
    assert_eq!(
        registry.get("openai").expect(NOTE),
        Some(Info {
            id: "openai".into(),
            name: "OpenAI".into(),
            methods: vec![],
            connections: vec![],
        })
    );

    registry.close_scope(scope).expect(NOTE);
    assert!(registry.get("openai").expect(NOTE).is_none());
}

#[test]
#[ignore = "porting: integration registry not implemented"]
fn reveals_the_previous_registration_when_an_override_closes() {
    let mut registry = Registry::new().expect(NOTE);
    let first = registry.open_scope().expect(NOTE);
    let second = registry.open_scope().expect(NOTE);

    registry.update(first, "openai", "OpenAI").expect(NOTE);
    registry
        .update(second, "openai", "OpenAI Override")
        .expect(NOTE);
    assert_eq!(
        registry
            .get("openai")
            .expect(NOTE)
            .expect("registered")
            .name,
        "OpenAI Override"
    );

    registry.close_scope(second).expect(NOTE);
    assert_eq!(
        registry
            .get("openai")
            .expect(NOTE)
            .expect("registered")
            .name,
        "OpenAI"
    );
    assert_eq!(
        registry
            .list()
            .expect(NOTE)
            .into_iter()
            .map(|info| info.id)
            .collect::<Vec<_>>(),
        vec!["openai"]
    );
}

#[test]
#[ignore = "porting: integration registry not implemented"]
fn registers_and_overrides_methods_independently() {
    let method = |label: &str| Method {
        id: Some("chatgpt".into()),
        kind: "oauth",
        label: label.into(),
    };

    let mut registry = Registry::new().expect(NOTE);
    let first = registry.open_scope().expect(NOTE);
    let second = registry.open_scope().expect(NOTE);

    registry
        .update_method(first, "openai", method("ChatGPT"))
        .expect(NOTE);
    registry
        .update_method(second, "openai", method("ChatGPT Override"))
        .expect(NOTE);

    assert_eq!(
        registry
            .get("openai")
            .expect(NOTE)
            .expect("registered")
            .methods[0]
            .label,
        "ChatGPT Override"
    );

    registry.close_scope(second).expect(NOTE);
    assert_eq!(
        registry
            .get("openai")
            .expect(NOTE)
            .expect("registered")
            .methods[0]
            .label,
        "ChatGPT"
    );
}

#[test]
#[ignore = "porting: integration registry not implemented"]
fn projects_the_active_credential_and_present_env_names() {
    let connections = project_connections(
        Some(("cred_personal", "Personal")),
        &[
            ("INTEGRATION_TEST_ACME_KEY", true),
            ("INTEGRATION_TEST_ACME_MISSING", false),
        ],
    )
    .expect(NOTE);

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
