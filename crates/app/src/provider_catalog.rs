//! Provider catalog selection (port of packages/app/src/hooks/provider-catalog.ts).

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Catalog {
    pub all: BTreeMap<String, String>,
    pub connected: Vec<String>,
    pub default: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelRef {
    pub provider_id: String,
    pub model_id: String,
}

pub fn empty_catalog() -> Catalog {
    Catalog {
        all: BTreeMap::new(),
        connected: Vec::new(),
        default: BTreeMap::new(),
    }
}

pub fn select_provider_catalog(
    explicit: bool,
    directory: Option<&str>,
    ready: bool,
    providers: Option<Catalog>,
    global: Option<Catalog>,
) -> Catalog {
    if directory.is_some() && ready {
        if let Some(providers) = providers {
            return providers;
        }
    }
    if explicit {
        return empty_catalog();
    }
    global.unwrap_or_else(empty_catalog)
}

pub fn resolve_default_model(
    current: Option<ModelRef>,
    _legacy_config: Option<&str>,
) -> Option<ModelRef> {
    current
}

pub fn resolve_default_model_legacy(
    current: Option<Option<ModelRef>>,
    legacy_config: Option<&str>,
) -> Option<ModelRef> {
    match current {
        Some(Some(model)) => Some(model),
        Some(None) => parse_legacy(legacy_config),
        None => None,
    }
}

fn parse_legacy(legacy_config: Option<&str>) -> Option<ModelRef> {
    let legacy = legacy_config?;
    let (provider_id, model_id) = legacy.split_once('/')?;
    Some(ModelRef {
        provider_id: provider_id.to_string(),
        model_id: model_id.to_string(),
    })
}
