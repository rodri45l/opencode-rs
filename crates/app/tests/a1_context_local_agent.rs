//! Port of packages/app/src/context/local-agent.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::local_agent::{has_custom_agent, resolve_agent, Agent};

fn agent(name: &str, native: Option<bool>) -> Agent {
    Agent {
        name: name.into(),
        native,
    }
}

#[test]
fn detects_explicitly_custom_agents() {
    assert!(has_custom_agent(&[
        agent("a", Some(true)),
        agent("b", Some(false))
    ]));
}

#[test]
fn ignores_built_in_and_unclassified_agents() {
    assert!(!has_custom_agent(&[
        agent("a", Some(true)),
        agent("b", None)
    ]));
}

#[test]
fn uses_the_requested_available_agent() {
    let agents = vec![
        agent("plan", None),
        agent("build", None),
        agent("custom", None),
    ];
    assert_eq!(
        resolve_agent(&agents, Some("custom")).map(|a| a.name),
        Some("custom".to_string())
    );
}

#[test]
fn defaults_to_build() {
    let agents = vec![
        agent("plan", None),
        agent("build", None),
        agent("custom", None),
    ];
    assert_eq!(
        resolve_agent(&agents, None).map(|a| a.name),
        Some("build".to_string())
    );
    assert_eq!(
        resolve_agent(&agents, Some("missing")).map(|a| a.name),
        Some("build".to_string())
    );
}

#[test]
fn uses_the_first_agent_when_build_is_unavailable() {
    assert_eq!(
        resolve_agent(&[agent("custom", None)], Some("missing")).map(|a| a.name),
        Some("custom".to_string())
    );
}
