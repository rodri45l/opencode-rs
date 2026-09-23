//! Port of packages/app/src/context/local-agent.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct Agent {
    name: String,
    native: Option<bool>,
}

// Local stubs (fast wave): real module lands later.
fn has_custom_agent(_agents: &[Agent]) -> bool {
    false
}

fn resolve_agent(_agents: &[Agent], _requested: Option<&str>) -> Option<Agent> {
    None
}

fn agent(name: &str, native: Option<bool>) -> Agent {
    Agent {
        name: name.into(),
        native,
    }
}

#[test]
#[ignore = "porting: context/local-agent not implemented"]
fn detects_explicitly_custom_agents() {
    assert!(has_custom_agent(&[
        agent("a", Some(true)),
        agent("b", Some(false))
    ]));
}

#[test]
#[ignore = "porting: context/local-agent not implemented"]
fn ignores_built_in_and_unclassified_agents() {
    assert!(!has_custom_agent(&[
        agent("a", Some(true)),
        agent("b", None)
    ]));
}

#[test]
#[ignore = "porting: context/local-agent not implemented"]
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
#[ignore = "porting: context/local-agent not implemented"]
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
#[ignore = "porting: context/local-agent not implemented"]
fn uses_the_first_agent_when_build_is_unavailable() {
    assert_eq!(
        resolve_agent(&[agent("custom", None)], Some("missing")).map(|a| a.name),
        Some("custom".to_string())
    );
}
