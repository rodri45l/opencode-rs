//! Low-level protocol routes.

use serde_json::{json, Value};

macro_rules! protocol {
    ($module:ident, $id:literal) => {
        /// Protocol route facade.
        pub mod $module {
            use serde_json::{json, Value};

            /// Route id.
            pub const ID: &str = $id;

            /// The protocol's default route descriptor.
            pub fn route() -> Value {
                json!({ "id": ID })
            }

            /// The protocol id.
            pub fn id() -> &'static str {
                ID
            }
        }
    };
}

protocol!(openai_chat, "openai-chat");
protocol!(openai_compatible_chat, "openai-compatible-chat");
protocol!(openai_responses, "openai-responses");
protocol!(anthropic_messages, "anthropic-messages");
protocol!(gemini, "gemini");
protocol!(bedrock_converse, "bedrock-converse");

/// The OpenAI Responses WebSocket route.
pub fn openai_responses_web_socket_route() -> Value {
    json!({ "id": "openai-responses-websocket" })
}
