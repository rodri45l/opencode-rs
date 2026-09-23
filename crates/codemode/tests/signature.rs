//! Port of packages/codemode/test/signature.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/codemode/src/tool-schema.ts: JSON Schema (and
//! annotation-carrying Effect schemas) render to TypeScript signatures, with field
//! descriptions and constraints surfacing as JSDoc, non-identifier keys quoted, unions
//! and intersections preserved, and cyclic/deep schemas staying total.
//! Re-derived: Effect `Schema.Struct` inputs are represented as the JSON Schema they
//! emit; the `$codemode.search`/`instructions` cases run through `CodeMode::execute`.
//! Red-first: the schema renderer and interpreter are not implemented.

#[allow(dead_code)]
mod tool_schema {
    use serde_json::Value;
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: code-mode tool schema rendering not implemented";

    #[derive(Debug, Clone)]
    pub struct Tool {
        pub input: Value,
        pub output: Option<Value>,
    }

    pub fn json_schema_to_type_script(_schema: &Value, _pretty: bool) -> PortResult<String> {
        Err(NotImplemented(NOTE))
    }

    pub fn input_type_script(_tool: &Tool, _pretty: bool) -> PortResult<String> {
        Err(NotImplemented(NOTE))
    }

    pub fn output_type_script(_tool: &Tool, _pretty: bool) -> PortResult<String> {
        Err(NotImplemented(NOTE))
    }
}

use opencode_codemode::{CodeMode, CodeModeResult};
use serde_json::{json, Value};
use tool_schema::{input_type_script, json_schema_to_type_script, output_type_script, Tool, NOTE};

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
#[ignore = "porting: code-mode tool schema rendering not implemented"]
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
    assert_eq!(
        input_type_script(&list_issues(), true).expect(NOTE),
        expected
    );
}

#[test]
#[ignore = "porting: code-mode tool schema rendering not implemented"]
fn compact_mode_output_is_unchanged_by_the_pretty_machinery() {
    assert_eq!(
        input_type_script(&list_issues(), false).expect(NOTE),
        "{ owner: string; after?: string; perPage?: number; labels?: Array<string>; state?: \"open\" | \"closed\" }"
    );
    assert_eq!(
        input_type_script(&lookup_order(), false).expect(NOTE),
        "{ id: string; verbose?: boolean }"
    );
    assert_eq!(
        output_type_script(&lookup_order(), false).expect(NOTE),
        "{ status: string }"
    );
}

#[test]
#[ignore = "porting: code-mode tool schema rendering not implemented"]
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
    assert_eq!(
        json_schema_to_type_script(&schema, true).expect(NOTE),
        expected
    );
}

#[test]
#[ignore = "porting: code-mode tool schema rendering not implemented"]
fn effect_schema_annotations_become_jsdoc_on_input_and_output_fields() {
    assert_eq!(
        input_type_script(&lookup_order(), true).expect(NOTE),
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
        output_type_script(&lookup_order(), true).expect(NOTE),
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
#[ignore = "porting: code-mode tool schema rendering not implemented"]
fn constraints_typescript_cannot_express_surface_as_jsdoc_tags() {
    let schema = json!({
        "type": "object",
        "properties": {
            "legacy": { "type": "string", "deprecated": true },
            "homepage": { "type": "string", "format": "uri" },
            "tags": { "type": "array", "items": { "type": "string" }, "minItems": 2, "maxItems": 5, "default": ["a", "b"] }
        }
    });
    let pretty = json_schema_to_type_script(&schema, true).expect(NOTE);
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
#[ignore = "porting: code-mode tool schema rendering not implemented"]
fn neutralizes_comment_closers_inside_descriptions() {
    let schema = json!({
        "type": "object",
        "properties": { "note": { "type": "string", "description": "Ends */ early" } }
    });
    let pretty = json_schema_to_type_script(&schema, true).expect(NOTE);
    assert!(pretty.contains("  /** Ends * / early */"));
    assert!(!pretty.contains("Ends */"));
}

#[test]
#[ignore = "porting: code-mode tool schema rendering not implemented"]
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
    assert_eq!(
        json_schema_to_type_script(&schema, true).expect(NOTE),
        expected
    );
}

#[test]
#[ignore = "porting: code-mode tool schema rendering not implemented"]
fn stays_total_on_cyclic_refs_and_pathological_nesting_in_both_modes() {
    let cyclic = json!({
        "$ref": "#/$defs/Node",
        "$defs": { "Node": { "type": "object", "properties": { "child": { "$ref": "#/$defs/Node" }, "name": { "type": "string" } } } }
    });
    assert_eq!(
        json_schema_to_type_script(&cyclic, false).expect(NOTE),
        "{ child?: unknown; name?: string }"
    );
    assert!(json_schema_to_type_script(&cyclic, true)
        .expect(NOTE)
        .contains("child?: unknown"));

    let mut deep = json!({ "type": "string" });
    for _ in 0..12 {
        deep = json!({ "type": "object", "properties": { "next": deep } });
    }
    for pretty in [false, true] {
        let rendered = json_schema_to_type_script(&deep, pretty).expect(NOTE);
        assert!(rendered.contains("unknown"));
        assert!(rendered.contains("next?:"));
    }
}

#[test]
#[ignore = "porting: code-mode tool schema rendering not implemented"]
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
        )
        .expect(NOTE),
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
        )
        .expect(NOTE),
        "({ name: string } | { count: number }) & { common: boolean }"
    );
    assert_eq!(
        json_schema_to_type_script(&json!({ "$ref": "https://example.com/schema.json" }), false)
            .expect(NOTE),
        "unknown"
    );
    assert_eq!(
        json_schema_to_type_script(
            &json!({
                "$ref": "#/$defs/User/properties/id",
                "$defs": { "User": { "type": "object" }, "id": { "type": "string" } }
            }),
            false
        )
        .expect(NOTE),
        "unknown"
    );
    assert_eq!(
        json_schema_to_type_script(
            &json!({ "type": ["object", "null"], "properties": { "name": { "type": "string" } } }),
            false
        )
        .expect(NOTE),
        "{ name?: string } | null"
    );
}

#[test]
#[ignore = "porting: code-mode tool schema rendering not implemented"]
fn compact_rendering_quotes_non_identifier_keys_and_leaves_identifiers_bare() {
    assert_eq!(
        json_schema_to_type_script(&raw_schema(), false).expect(NOTE),
        "{ \"123\"?: number; \"foo-bar\"?: string; \"@type\": string; \"x.y\"?: number; plain?: boolean }"
    );
}

#[test]
#[ignore = "porting: code-mode tool schema rendering not implemented"]
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
    assert_eq!(
        json_schema_to_type_script(&raw_schema(), true).expect(NOTE),
        expected
    );
}

#[test]
#[ignore = "porting: code-mode tool schema rendering not implemented"]
fn json_schema_input_and_output_signatures_of_a_tool_both_quote() {
    let tool = Tool {
        input: raw_schema(),
        output: Some(json!({
            "type": "object",
            "properties": { "content-type": { "type": "string" } },
            "required": ["content-type"]
        })),
    };
    assert!(input_type_script(&tool, false)
        .expect(NOTE)
        .contains("\"foo-bar\"?: string"));
    assert_eq!(
        output_type_script(&tool, false).expect(NOTE),
        "{ \"content-type\": string }"
    );
    assert_eq!(
        output_type_script(&tool, true).expect(NOTE),
        ["{", "  \"content-type\": string,", "}"].join("\n")
    );
}

#[test]
#[ignore = "porting: code-mode tool schema rendering not implemented"]
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
        input_type_script(&tool, false).expect(NOTE),
        "{ \"foo-bar\": string; plain?: number }"
    );
    assert_eq!(
        input_type_script(&tool, true).expect(NOTE),
        ["{", "  \"foo-bar\": string,", "  plain?: number,", "}"].join("\n")
    );
}

#[test]
#[ignore = "porting: code-mode tool schema rendering not implemented"]
fn union_schemas_render_every_alternative() {
    assert_eq!(
        json_schema_to_type_script(
            &json!({ "anyOf": [{ "type": "string" }, { "type": "number" }] }),
            false
        )
        .expect(NOTE),
        "string | number"
    );
    assert_eq!(
        json_schema_to_type_script(
            &json!({ "oneOf": [{ "type": "number" }, { "type": "null" }] }),
            false
        )
        .expect(NOTE),
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
        input_type_script(&tool, false).expect(NOTE),
        "{ value?: string | number }"
    );
    assert_eq!(
        output_type_script(&tool, false).expect(NOTE),
        "number | boolean"
    );
}

#[test]
#[ignore = "porting: code-mode tool schema rendering not implemented"]
fn allof_renders_intersections_with_parenthesized_union_members() {
    assert_eq!(
        json_schema_to_type_script(
            &json!({ "allOf": [{ "type": "object", "properties": { "id": { "type": "string" } } }, { "type": ["string", "null"] }] }),
            false
        )
        .expect(NOTE),
        "{ id?: string } & (string | null)"
    );
    assert_eq!(
        json_schema_to_type_script(
            &json!({ "allOf": [{ "type": "string" }, { "$ref": "https://example.com/external.json" }] }),
            false
        )
        .expect(NOTE),
        "unknown"
    );
    assert_eq!(
        json_schema_to_type_script(
            &json!({ "allOf": [{ "type": "string" }, { "allOf": [{ "$ref": "https://example.com/external.json" }] }] }),
            false
        )
        .expect(NOTE),
        "unknown"
    );
    assert_eq!(
        json_schema_to_type_script(
            &json!({ "type": "string", "allOf": [{ "$ref": "#/$defs/Constraint" }], "$defs": { "Constraint": { "description": "TypeScript-neutral constraint" } } }),
            false
        )
        .expect(NOTE),
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
