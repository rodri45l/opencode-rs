//! User-facing provider facades.
//!
//! Each facade resolves a model id to a route descriptor. Behaviour is pinned by
//! the ported tests in `tests/provider/*`.

use serde_json::{json, Value};

/// A configured provider facade.
#[derive(Debug, Clone)]
pub struct Configured {
    provider: String,
    default_route: String,
    chat_route: String,
    responses_route: String,
    base_url: Option<String>,
}

impl Configured {
    fn with_routes(
        provider: &str,
        default_route: &str,
        chat_route: &str,
        responses_route: &str,
    ) -> Self {
        Self {
            provider: provider.to_string(),
            default_route: default_route.to_string(),
            chat_route: chat_route.to_string(),
            responses_route: responses_route.to_string(),
            base_url: None,
        }
    }

    fn model_on(&self, id: &str, route: &str) -> Value {
        json!({
            "id": id,
            "provider": self.provider,
            "route": { "id": route },
            "endpoint": { "baseURL": self.base_url },
        })
    }

    /// Model using the provider's default route.
    pub fn model(&self, id: &str) -> Value {
        self.model_on(id, &self.default_route)
    }

    /// Model on the provider's chat route.
    pub fn chat(&self, id: &str) -> Value {
        self.model_on(id, &self.chat_route)
    }

    /// Model on the provider's responses route.
    pub fn responses(&self, id: &str) -> Value {
        self.model_on(id, &self.responses_route)
    }

    /// Model on the provider's responses WebSocket route.
    pub fn responses_web_socket(&self, id: &str) -> Value {
        self.model_on(id, "openai-responses-websocket")
    }

    /// Provider id.
    pub fn provider_id(&self) -> &str {
        &self.provider
    }
}

/// The root `Provider` export.
pub struct Provider;

impl Provider {
    /// Build a provider descriptor.
    pub fn make(input: Value) -> Value {
        input
    }
}

/// OpenAI-compatible provider families.
pub mod openai_compatible {
    use super::{Configured, Value};
    use serde_json::json;

    /// A provider family with a fixed base URL.
    #[derive(Debug, Clone, Copy)]
    pub struct Family {
        provider: &'static str,
        base_url: &'static str,
    }

    impl Family {
        /// Configure the family with credentials/overrides.
        pub fn configure(&self, config: Value) -> Configured {
            let mut configured = Configured::with_routes(
                self.provider,
                "openai-compatible-chat",
                "openai-compatible-chat",
                "openai-compatible-chat",
            );
            let mut merged = json!({ "baseURL": self.base_url });
            if let (Some(dst), Some(src)) = (merged.as_object_mut(), config.as_object()) {
                for (key, value) in src {
                    dst.insert(key.clone(), value.clone());
                }
            }
            configured = configured.with_config(&merged);
            configured
        }

        /// Model using the family defaults.
        pub fn model(&self, id: &str) -> Value {
            self.configure(json!({})).model(id)
        }
    }

    macro_rules! family {
        ($name:ident, $provider:literal, $url:literal) => {
            /// The provider family.
            pub fn $name() -> Family {
                Family {
                    provider: $provider,
                    base_url: $url,
                }
            }
        };
    }

    family!(baseten, "baseten", "https://inference.baseten.co/v1");
    family!(cerebras, "cerebras", "https://api.cerebras.ai/v1");
    family!(
        deepinfra,
        "deepinfra",
        "https://api.deepinfra.com/v1/openai"
    );
    family!(deepseek, "deepseek", "https://api.deepseek.com/v1");
    family!(
        fireworks,
        "fireworks",
        "https://api.fireworks.ai/inference/v1"
    );
    family!(togetherai, "togetherai", "https://api.together.xyz/v1");
    family!(groq, "groq", "https://api.groq.com/openai/v1");
}

macro_rules! facade {
    ($module:ident, $provider:literal, $default:literal, $chat:literal, $responses:literal) => {
        /// Provider facade.
        pub mod $module {
            use super::{Configured, Value};
            use serde_json::json;

            /// Configure the provider.
            pub fn configure(config: Value) -> Configured {
                Configured::with_routes($provider, $default, $chat, $responses).with_config(&config)
            }

            /// Default model helper.
            pub fn model(id: &str) -> Value {
                configure(json!({})).model(id)
            }

            /// Responses model helper.
            pub fn responses(id: &str) -> Value {
                configure(json!({})).responses(id)
            }

            /// Responses WebSocket model helper.
            pub fn responses_web_socket(id: &str) -> Value {
                configure(json!({})).responses_web_socket(id)
            }

            /// The provider barrel's nested facade.
            pub fn provider() -> Facade {
                Facade
            }

            /// Nested facade mirroring the reference `provider` subpath.
            pub struct Facade;

            impl Facade {
                /// Default model helper.
                pub fn model(&self, id: &str) -> Value {
                    model(id)
                }

                /// Responses model helper.
                pub fn responses(&self, id: &str) -> Value {
                    responses(id)
                }

                /// Responses WebSocket model helper.
                pub fn responses_web_socket(&self, id: &str) -> Value {
                    responses_web_socket(id)
                }
            }
        }
    };
}

facade!(
    openai,
    "openai",
    "openai-responses",
    "openai-chat",
    "openai-responses"
);
facade!(
    xai,
    "xai",
    "openai-responses",
    "openai-compatible-chat",
    "openai-responses"
);
facade!(
    azure,
    "azure",
    "openai-chat",
    "openai-chat",
    "openai-responses"
);

/// OpenRouter facade.
pub mod openrouter {
    use super::{Configured, Value};
    use serde_json::json;

    /// Configure OpenRouter.
    pub fn configure(config: Value) -> Configured {
        Configured::with_routes("openrouter", "openrouter", "openrouter", "openrouter")
            .with_config(&config)
    }

    /// Model helper.
    pub fn model(id: &str) -> Value {
        configure(json!({})).model(id)
    }

    /// The nested facade.
    pub fn provider() -> Configured {
        configure(json!({}))
    }
}

/// Anthropic facade.
pub mod anthropic {
    use super::{Configured, Value};
    use serde_json::json;

    /// Configure Anthropic.
    pub fn configure(config: Value) -> Configured {
        Configured::with_routes(
            "anthropic",
            "anthropic-messages",
            "anthropic-messages",
            "anthropic-messages",
        )
        .with_config(&config)
    }

    /// Model helper.
    pub fn model(id: &str) -> Value {
        configure(json!({})).model(id)
    }
}

/// Google Gemini facade.
pub mod google {
    use super::{Configured, Value};
    use serde_json::json;

    /// Configure Google.
    pub fn configure(config: Value) -> Configured {
        Configured::with_routes("google", "gemini", "gemini", "gemini").with_config(&config)
    }

    /// Model helper.
    pub fn model(id: &str) -> Value {
        configure(json!({})).model(id)
    }
}

/// Amazon Bedrock facade.
pub mod amazon_bedrock {
    use super::{Configured, Value};
    use serde_json::json;

    /// Configure Bedrock.
    pub fn configure(config: Value) -> Configured {
        Configured::with_routes(
            "amazon-bedrock",
            "bedrock-converse",
            "bedrock-converse",
            "bedrock-converse",
        )
        .with_config(&config)
    }

    /// Model helper.
    pub fn model(id: &str) -> Value {
        configure(json!({})).model(id)
    }
}

/// Cloudflare facade.
pub mod cloudflare {
    use super::{Configured, Value};

    /// Cloudflare AI Gateway.
    pub fn ai_gateway(config: Value) -> Configured {
        Configured::with_routes(
            "cloudflare-ai-gateway",
            "cloudflare-ai-gateway",
            "cloudflare-ai-gateway",
            "cloudflare-ai-gateway",
        )
        .with_config(&config)
    }

    /// Cloudflare Workers AI.
    pub fn workers_ai(config: Value) -> Configured {
        Configured::with_routes(
            "cloudflare-workers-ai",
            "cloudflare-workers-ai",
            "cloudflare-workers-ai",
            "cloudflare-workers-ai",
        )
        .with_config(&config)
    }
}

/// GitHub Copilot facade.
pub mod github_copilot {
    use super::{Configured, Value};
    use serde_json::json;

    /// Configure Copilot. The `endpoint` config selects the default route.
    pub fn configure(config: Value) -> Configured {
        let route = match config.get("endpoint").and_then(Value::as_str) {
            Some("chat") => "openai-chat",
            Some("responses") => "openai-responses",
            _ => "openai-responses",
        };
        Configured::with_routes("github-copilot", route, "openai-chat", "openai-responses")
            .with_config(&config)
    }

    /// Model helper.
    pub fn model(id: &str) -> Value {
        configure(json!({})).model(id)
    }
}

impl Configured {
    /// Apply the config's `baseURL` and `endpoint` overrides.
    pub fn with_config(mut self, config: &Value) -> Self {
        self.base_url = config
            .get("baseURL")
            .and_then(Value::as_str)
            .map(str::to_string);
        self
    }
}
