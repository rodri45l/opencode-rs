//! Local agent resolution (port of packages/app/src/context/local-agent.ts).

#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    pub name: String,
    pub native: Option<bool>,
}

pub fn has_custom_agent(agents: &[Agent]) -> bool {
    agents.iter().any(|agent| agent.native == Some(false))
}

pub fn resolve_agent(agents: &[Agent], requested: Option<&str>) -> Option<Agent> {
    if let Some(name) = requested {
        if let Some(agent) = agents.iter().find(|agent| agent.name == name) {
            return Some(agent.clone());
        }
    }
    if let Some(agent) = agents.iter().find(|agent| agent.name == "build") {
        return Some(agent.clone());
    }
    agents.first().cloned()
}
