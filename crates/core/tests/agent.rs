//! Port of packages/core/test/agent.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: transform-created agents, replay through `reload`, direct
//! update/remove, and runtime defaults. Dropped (re-derived): the Effect `Scope`
//! transform-removal case and the `AgentPlugin`/`Layer` built-in-agent case
//! (`does not ambiently opt built-in agents into bash`), which only exercise
//! Effect layer/scope wiring that has no Rust analogue here.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use opencode_core::agent::{AgentId, AgentInfo, AgentMode, AgentRegistry};

#[test]
fn starts_without_agents() {
    let agent = AgentRegistry::new().unwrap();
    assert!(agent.all().unwrap().is_empty());
    assert!(agent.get(&AgentId::make("build")).unwrap().is_none());
}

#[test]
fn materializes_replayable_agent_transforms() {
    let agent = AgentRegistry::new().unwrap();
    let id = AgentId::make("reviewer");
    let target = id.clone();

    agent
        .transform(move |editor| {
            editor.update(target.clone(), |info| {
                info.description = Some("Reviews code".into());
                info.mode = Some(AgentMode::Subagent);
            });
        })
        .unwrap();

    let got = agent.get(&id).unwrap().expect("agent exists");
    assert_eq!(got.id, id);
    assert_eq!(got.description, "Reviews code");
    assert_eq!(got.mode, AgentMode::Subagent);

    let ids: Vec<AgentId> = agent
        .all()
        .unwrap()
        .into_iter()
        .map(|info| info.id)
        .collect();
    assert_eq!(ids, vec![id]);
}

#[test]
fn rebuilds_state_when_a_transform_is_replaced() {
    let agent = AgentRegistry::new().unwrap();
    let id = AgentId::make("reviewer");
    let description = Rc::new(RefCell::new("Old description".to_string()));
    let hidden = Rc::new(Cell::new(true));

    let captured_description = Rc::clone(&description);
    let captured_hidden = Rc::clone(&hidden);
    let target = id.clone();
    agent
        .transform(move |editor| {
            editor.update(target.clone(), |info| {
                info.description = Some(captured_description.borrow().clone());
                info.hidden = Some(captured_hidden.get());
            });
        })
        .unwrap();

    *description.borrow_mut() = "New description".to_string();
    hidden.set(false);
    agent.reload().unwrap();

    let got = agent.get(&id).unwrap().expect("agent exists");
    assert_eq!(got.description, "New description");
    assert!(!got.hidden);
}

#[test]
fn applies_direct_agent_updates() {
    let agent = AgentRegistry::new().unwrap();
    let id = AgentId::make("build");
    let target = id.clone();

    agent
        .transform(move |editor| {
            editor.update(target.clone(), |info| {
                info.mode = Some(AgentMode::Primary);
                info.hidden = Some(true);
            });
        })
        .unwrap();

    let got = agent.get(&id).unwrap().expect("agent exists");
    assert_eq!(got.id, id);
    assert_eq!(got.mode, AgentMode::Primary);
    assert!(got.hidden);
}

#[test]
fn creates_agents_with_runtime_defaults_and_supports_removal() {
    let agent = AgentRegistry::new().unwrap();
    let id = AgentId::make("custom");
    let target = id.clone();

    agent
        .transform(move |editor| editor.update(target.clone(), |_| {}))
        .unwrap();
    assert_eq!(
        agent.get(&id).unwrap().expect("agent exists"),
        AgentInfo::empty(id.clone()).unwrap()
    );

    let target = id.clone();
    agent
        .transform(move |editor| editor.remove(target.clone()))
        .unwrap();
    assert!(agent.get(&id).unwrap().is_none());
}
