//! Global sync normalisation helpers
//! (port of packages/app/src/context/global-sync/utils.ts).

use std::collections::BTreeMap;

use crate::path_key::path_key;

#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    pub name: String,
    pub description: Option<String>,
    pub mode: Option<String>,
    pub hidden: bool,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub color: Option<String>,
    pub permission: Vec<Permission>,
    pub model: Option<ModelRef>,
    pub variant: Option<String>,
    pub prompt: Option<String>,
    pub steps: Option<i64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Permission {
    pub permission: String,
    pub pattern: String,
    pub action: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelRef {
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PermissionRequest {
    pub id: String,
    pub session_id: String,
    pub permission: String,
    pub patterns: Vec<String>,
    pub always: Vec<String>,
    pub metadata_path: Option<String>,
    pub tool_message_id: Option<String>,
    pub tool_call_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelInfo {
    pub id: String,
    pub provider_id: String,
    pub toolcall: bool,
    pub attachment: bool,
    pub cost_input: f64,
    pub cost_output: f64,
    pub variants: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub models: BTreeMap<String, ModelInfo>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderList {
    pub connected: Vec<String>,
    pub default_model: Option<ModelRef>,
    pub default: BTreeMap<String, String>,
    pub all: BTreeMap<String, Provider>,
}

pub fn normalize_agent_list(input: Vec<Agent>) -> Vec<Agent> {
    input
}

pub fn normalize_permission_request(input: PermissionRequest) -> PermissionRequest {
    input
}

pub fn normalize_provider_list(
    providers: Vec<(String, String)>,
    models: Vec<ModelInfo>,
    default: Option<ModelRef>,
) -> ProviderList {
    let mut all: BTreeMap<String, Provider> = BTreeMap::new();
    for (id, name) in &providers {
        all.insert(
            id.clone(),
            Provider {
                id: id.clone(),
                name: name.clone(),
                models: BTreeMap::new(),
            },
        );
    }
    for model in models {
        if !model.toolcall {
            continue;
        }
        if let Some(provider) = all.get_mut(&model.provider_id) {
            provider.models.insert(model.id.clone(), model);
        }
    }

    let mut defaults: BTreeMap<String, String> = BTreeMap::new();
    for (id, _) in &providers {
        if let Some(default) = &default {
            if default.provider_id == *id {
                defaults.insert(id.clone(), default.model_id.clone());
                continue;
            }
        }
        if let Some(provider) = all.get(id) {
            if let Some((model_id, _)) = provider.models.iter().next() {
                defaults.insert(id.clone(), model_id.clone());
            }
        }
    }

    ProviderList {
        connected: providers.iter().map(|(id, _)| id.clone()).collect(),
        default_model: default,
        default: defaults,
        all,
    }
}

pub fn directory_key(path: &str) -> String {
    path_key(path)
}
