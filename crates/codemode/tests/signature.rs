//! Port of packages/codemode/test/signature.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/codemode/src/tool-schema.ts: JSON Schema (and
//! annotation-carrying Effect schemas) render to TypeScript signatures, with field
//! descriptions and constraints surfacing as JSDoc, non-identifier keys quoted, unions
//! and intersections preserved, and cyclic/deep schemas staying total.
//! Re-derived: Effect `Schema.Struct` inputs are represented as the JSON Schema they
//! emit; the `$codemode.search`/`instructions` cases run through `CodeMode::execute`.
//! The schema renderer is implemented; the two search cases remain red until the
//! interpreter lands.

use opencode_codemode::tool_schema::{
    input_type_script, json_schema_to_type_script, output_type_script, Tool,
};
use opencode_codemode::{CodeMode, CodeModeResult};
use serde_json::{json, Value};

const INTERP_NOTE: &str = "porting: code-mode interpreter not implemented";

fn run(code: &str) -> CodeModeResult {
    CodeMode::execute(code).expect(INTERP_NOTE)
}

fn value(code: &str) -> Value {
    let result = run(code);
    assert!(result.ok, "expected success, got {:?}", result.error);
    result.value.expect("successful result carries a value")
}

fn list_issues() -> Tool {
    Tool {
        input: json!({
            "type": "object",
            "properties": {
                "owner": { "type": "string", "description": "Repository owner" },
                "after": { "type": "string", "description": "Cursor from the previous response's pageInfo" },
                "perPage": { "type": "number", "description": "Results per page", "default": 30 },
                "labels": { "type": "array", "items": { "type": "string" }, "description": "Filter by labels", "minItems": 1, "maxItems": 10 },
                "state": { "type": "string", "enum": ["open", "closed"] }
            },
            "required": ["owner"]
        }),
        output: None,
    }
}

fn lookup_order() -> Tool {
    Tool {
        input: json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Order identifier" },
                "verbose": { "type": "boolean" }
            },
            "required": ["id"]
        }),
        output: Some(json!({
            "type": "object",
            "properties": { "status": { "type": "string", "description": "Current order status" } },
            "required": ["status"]
        })),
    }
}

#[test]
fn described_fields_get_jsdoc_comments_undescribed_and_untagged_fields_get_none() {
    let expected = [
        "{",
        "  /** Repository owner */",
        "  owner: string,",
        "  /** Cursor from the previous response's pageInfo */",
        "  after?: string,",
        "  /**",
        "   * Results per page",
        "   * @default 30",
        "   */",
        "  perPage?: number,",
        "  /**",
        "   * Filter by labels",
        "   * @minItems 1",
        "   * @maxItems 10",
        "   */",
        "  labels?: Array<string>,",
        "  state?: \"open\" | \"closed\",",
        "}",
    ]
    .join("\n");
    assert_eq!(input_type_script(&list_issues(), true), expected);
}

#[test]
fn compact_mode_output_is_unchanged_by_the_pretty_machinery() {
    assert_eq!(
        input_type_script(&list_issues(), false),
        "{ owner: string; after?: string; perPage?: number; labels?: Array<string>; state?: \"open\" | \"closed\" }"
    );
    assert_eq!(
        input_type_script(&lookup_order(), false),
        "{ id: string; verbose?: boolean }"
    );
    assert_eq!(
        output_type_script(&lookup_order(), false),
        "{ status: string }"
    );
}

#[test]
fn nested_objects_recurse_with_increasing_indent_and_their_own_jsdoc() {
    let schema = json!({
        "type": "object",
        "properties": {
            "filter": {
                "type": "object",
                "description": "Search filter",
                "properties": { "state": { "type": "string", "description": "Issue state" } }
            }
        }
    });
    let expected = [
        "{",
        "  /** Search filter */",
        "  filter?: {",
        "    /** Issue state */",
        "    state?: string,",
        "  },",
        "}",
    ]
    .join("\n");
    assert_eq!(json_schema_to_type_script(&schema, true), expected);
}

#[test]
fn effect_schema_annotations_become_jsdoc_on_input_and_output_fields() {
    assert_eq!(
        input_type_script(&lookup_order(), true),
        [
            "{",
            "  /** Order identifier */",
            "  id: string,",
            "  verbose?: boolean,",
            "}"
        ]
        .join("\n")
    );
    assert_eq!(
        output_type_script(&lookup_order(), true),
        [
            "{",
            "  /** Current order status */",
            "  status: string,",
            "}"
        ]
        .join("\n")
    );
}

#[test]
fn constraints_typescript_cannot_express_surface_as_jsdoc_tags() {
    let schema = json!({
        "type": "object",
        "properties": {
            "legacy": { "type": "string", "deprecated": true },
            "homepage": { "type": "string", "format": "uri" },
            "tags": { "type": "array", "items": { "type": "string" }, "minItems": 2, "maxItems": 5, "default": ["a", "b"] }
        }
    });
    let pretty = json_schema_to_type_script(&schema, true);
    assert!(pretty.contains("  /** @deprecated */\n  legacy?: string"));
    assert!(pretty.contains("  /** @format uri */\n  homepage?: string"));
    assert!(pretty.contains(
        [
            "  /**",
            "   * @default [\"a\",\"b\"]",
            "   * @minItems 2",
            "   * @maxItems 5",
            "   */",
            "  tags?: Array<string>"
        ]
        .join("\n")
        .as_str()
    ));
}

#[test]
fn neutralizes_comment_closers_inside_descriptions() {
    let schema = json!({
        "type": "object",
        "properties": { "note": { "type": "string", "description": "Ends */ early" } }
    });
    let pretty = json_schema_to_type_script(&schema, true);
    assert!(pretty.contains("  /** Ends * / early */"));
    assert!(!pretty.contains("Ends */"));
}

#[test]
fn multiline_descriptions_become_star_prefixed_blocks_with_blank_edges_trimmed() {
    let schema = json!({
        "type": "object",
        "properties": { "query": { "type": "string", "description": "\nFirst line\n\nSecond line\n" } }
    });
    let expected = [
        "{",
        "  /**",
        "   * First line",
        "   *",
        "   * Second line",
        "   */",
        "  query?: string,",
        "}",
    ]
    .join("\n");
    assert_eq!(json_schema_to_type_script(&schema, true), expected);
}

#[test]
fn stays_total_on_cyclic_refs_and_pathological_nesting_in_both_modes() {
    let cyclic = json!({
        "$ref": "#/$defs/Node",
        "$defs": { "Node": { "type": "object", "properties": { "child": { "$ref": "#/$defs/Node" }, "name": { "type": "string" } } } }
    });
    assert_eq!(
        json_schema_to_type_script(&cyclic, false),
        "{ child?: unknown; name?: string }"
    );
    assert!(json_schema_to_type_script(&cyclic, true).contains("child?: unknown"));

    let mut deep = json!({ "type": "string" });
    for _ in 0..12 {
        deep = json!({ "type": "object", "properties": { "next": deep } });
    }
    for pretty in [false, true] {
        let rendered = json_schema_to_type_script(&deep, pretty);
        assert!(rendered.contains("unknown"));
        assert!(rendered.contains("next?:"));
    }
}

#[test]
fn intersects_ref_and_union_siblings_instead_of_discarding_them() {
    assert_eq!(
        json_schema_to_type_script(
            &json!({
                "$ref": "#/$defs/User",
                "properties": { "active": { "type": "boolean" } },
                "required": ["active"],
                "$defs": { "User": { "type": "object", "properties": { "id": { "type": "string" } }, "required": ["id"] } }
            }),
            false
        ),
        "{ id: string } & { active: boolean }"
    );
    assert_eq!(
        json_schema_to_type_script(
            &json!({
                "type": "object",
                "properties": { "common": { "type": "boolean" } },
                "required": ["common"],
                "anyOf": [
                    { "type": "object", "properties": { "name": { "type": "string" } }, "required": ["name"] },
                    { "type": "object", "properties": { "count": { "type": "number" } }, "required": ["count"] }
                ]
            }),
            false
        ),
        "({ name: string } | { count: number }) & { common: boolean }"
    );
    assert_eq!(
        json_schema_to_type_script(&json!({ "$ref": "https://example.com/schema.json" }), false),
        "unknown"
    );
    assert_eq!(
        json_schema_to_type_script(
            &json!({
                "$ref": "#/$defs/User/properties/id",
                "$defs": { "User": { "type": "object" }, "id": { "type": "string" } }
            }),
            false
        ),
        "unknown"
    );
    assert_eq!(
        json_schema_to_type_script(
            &json!({ "type": ["object", "null"], "properties": { "name": { "type": "string" } } }),
            false
        ),
        "{ name?: string } | null"
    );
}

#[test]
fn compact_rendering_quotes_non_identifier_keys_and_leaves_identifiers_bare() {
    assert_eq!(
        json_schema_to_type_script(&raw_schema(), false),
        "{ \"123\"?: number; \"foo-bar\"?: string; \"@type\": string; \"x.y\"?: number; plain?: boolean }"
    );
}

#[test]
fn pretty_rendering_quotes_non_identifier_keys_and_keeps_their_jsdoc() {
    let expected = [
        "{",
        "  \"123\"?: number,",
        "  \"foo-bar\"?: string,",
        "  \"@type\": string,",
        "  /** Dotted name */",
        "  \"x.y\"?: number,",
        "  plain?: boolean,",
        "}",
    ]
    .join("\n");
    assert_eq!(json_schema_to_type_script(&raw_schema(), true), expected);
}

#[test]
fn json_schema_input_and_output_signatures_of_a_tool_both_quote() {
    let tool = Tool {
        input: raw_schema(),
        output: Some(json!({
            "type": "object",
            "properties": { "content-type": { "type": "string" } },
            "required": ["content-type"]
        })),
    };
    assert!(input_type_script(&tool, false).contains("\"foo-bar\"?: string"));
    assert_eq!(
        output_type_script(&tool, false),
        "{ \"content-type\": string }"
    );
    assert_eq!(
        output_type_script(&tool, true),
        ["{", "  \"content-type\": string,", "}"].join("\n")
    );
}

#[test]
fn effect_schema_structs_with_non_identifier_field_names_quote_too() {
    let tool = Tool {
        input: json!({
            "type": "object",
            "properties": { "foo-bar": { "type": "string" }, "plain": { "type": "number" } },
            "required": ["foo-bar"]
        }),
        output: None,
    };
    assert_eq!(
        input_type_script(&tool, false),
        "{ \"foo-bar\": string; plain?: number }"
    );
    assert_eq!(
        input_type_script(&tool, true),
        ["{", "  \"foo-bar\": string,", "  plain?: number,", "}"].join("\n")
    );
}

#[test]
fn union_schemas_render_every_alternative() {
    assert_eq!(
        json_schema_to_type_script(
            &json!({ "anyOf": [{ "type": "string" }, { "type": "number" }] }),
            false
        ),
        "string | number"
    );
    assert_eq!(
        json_schema_to_type_script(
            &json!({ "oneOf": [{ "type": "number" }, { "type": "null" }] }),
            false
        ),
        "number | null"
    );
    let tool = Tool {
        input: json!({
            "type": "object",
            "properties": { "value": { "anyOf": [{ "type": "string" }, { "type": "number" }] } }
        }),
        output: Some(json!({ "anyOf": [{ "type": "number" }, { "type": "boolean" }] })),
    };
    assert_eq!(
        input_type_script(&tool, false),
        "{ value?: string | number }"
    );
    assert_eq!(output_type_script(&tool, false), "number | boolean");
}

#[test]
fn allof_renders_intersections_with_parenthesized_union_members() {
    assert_eq!(
        json_schema_to_type_script(
            &json!({ "allOf": [{ "type": "object", "properties": { "id": { "type": "string" } } }, { "type": ["string", "null"] }] }),
            false
        ),
        "{ id?: string } & (string | null)"
    );
    assert_eq!(
        json_schema_to_type_script(
            &json!({ "allOf": [{ "type": "string" }, { "$ref": "https://example.com/external.json" }] }),
            false
        ),
        "unknown"
    );
    assert_eq!(
        json_schema_to_type_script(
            &json!({ "allOf": [{ "type": "string" }, { "allOf": [{ "$ref": "https://example.com/external.json" }] }] }),
            false
        ),
        "unknown"
    );
    assert_eq!(
        json_schema_to_type_script(
            &json!({ "type": "string", "allOf": [{ "$ref": "#/$defs/Constraint" }], "$defs": { "Constraint": { "description": "TypeScript-neutral constraint" } } }),
            false
        ),
        "string"
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn search_result_signatures_carry_field_jsdoc_and_tags() {
    let result = value(
        r#"
        const items = await tools.$codemode.search({ query: "list issues repository" })
        return items
    "#,
    );
    let item = result["items"]
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .find(|item| item["path"] == "tools.github.list_issues")
        })
        .expect("list_issues signature");
    assert!(item["signature"]
        .as_str()
        .expect("signature")
        .contains("/** Repository owner */"));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn search_results_return_callable_bracket_notation_paths_and_signatures() {
    let result = value(
        r#"
        const items = await tools.$codemode.search({ query: "resolve library" })
        return items
    "#,
    );
    let item = &result["items"][0];
    assert_eq!(
        item["path"],
        json!("tools.context7[\"resolve-library-id\"]")
    );
    assert!(item["signature"]
        .as_str()
        .expect("signature")
        .contains("tools.context7[\"resolve-library-id\"](input: {"));
}

fn raw_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "foo-bar": { "type": "string" },
            "@type": { "type": "string" },
            "x.y": { "type": "number", "description": "Dotted name" },
            "123": { "type": "number" },
            "plain": { "type": "boolean" }
        },
        "required": ["@type"]
    })
}
