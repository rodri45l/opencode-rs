//! Session message projection (port of packages/app/src/utils/session-message.ts).

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub provider_id: String,
    pub model_id: String,
    pub variant: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub id: String,
    pub role: String,
    pub parent_id: Option<String>,
    pub agent: Option<String>,
    pub model: Option<Model>,
    pub cost: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub id: String,
    pub part_type: String,
    pub tool: Option<String>,
}

#[derive(Default)]
pub struct Normalized {
    pub messages: Vec<Message>,
    pub parts: BTreeMap<String, Vec<Part>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContentPart {
    Reasoning,
    Text,
    Tool {
        id: String,
        name: String,
        output: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum SourceMessage {
    AgentSwitched {
        id: String,
        agent: String,
    },
    ModelSwitched {
        id: String,
        model: Model,
    },
    User {
        id: String,
        text: String,
        file_count: usize,
        agent_count: usize,
    },
    Assistant {
        id: String,
        agent: String,
        model: Model,
        content: Vec<ContentPart>,
        cost: f64,
    },
    Compaction {
        id: String,
    },
    Shell {
        id: String,
        command: String,
        output: String,
    },
}

pub fn normalize_session_messages(_session_id: &str, source: &[SourceMessage]) -> Normalized {
    let mut messages: Vec<Message> = Vec::new();
    let mut parts: BTreeMap<String, Vec<Part>> = BTreeMap::new();
    let mut agent = String::new();
    let mut model: Option<Model> = None;
    let mut parent_id: Option<String> = None;

    for message in source {
        match message {
            SourceMessage::AgentSwitched { agent: next, .. } => agent = next.clone(),
            SourceMessage::ModelSwitched { model: next, .. } => model = Some(next.clone()),
            SourceMessage::User {
                id,
                file_count,
                agent_count,
                ..
            } => {
                parent_id = Some(id.clone());
                messages.push(Message {
                    id: id.clone(),
                    role: "user".to_string(),
                    parent_id: None,
                    agent: Some(agent.clone()),
                    model: model.clone(),
                    cost: None,
                });
                let mut list = vec![Part {
                    id: format!("{id}:text:0"),
                    part_type: "text".to_string(),
                    tool: None,
                }];
                for index in 0..*file_count {
                    list.push(Part {
                        id: format!("{id}:file:{index}"),
                        part_type: "file".to_string(),
                        tool: None,
                    });
                }
                for index in 0..*agent_count {
                    list.push(Part {
                        id: format!("{id}:agent:{index}"),
                        part_type: "agent".to_string(),
                        tool: None,
                    });
                }
                parts.insert(id.clone(), list);
            }
            SourceMessage::Assistant {
                id,
                agent: next_agent,
                model: next_model,
                content,
                cost,
            } => {
                agent = next_agent.clone();
                model = Some(next_model.clone());
                if parent_id.is_none() {
                    continue;
                }
                if let Some(parent) = messages
                    .iter_mut()
                    .find(|item| Some(&item.id) == parent_id.as_ref())
                {
                    if parent.role == "user" {
                        parent.agent = Some(next_agent.clone());
                        parent.model = Some(next_model.clone());
                    }
                }
                messages.push(Message {
                    id: id.clone(),
                    role: "assistant".to_string(),
                    parent_id: parent_id.clone(),
                    agent: None,
                    model: None,
                    cost: Some(*cost),
                });
                let mut list = Vec::new();
                let mut text = 0;
                let mut reasoning = 0;
                for item in content {
                    match item {
                        ContentPart::Text => {
                            list.push(Part {
                                id: format!("{id}:text:{text}"),
                                part_type: "text".to_string(),
                                tool: None,
                            });
                            text += 1;
                        }
                        ContentPart::Reasoning => {
                            list.push(Part {
                                id: format!("{id}:reasoning:{reasoning}"),
                                part_type: "reasoning".to_string(),
                                tool: None,
                            });
                            reasoning += 1;
                        }
                        ContentPart::Tool {
                            id: tool_id, name, ..
                        } => list.push(Part {
                            id: tool_id.clone(),
                            part_type: "tool".to_string(),
                            tool: Some(name.clone()),
                        }),
                    }
                }
                parts.insert(id.clone(), list);
            }
            SourceMessage::Compaction { id } => {
                if let Some(pid) = &parent_id {
                    parts.entry(pid.clone()).or_default().push(Part {
                        id: format!("{id}:compaction"),
                        part_type: "compaction".to_string(),
                        tool: None,
                    });
                }
            }
            SourceMessage::Shell { id, .. } => {
                messages.push(Message {
                    id: id.clone(),
                    role: "user".to_string(),
                    parent_id: None,
                    agent: Some(agent.clone()),
                    model: model.clone(),
                    cost: None,
                });
                messages.push(Message {
                    id: format!("{id}:assistant"),
                    role: "assistant".to_string(),
                    parent_id: Some(id.clone()),
                    agent: None,
                    model: None,
                    cost: None,
                });
                parts.insert(
                    id.clone(),
                    vec![Part {
                        id: format!("{id}:text:0"),
                        part_type: "text".to_string(),
                        tool: None,
                    }],
                );
                parts.insert(
                    format!("{id}:assistant"),
                    vec![Part {
                        id: format!("{id}:tool"),
                        part_type: "tool".to_string(),
                        tool: Some("bash".to_string()),
                    }],
                );
                parent_id = None;
            }
        }
    }

    Normalized { messages, parts }
}
