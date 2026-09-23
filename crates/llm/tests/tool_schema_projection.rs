//! Port of packages/llm/test/tool-schema-projection.test.ts (upstream 18ef3cc).
//! Behaviour pinned by `ToolSchemaProjection` and model compatibility lowering.

use opencode_llm::{Auth, LLMClient, ToolSchemaProjection, LLM};
use serde_json::json;

#[test]
#[ignore = "porting: tool schema projection not implemented"]
fn moonshot_strips_ref_siblings_and_converts_tuple_arrays() {
    let projected = ToolSchemaProjection::moonshot(json!({
        "type": "object",
        "properties": {
            "linked": { "$ref": "#/$defs/Linked", "description": "drop me" },
            "tuple": { "type": "array", "items": [{ "type": "string" }, { "type": "number" }] },
            "prefixTuple": { "type": "array", "prefixItems": [{ "type": "boolean" }, { "type": "string" }] }
        }
    }))
    .expect("moonshot projection");

    assert_eq!(
        projected,
        json!({
            "type": "object",
            "properties": {
                "linked": { "$ref": "#/$defs/Linked" },
                "tuple": { "type": "array", "items": { "anyOf": [{ "type": "string" }, { "type": "number" }] } },
                "prefixTuple": { "type": "array", "items": { "anyOf": [{ "type": "boolean" }, { "type": "string" }] } }
            }
        })
    );
}

#[test]
#[ignore = "porting: tool schema projection not implemented"]
fn gemini_normalises_numeric_enums_dangling_required_and_untyped_arrays() {
    let projected = ToolSchemaProjection::gemini(json!({
        "type": "object",
        "required": ["status", "missing"],
        "properties": {
            "status": { "type": "integer", "enum": [1, 2] },
            "tags": { "type": "array" },
            "name": { "type": "string", "properties": { "ignored": { "type": "string" } }, "required": ["ignored"] }
        }
    }))
    .expect("gemini projection");

    assert_eq!(
        projected,
        json!({
            "type": "object",
            "required": ["status"],
            "properties": {
                "status": { "type": "string", "enum": ["1", "2"] },
                "tags": { "type": "array", "items": { "type": "string" } },
                "name": { "type": "string" }
            }
        })
    );
}

#[test]
#[ignore = "porting: tool schema projection not implemented"]
fn openai_keeps_one_flat_object_top_level_schema() {
    let projected = ToolSchemaProjection::open_ai(json!({
        "anyOf": [
            { "type": "object", "properties": { "path": { "type": "string" }, "maybe": { "anyOf": [{ "type": "string" }, { "type": "null" }] } } },
            { "type": "object", "properties": { "resource": { "type": "string" } } }
        ]
    }))
    .expect("openai projection");

    assert_eq!(
        projected,
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "maybe": { "type": "string" },
                "resource": { "type": "string" }
            },
            "additionalProperties": false
        })
    );
}

#[test]
#[ignore = "porting: tool schema projection not implemented"]
fn applies_model_compatibility_before_protocol_projection() {
    let request = LLM::request(json!({
        "model": {
            "id": "kimi-k2",
            "provider": "openai",
            "route": { "id": "openai-chat" },
            "endpoint": { "baseURL": "https://api.openai.test/v1/" },
            "auth": Auth::bearer("test"),
            "compatibility": { "toolSchema": "moonshot" }
        },
        "prompt": "Use the tool.",
        "tools": [{
            "name": "lookup",
            "description": "Lookup data.",
            "inputSchema": {
                "type": "object",
                "anyOf": [{
                    "type": "object",
                    "properties": {
                        "tuple": { "type": "array", "items": [{ "type": "string" }, { "type": "number" }] },
                        "linked": { "$ref": "#/$defs/Linked", "description": "drop me" }
                    }
                }]
            }
        }]
    }));

    let prepared = LLMClient::prepare(request).expect("prepare");

    assert_eq!(
        prepared.body["tools"][0]["function"]["parameters"],
        json!({
            "type": "object",
            "properties": {
                "tuple": { "type": "array", "items": { "anyOf": [{ "type": "string" }, { "type": "number" }] } },
                "linked": { "$ref": "#/$defs/Linked" }
            },
            "additionalProperties": false
        })
    );
}
