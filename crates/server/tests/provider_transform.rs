//! Port of packages/opencode/test/provider/transform.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `ProviderTransform.schema` for Gemini (array items, type
//! arrays, combiner nodes, non-object cleanup), OpenAI (supported-keyword
//! subset), and Moonshot (`$ref` siblings, tuple items).
//!
//! Dropped (needs the live SDK/catalog): `options`, `providerOptions`, and
//! `message` cases.

use opencode_server::port::provider_transform::schema;
use serde_json::{json, Value};

fn gemini(value: Value) -> Value {
    schema("google", "gemini-3-pro", "", &value)
}

#[test]
fn gemini_adds_missing_items_for_array_properties() {
    let result = gemini(json!({
        "type": "object",
        "properties": {
            "nodes": { "type": "array" },
            "edges": { "type": "array", "items": { "type": "string" } }
        }
    }));
    assert!(result["properties"]["nodes"]["items"].is_object());
    assert_eq!(
        result["properties"]["edges"]["items"]["type"],
        json!("string")
    );
}

#[test]
fn gemini_adds_type_to_2d_array_with_empty_inner_items() {
    let result = gemini(json!({
        "type": "object",
        "properties": {
            "values": { "type": "array", "items": { "type": "array", "items": {} } }
        }
    }));
    assert_eq!(
        result["properties"]["values"]["items"]["items"]["type"],
        json!("string")
    );
}

#[test]
fn gemini_adds_items_and_type_to_2d_array_with_missing_inner_items() {
    let result = gemini(json!({
        "type": "object",
        "properties": {
            "data": { "type": "array", "items": { "type": "array" } }
        }
    }));
    assert!(result["properties"]["data"]["items"]["items"].is_object());
    assert_eq!(
        result["properties"]["data"]["items"]["items"]["type"],
        json!("string")
    );
}

#[test]
fn gemini_handles_deeply_nested_arrays() {
    let result = gemini(json!({
        "type": "object",
        "properties": {
            "matrix": {
                "type": "array",
                "items": { "type": "array", "items": { "type": "array" } }
            }
        }
    }));
    assert!(result["properties"]["matrix"]["items"]["items"]["items"].is_object());
    assert_eq!(
        result["properties"]["matrix"]["items"]["items"]["items"]["type"],
        json!("string")
    );
}

#[test]
fn gemini_preserves_existing_item_types_in_nested_arrays() {
    let result = gemini(json!({
        "type": "object",
        "properties": {
            "numbers": { "type": "array", "items": { "type": "array", "items": { "type": "number" } } }
        }
    }));
    assert_eq!(
        result["properties"]["numbers"]["items"]["items"]["type"],
        json!("number")
    );
}

#[test]
fn gemini_handles_mixed_nested_structures() {
    let result = gemini(json!({
        "type": "object",
        "properties": {
            "spreadsheetData": {
                "type": "object",
                "properties": {
                    "rows": { "type": "array", "items": { "type": "array", "items": {} } }
                }
            }
        }
    }));
    assert_eq!(
        result["properties"]["spreadsheetData"]["properties"]["rows"]["items"]["items"]["type"],
        json!("string")
    );
}

#[test]
fn gemini_splits_a_multi_type_array_into_any_of() {
    let result = gemini(json!({
        "type": "object",
        "properties": {
            "status": { "type": ["number", "string"], "description": "status filter" }
        }
    }));
    assert!(result["properties"]["status"].get("type").is_none());
    assert_eq!(
        result["properties"]["status"]["anyOf"],
        json!([{ "type": "number" }, { "type": "string" }])
    );
    assert!(result["properties"]["status"].get("nullable").is_none());
    assert_eq!(
        result["properties"]["status"]["description"],
        json!("status filter")
    );
}

#[test]
fn gemini_lifts_null_into_nullable_for_type_array() {
    let result = gemini(json!({
        "type": "object",
        "properties": {
            "maybe": { "type": ["string", "null"], "description": "nullable string" }
        }
    }));
    assert!(result["properties"]["maybe"].get("type").is_none());
    assert_eq!(
        result["properties"]["maybe"]["anyOf"],
        json!([{ "type": "string" }])
    );
    assert_eq!(result["properties"]["maybe"]["nullable"], json!(true));
}

#[test]
fn gemini_collapses_all_null_type_array() {
    let result = gemini(json!({
        "type": "object",
        "properties": { "nothing": { "type": ["null"] } }
    }));
    assert_eq!(result["properties"]["nothing"]["type"], json!("null"));
    assert!(result["properties"]["nothing"].get("anyOf").is_none());
}

#[test]
fn gemini_rewrites_type_arrays_for_copilot_gemini() {
    let result = schema(
        "github-copilot",
        "gemini-3.5-flash",
        "@ai-sdk/github-copilot",
        &json!({
            "type": "object",
            "properties": {
                "hook_id": { "type": "number", "description": "ID of the webhook" },
                "status": { "type": ["number", "string"], "description": "Filter by response status code" }
            },
            "required": ["hook_id"],
            "additionalProperties": false
        }),
    );
    assert_eq!(
        result["properties"]["status"]["anyOf"],
        json!([{ "type": "number" }, { "type": "string" }])
    );
    assert!(result["properties"]["status"].get("type").is_none());
    assert_eq!(result["properties"]["hook_id"]["type"], json!("number"));
}

#[test]
fn gemini_keeps_items_any_of_without_adding_type() {
    let result = gemini(json!({
        "type": "object",
        "properties": {
            "edits": {
                "type": "array",
                "items": {
                    "anyOf": [
                        { "type": "object", "properties": { "old_string": { "type": "string" } } },
                        { "type": "object", "properties": { "new_string": { "type": "string" } } }
                    ]
                }
            }
        }
    }));
    assert!(result["properties"]["edits"]["items"]["anyOf"].is_array());
    assert!(result["properties"]["edits"]["items"].get("type").is_none());
}

#[test]
fn gemini_does_not_add_sibling_keys_to_combiner_nodes() {
    let input = json!({
        "type": "object",
        "properties": {
            "edits": {
                "type": "array",
                "items": { "anyOf": [{ "type": "string" }, { "type": "number" }] }
            },
            "value": { "oneOf": [{ "type": "string" }, { "type": "boolean" }] },
            "meta": {
                "allOf": [
                    { "type": "object", "properties": { "a": { "type": "string" } } },
                    { "type": "object", "properties": { "b": { "type": "string" } } }
                ]
            }
        }
    });
    assert_eq!(gemini(input.clone()), input);
}

#[test]
fn gemini_removes_properties_from_non_object_types() {
    let result = gemini(json!({
        "type": "object",
        "properties": {
            "data": { "type": "string", "properties": { "invalid": { "type": "string" } } }
        }
    }));
    assert_eq!(result["properties"]["data"]["type"], json!("string"));
    assert!(result["properties"]["data"].get("properties").is_none());
}

#[test]
fn gemini_removes_required_from_non_object_types() {
    let result = gemini(json!({
        "type": "object",
        "properties": {
            "data": { "type": "array", "items": { "type": "string" }, "required": ["invalid"] }
        }
    }));
    assert_eq!(result["properties"]["data"]["type"], json!("array"));
    assert!(result["properties"]["data"].get("required").is_none());
}

#[test]
fn gemini_removes_properties_and_required_from_nested_non_objects() {
    let result = gemini(json!({
        "type": "object",
        "properties": {
            "outer": {
                "type": "object",
                "properties": {
                    "inner": { "type": "number", "properties": { "bad": { "type": "string" } }, "required": ["bad"] }
                }
            }
        }
    }));
    assert_eq!(
        result["properties"]["outer"]["properties"]["inner"]["type"],
        json!("number")
    );
    assert!(result["properties"]["outer"]["properties"]["inner"]
        .get("properties")
        .is_none());
    assert!(result["properties"]["outer"]["properties"]["inner"]
        .get("required")
        .is_none());
}

#[test]
fn gemini_keeps_properties_and_required_on_object_types() {
    let result = gemini(json!({
        "type": "object",
        "properties": {
            "data": { "type": "object", "properties": { "name": { "type": "string" } }, "required": ["name"] }
        }
    }));
    assert_eq!(result["properties"]["data"]["type"], json!("object"));
    assert!(result["properties"]["data"]["properties"].is_object());
    assert_eq!(result["properties"]["data"]["required"], json!(["name"]));
}

#[test]
fn gemini_does_not_affect_non_gemini_providers() {
    let result = schema(
        "openai",
        "gpt-4",
        "@ai-sdk/openai",
        &json!({
            "type": "object",
            "properties": { "data": { "type": "string", "properties": { "invalid": { "type": "string" } } } }
        }),
    );
    assert!(result["properties"]["data"]["properties"].is_object());
}

#[test]
fn openai_removes_unsupported_keywords_recursively() {
    let result = schema(
        "openai",
        "gpt-4.1",
        "@ai-sdk/openai",
        &json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Search",
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query",
                    "format": "uri",
                    "pattern": "^https://",
                    "minLength": 1,
                    "maxLength": 100,
                    "default": "https://example.com"
                },
                "count": { "type": "integer", "minimum": 1, "maximum": 10, "multipleOf": 1 },
                "createdAt": { "format": "date-time" },
                "mode": { "const": "fast" },
                "tags": { "type": "array", "minItems": 1, "maxItems": 3, "uniqueItems": true },
                "tuple": {
                    "type": "array",
                    "items": [
                        { "type": "number", "minimum": 0 },
                        { "type": "string", "pattern": "^ok$" }
                    ]
                },
                "metadata": {
                    "type": "object",
                    "patternProperties": { "^x-": { "type": "string" } },
                    "additionalProperties": { "type": "string", "pattern": "^safe$" }
                }
            },
            "patternProperties": { "^extra": { "type": "string" } },
            "required": ["query"],
            "additionalProperties": false
        }),
    );
    assert_eq!(
        result,
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query" },
                "count": { "type": "integer" },
                "createdAt": { "type": "string" },
                "mode": { "enum": ["fast"], "type": "string" },
                "tags": { "type": "array", "items": { "type": "string" } },
                "tuple": { "type": "array", "items": [{ "type": "number" }, { "type": "string" }] },
                "metadata": {
                    "type": "object",
                    "properties": {},
                    "additionalProperties": { "type": "string" }
                }
            },
            "required": ["query"],
            "additionalProperties": false
        })
    );
}

#[test]
fn openai_keeps_local_references_and_sanitizes_definitions() {
    let result = schema(
        "openai",
        "gpt-4.1",
        "@ai-sdk/openai",
        &json!({
            "type": "object",
            "properties": {
                "value": { "$ref": "#/$defs/Value", "description": "Referenced value", "examples": ["ignored"] }
            },
            "$defs": {
                "Value": { "type": "string", "pattern": "^value$", "description": "Definition description" },
                "Unused": { "type": "number", "minimum": 0 }
            }
        }),
    );
    assert_eq!(
        result["properties"]["value"],
        json!({ "$ref": "#/$defs/Value", "description": "Referenced value" })
    );
    assert_eq!(
        result["$defs"],
        json!({
            "Value": { "type": "string", "description": "Definition description" },
            "Unused": { "type": "number" }
        })
    );
}

#[test]
fn openai_does_not_sanitize_non_openai_providers() {
    let result = schema(
        "anthropic",
        "claude-sonnet-4",
        "@ai-sdk/anthropic",
        &json!({
            "type": "object",
            "properties": { "query": { "type": "string", "pattern": "^https://" } }
        }),
    );
    assert_eq!(result["properties"]["query"]["pattern"], json!("^https://"));
}

#[test]
fn openai_sanitizes_openai_and_azure_models() {
    for (provider_id, npm) in [
        ("opencode", "@ai-sdk/openai"),
        ("custom-openai-compatible", "@ai-sdk/openai"),
        ("azure", "@ai-sdk/azure"),
    ] {
        let result = schema(
            provider_id,
            "custom-model",
            npm,
            &json!({
                "type": "object",
                "properties": { "query": { "type": "string", "pattern": "^https://" } }
            }),
        );
        assert_eq!(
            result,
            json!({ "type": "object", "properties": { "query": { "type": "string" } } }),
            "provider {provider_id}"
        );
    }
}

#[test]
fn moonshot_removes_sibling_descriptions_from_refs() {
    let result = schema(
        "moonshotai",
        "kimi-k2",
        "",
        &json!({
            "type": "object",
            "properties": {
                "variantOptions": {
                    "$ref": "#/$defs/VariantOptions",
                    "description": "Required. The variant options for generation."
                }
            },
            "required": ["variantOptions"],
            "$defs": {
                "VariantOptions": {
                    "description": "Configuration options for design variant generation.",
                    "type": "object"
                }
            },
            "additionalProperties": false
        }),
    );
    assert_eq!(
        result["properties"]["variantOptions"],
        json!({ "$ref": "#/$defs/VariantOptions" })
    );
    assert_eq!(
        result["$defs"]["VariantOptions"]["description"],
        json!("Configuration options for design variant generation.")
    );
}

#[test]
fn moonshot_runs_for_kimi_models_outside_the_provider() {
    let result = schema(
        "openrouter",
        "moonshotai/kimi-k2",
        "",
        &json!({
            "type": "object",
            "properties": {
                "value": { "$ref": "#/$defs/Value", "description": "Moonshot rejects this sibling." }
            },
            "$defs": {
                "Value": { "description": "Referenced schema description stays here.", "type": "object" }
            }
        }),
    );
    assert_eq!(
        result["properties"]["value"],
        json!({ "$ref": "#/$defs/Value" })
    );
}

#[test]
fn moonshot_converts_tuple_items_to_a_single_item_schema() {
    let result = schema(
        "moonshotai",
        "kimi-k2",
        "",
        &json!({
            "type": "object",
            "properties": {
                "codeSpec": {
                    "type": "object",
                    "properties": {
                        "accessibility": {
                            "type": "object",
                            "properties": {
                                "renderedSize": {
                                    "type": "array",
                                    "items": [{ "type": "number" }, { "type": "number" }],
                                    "minItems": 2,
                                    "maxItems": 2
                                }
                            }
                        }
                    }
                }
            }
        }),
    );
    assert_eq!(
        result["properties"]["codeSpec"]["properties"]["accessibility"]["properties"]
            ["renderedSize"]["items"],
        json!({ "type": "number" })
    );
}
