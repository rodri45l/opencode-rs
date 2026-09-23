//! Built-in SDK provider plugins (re-derived behavioural subset).
//!
//! Ports the observable behaviour of the `AISDK`-bound provider plugins in
//! `packages/core/src/plugin/provider/*`: each plugin binds to an exact SDK
//! package, derives the SDK provider name from the provider id, selects a
//! language-model accessor, defaults `includeUsage` for the openai-compatible
//! fallback, and may disable specific catalog models. The Effect service wiring
//! and the SDK factories themselves are dropped; the pure decisions remain.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// Which SDK accessor a provider plugin uses to select a language model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageSelector {
    /// `sdk.languageModel(model_id)`.
    LanguageModel,
    /// `sdk.responses(model_id)`.
    Responses,
    /// `sdk.messages(model_id)`.
    Messages,
    /// `sdk.chat(model_id)`.
    Chat,
}

/// The accessors an SDK advertises, used to resolve fallback order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SdkCapabilities {
    /// The SDK exposes `responses`.
    pub responses: bool,
    /// The SDK exposes `messages`.
    pub messages: bool,
    /// The SDK exposes `chat`.
    pub chat: bool,
    /// The SDK exposes `language_model`.
    pub language_model: bool,
}

/// A resolved language-model selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageSelection {
    /// The accessor the plugin chose.
    pub selector: LanguageSelector,
    /// The model id passed to the accessor.
    pub model_id: String,
}

/// A built-in SDK provider plugin descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdkPlugin {
    /// Plugin id (for example `"anthropic"`).
    pub id: &'static str,
    /// The exact npm package this plugin binds to.
    pub package: &'static str,
}

/// Inputs for resolving a plugin's language-model selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageQuery<'a> {
    /// Plugin id.
    pub plugin: &'a str,
    /// Provider id the model belongs to.
    pub provider_id: &'a str,
    /// Catalog model id (the alias the user selected).
    pub model_id: &'a str,
    /// Wire API model id advertised by the model.
    pub api_id: &'a str,
    /// Accessors the SDK advertises.
    pub capabilities: SdkCapabilities,
    /// Whether the provider is configured to use legacy completion URLs.
    pub use_completion_urls: bool,
}

/// Registry of built-in SDK provider plugins.
#[derive(Debug, Default)]
pub struct ProviderSdkPlugins;

const PLUGINS: &[SdkPlugin] = &[
    SdkPlugin {
        id: "openai",
        package: "@ai-sdk/openai",
    },
    SdkPlugin {
        id: "anthropic",
        package: "@ai-sdk/anthropic",
    },
    SdkPlugin {
        id: "google",
        package: "@ai-sdk/google",
    },
    SdkPlugin {
        id: "google-vertex-anthropic",
        package: "@ai-sdk/google-vertex/anthropic",
    },
    SdkPlugin {
        id: "google-vertex",
        package: "@ai-sdk/google-vertex",
    },
    SdkPlugin {
        id: "azure",
        package: "@ai-sdk/azure",
    },
    SdkPlugin {
        id: "azure-cognitive-services",
        package: "@ai-sdk/openai-compatible",
    },
    SdkPlugin {
        id: "amazon-bedrock",
        package: "@ai-sdk/amazon-bedrock",
    },
    SdkPlugin {
        id: "openai-compatible",
        package: "@ai-sdk/openai-compatible",
    },
    SdkPlugin {
        id: "cerebras",
        package: "@ai-sdk/cerebras",
    },
    SdkPlugin {
        id: "deepinfra",
        package: "@ai-sdk/deepinfra",
    },
    SdkPlugin {
        id: "groq",
        package: "@ai-sdk/groq",
    },
    SdkPlugin {
        id: "mistral",
        package: "@ai-sdk/mistral",
    },
    SdkPlugin {
        id: "togetherai",
        package: "@ai-sdk/togetherai",
    },
    SdkPlugin {
        id: "xai",
        package: "@ai-sdk/xai",
    },
    SdkPlugin {
        id: "cohere",
        package: "@ai-sdk/cohere",
    },
    SdkPlugin {
        id: "alibaba",
        package: "@ai-sdk/alibaba",
    },
    SdkPlugin {
        id: "perplexity",
        package: "@ai-sdk/perplexity",
    },
    SdkPlugin {
        id: "openrouter",
        package: "@openrouter/ai-sdk-provider",
    },
    SdkPlugin {
        id: "venice",
        package: "venice-ai-sdk-provider",
    },
    SdkPlugin {
        id: "github-copilot",
        package: "@ai-sdk/github-copilot",
    },
    SdkPlugin {
        id: "gateway",
        package: "@ai-sdk/gateway",
    },
    SdkPlugin {
        id: "vercel",
        package: "@ai-sdk/vercel",
    },
];

fn suffix_chat(plugin: &str) -> bool {
    matches!(
        plugin,
        "openai-compatible" | "cohere" | "deepinfra" | "togetherai" | "venice" | "vercel"
    )
}

impl ProviderSdkPlugins {
    /// Find the plugin bound to the exact `package`, preserving registry order.
    pub fn by_package(package: &str) -> CoreResult<Option<SdkPlugin>> {
        Ok(PLUGINS
            .iter()
            .find(|plugin| plugin.package == package)
            .cloned())
    }

    /// Whether `plugin` handles `package` under that plugin's matching rule
    /// (most are exact; the openai-compatible fallback also matches package
    /// paths that contain it).
    pub fn matches_package(plugin: &str, package: &str) -> CoreResult<bool> {
        let Some(entry) = PLUGINS.iter().find(|entry| entry.id == plugin) else {
            return Ok(false);
        };
        if package != entry.package {
            return Ok(false);
        }
        // openai-compatible must not match a more specific provider package.
        if plugin == "openai-compatible" && package != "@ai-sdk/openai-compatible" {
            return Ok(false);
        }
        Ok(true)
    }

    /// The SDK provider name a plugin reports for `provider_id`.
    pub fn sdk_provider_name(plugin: &str, provider_id: &str) -> CoreResult<String> {
        if !PLUGINS.iter().any(|entry| entry.id == plugin) {
            return Ok(provider_id.to_string());
        }
        if plugin == "perplexity" {
            return Ok("perplexity".to_string());
        }
        if plugin == "alibaba" {
            return Ok("alibaba.chat".to_string());
        }
        if plugin == "togetherai" {
            return Ok("togetherai.chat".to_string());
        }
        if plugin == "vercel" {
            return Ok("vercel.chat".to_string());
        }
        if suffix_chat(plugin) {
            return Ok(format!("{provider_id}.chat"));
        }
        Ok(provider_id.to_string())
    }

    /// Resolve the language-model selection for a provider turn.
    pub fn select_language(query: &LanguageQuery<'_>) -> CoreResult<Option<LanguageSelection>> {
        let plugin = query.plugin;
        let selection = |selector: LanguageSelector| {
            Some(LanguageSelection {
                selector,
                model_id: query.api_id.to_string(),
            })
        };
        match plugin {
            "openai" | "xai" => {
                if query.provider_id != plugin {
                    return Ok(None);
                }
                if !query.capabilities.responses {
                    return Ok(None);
                }
                Ok(selection(LanguageSelector::Responses))
            }
            "azure" | "azure-cognitive-services" => {
                if plugin == "azure-cognitive-services"
                    && query.provider_id != "azure-cognitive-services"
                {
                    return Ok(None);
                }
                if plugin == "azure" && query.provider_id != "azure" {
                    return Ok(None);
                }
                if query.use_completion_urls {
                    return Ok(selection(LanguageSelector::Chat));
                }
                if query.capabilities.responses {
                    return Ok(selection(LanguageSelector::Responses));
                }
                if query.capabilities.messages {
                    return Ok(selection(LanguageSelector::Messages));
                }
                if query.capabilities.chat {
                    return Ok(selection(LanguageSelector::Chat));
                }
                if query.capabilities.language_model {
                    return Ok(selection(LanguageSelector::LanguageModel));
                }
                Ok(None)
            }
            "github-copilot" => {
                if query.provider_id != "github-copilot" {
                    return Ok(None);
                }
                let model = query.api_id;
                let is_gpt5 = model.starts_with("gpt-5");
                let is_mini = model.starts_with("gpt-5-mini");
                if is_gpt5 && !is_mini && query.capabilities.responses {
                    return Ok(selection(LanguageSelector::Responses));
                }
                if query.capabilities.chat {
                    return Ok(selection(LanguageSelector::Chat));
                }
                if query.capabilities.language_model {
                    return Ok(selection(LanguageSelector::LanguageModel));
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// The defaulted `includeUsage` option for an OpenAI-compatible SDK.
    pub fn include_usage(options: &Value) -> CoreResult<bool> {
        match options.get("includeUsage") {
            Some(Value::Bool(value)) => Ok(*value),
            _ => Ok(true),
        }
    }

    /// Whether `plugin` disables `model_id` for `provider_id` in catalog transforms.
    pub fn disables_model(plugin: &str, provider_id: &str, model_id: &str) -> CoreResult<bool> {
        Ok(match plugin {
            "openai" => provider_id == "openai" && model_id == "gpt-5-chat-latest",
            "github-copilot" => provider_id == "github-copilot" && model_id == "gpt-5-chat-latest",
            "openrouter" => provider_id == "openrouter" && model_id == "openai/gpt-5-chat",
            _ => false,
        })
    }
}

/// Error for a query against an unknown plugin.
pub fn unknown_plugin(plugin: &str) -> CoreError {
    CoreError::Invalid(format!("unknown sdk plugin: {plugin}"))
}
