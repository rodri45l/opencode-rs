//! Model variant resolution (port of packages/app/src/context/model-variant.ts).

#[derive(Clone, Debug, PartialEq)]
pub struct ModelRef {
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    pub model: ModelRef,
    pub variant: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub provider_id: String,
    pub model_id: String,
    pub variants: Vec<String>,
}

pub fn get_configured_agent_variant(agent: &Agent, model: &Model) -> Option<String> {
    let variant = agent.variant.as_ref()?;
    if agent.model.provider_id != model.provider_id {
        return None;
    }
    if agent.model.model_id != model.model_id {
        return None;
    }
    if !model.variants.contains(variant) {
        return None;
    }
    Some(variant.clone())
}

pub fn resolve_model_variant(
    variants: &[String],
    selected: Option<Option<String>>,
    configured: Option<String>,
) -> Option<String> {
    if selected == Some(None) {
        return None;
    }
    if let Some(Some(value)) = selected {
        if variants.contains(&value) {
            return Some(value);
        }
    }
    if let Some(value) = configured {
        if variants.contains(&value) {
            return Some(value);
        }
    }
    None
}

pub fn cycle_model_variant(
    variants: &[String],
    selected: Option<Option<String>>,
    configured: Option<String>,
) -> Option<String> {
    if variants.is_empty() {
        return None;
    }
    if selected == Some(None) {
        return Some(variants[0].clone());
    }
    if let Some(Some(value)) = selected {
        if let Some(index) = variants.iter().position(|item| *item == value) {
            if index == variants.len() - 1 {
                return None;
            }
            return Some(variants[index + 1].clone());
        }
    }
    if let Some(value) = configured {
        if let Some(index) = variants.iter().position(|item| *item == value) {
            if index == variants.len() - 1 {
                return Some(variants[0].clone());
            }
            return Some(variants[index + 1].clone());
        }
    }
    Some(variants[0].clone())
}
