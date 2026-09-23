//! Port of packages/opencode/test/tool/code-mode.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the `execute` (Code Mode) parameter contract, the pure
//! catalog/instruction renderer (`describeCatalog`), hard-deny permission
//! visibility, and the MCP-result projection that strips media into
//! attachments. The confined JS interpreter itself is not ported here (it needs
//! a JavaScript engine), so the execution-path tests are dropped.

#![allow(dead_code)]

use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
enum CodeModeError {
    NotImplemented(&'static str),
    InvalidParameters(&'static str),
    InvalidInput(&'static str),
}

type Result<T> = std::result::Result<T, CodeModeError>;

#[derive(Debug, Clone, PartialEq)]
struct CatalogTool {
    key: String,
    description: String,
    input_schema: Value,
    output_schema: Option<Value>,
}

impl CatalogTool {
    fn new(
        key: &str,
        description: &str,
        input_schema: Value,
        output_schema: Option<Value>,
    ) -> Self {
        Self {
            key: key.to_string(),
            description: description.to_string(),
            input_schema,
            output_schema,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionRule {
    permission: String,
    pattern: String,
    action: String,
}

impl PermissionRule {
    fn deny(permission: &str) -> Self {
        Self {
            permission: permission.to_string(),
            pattern: "*".to_string(),
            action: "deny".to_string(),
        }
    }

    fn ask(permission: &str) -> Self {
        Self {
            permission: permission.to_string(),
            pattern: "*".to_string(),
            action: "ask".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum McpBlock {
    Text(String),
    Image {
        data: String,
        mime: String,
    },
    Resource {
        uri: String,
        mime: Option<String>,
        blob: String,
    },
    ResourceLink {
        uri: String,
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct Projected {
    value: Value,
    attachments: Vec<Value>,
}

fn base_description() -> &'static str {
    "Run a confined orchestration script with access to connected MCP tools."
}

fn parameters_json_schema() -> Result<Value> {
    Err(CodeModeError::NotImplemented("code-mode parameters schema"))
}

fn validate_parameters(_input: &Value) -> Result<Value> {
    Err(CodeModeError::NotImplemented(
        "code-mode parameters validation",
    ))
}

fn visible_tools(
    _tools: &[CatalogTool],
    _permission: &[PermissionRule],
) -> Result<Vec<CatalogTool>> {
    Err(CodeModeError::NotImplemented("Permission.visibleTools"))
}

fn describe_catalog(_tools: &[CatalogTool], _servers: &[String]) -> Result<String> {
    Err(CodeModeError::NotImplemented("CodeMode.describeCatalog"))
}

fn describe_for(
    tools: &[CatalogTool],
    servers: &[String],
    permission: &[PermissionRule],
) -> Result<String> {
    let visible = visible_tools(tools, permission)?;
    describe_catalog(&visible, servers)
}

fn project_mcp_result(_blocks: &[McpBlock], _structured: Option<Value>) -> Result<Projected> {
    Err(CodeModeError::NotImplemented(
        "code-mode MCP result projection",
    ))
}

fn metadata_input(_input: &Value) -> Result<Option<Value>> {
    Err(CodeModeError::NotImplemented(
        "code-mode tool-call metadata input",
    ))
}

fn tool(key: &str, description: &str) -> CatalogTool {
    CatalogTool::new(
        key,
        description,
        json!({ "type": "object", "properties": {} }),
        None,
    )
}

fn attachment(mime: &str, url: &str, filename: Option<&str>) -> Value {
    match filename {
        Some(name) => json!({ "type": "file", "mime": mime, "url": url, "filename": name }),
        None => json!({ "type": "file", "mime": mime, "url": url }),
    }
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn parameters_require_code_with_description() {
    assert_eq!(
        validate_parameters(&json!({ "code": "return 1" })).expect("code-mode ported"),
        json!({ "code": "return 1" })
    );
    assert!(matches!(
        validate_parameters(&json!({})),
        Err(CodeModeError::InvalidParameters(_))
    ));
    let schema = parameters_json_schema().expect("code-mode ported");
    assert_eq!(
        schema["properties"]["code"]["description"],
        json!("Script body executed by the confined interpreter.")
    );
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn groups_multi_underscore_server_by_longest_prefix() {
    let description = describe_for(
        &[tool("my_server_do_thing", "do thing")],
        &["my_server".to_string()],
        &[],
    )
    .expect("code-mode ported");
    assert!(description.contains("- my_server (1 tool)"));
    assert!(description.contains("tools.my_server.do_thing("));
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn group_by_server_uses_whole_key_when_no_underscore() {
    let description =
        describe_for(&[tool("standalone", "standalone")], &[], &[]).expect("code-mode ported");
    assert!(description.contains("- standalone (1 tool)"));
    assert!(description.contains("tools.standalone.standalone("));
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn catalog_carries_raw_mcp_schemas() {
    let description = describe_for(
        &[CatalogTool::new(
            "weather_current",
            "current",
            json!({ "type": "object", "properties": { "city": { "type": "string" } }, "required": ["city"] }),
            Some(json!({ "type": "object", "properties": { "tempC": { "type": "number" } }, "required": ["tempC"] })),
        )],
        &["weather".to_string()],
        &[],
    )
    .expect("code-mode ported");
    assert!(description.contains(
        "tools.weather.current(input: {\n  city: string,\n}): Promise<{\n  tempC: number,\n}>"
    ));
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn static_base_description_carries_no_catalog() {
    assert_eq!(
        base_description(),
        "Run a confined orchestration script with access to connected MCP tools."
    );
    assert!(!base_description().contains("Available tools"));
    assert!(!base_description().contains("list_issues"));
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn small_catalog_inlines_every_full_signature() {
    let tools = vec![
        CatalogTool::new(
            "github_create_issue",
            "create_issue",
            json!({
                "type": "object",
                "properties": { "title": { "type": "string" }, "body": { "type": "string" } },
                "required": ["title"]
            }),
            None,
        ),
        tool("github_list_issues", "list_issues"),
        tool("linear_search", "search"),
    ];
    let description = describe_for(&tools, &["github".to_string(), "linear".to_string()], &[])
        .expect("code-mode ported");

    assert!(description.contains("Available tools (COMPLETE list"));
    assert!(description.contains("- github (2 tools)"));
    assert!(description.contains("- linear (1 tool)"));
    assert!(description.contains(
        "tools.github.create_issue(input: {\n  title: string,\n  body?: string,\n}): Promise<unknown>"
    ));
    assert!(description.contains("tools.github.list_issues("));
    assert!(description.contains("tools.linear.search("));
    assert!(description.contains("tools.linear.search(input: {}): Promise<unknown>"));
    assert!(!description.contains("$codemode"));
    assert!(!description.contains("Browse one namespace"));
    assert!(description.contains("## Workflow"));
    assert!(description.contains("1. Pick a tool from the list under `## Available tools`"));
    assert!(!description.contains("JSON.parse(res)"));
    assert!(description.contains("check that it is a non-null object and not an array"));
    assert!(description.contains("Return only the fields you need"));
    assert!(!description.contains("total_count"));
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn signatures_render_declared_output_schema_as_return_type() {
    let description = describe_for(
        &[CatalogTool::new(
            "weather_current",
            "current",
            json!({ "type": "object", "properties": { "city": { "type": "string" } }, "required": ["city"] }),
            Some(json!({
                "type": "object",
                "properties": { "tempC": { "type": "number" }, "summary": { "type": "string" } },
                "required": ["tempC"]
            })),
        )],
        &[],
        &[],
    )
    .expect("code-mode ported");
    assert!(description.contains(
        "tools.weather.current(input: {\n  city: string,\n}): Promise<{\n  tempC: number,\n  summary?: string,\n}>"
    ));
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn large_catalogs_inline_budgeted_partial_list_plus_search() {
    let mut tools = Vec::new();
    let filler =
        "a searchable description of this operation that consumes catalog budget ".repeat(3);
    for i in 0..150 {
        tools.push(CatalogTool::new(
            &format!("alpha_op_{i}"),
            &format!("{filler}{i}"),
            json!({ "type": "object", "properties": { "value": { "type": "string" }, "count": { "type": "number" } } }),
            None,
        ));
    }
    tools.push(CatalogTool::new(
        "zeta_only_tool",
        "only_tool",
        json!({
            "type": "object",
            "properties": { "topic": { "type": "string", "description": "Subject to look up" } },
            "required": ["topic"]
        }),
        None,
    ));
    let description = describe_for(&tools, &["alpha".to_string(), "zeta".to_string()], &[])
        .expect("code-mode ported");

    assert!(description.contains("Available tools (PARTIAL - "));
    assert!(description.contains("- alpha (150 tools, "));
    assert!(description.contains(" shown)"));
    assert!(description.contains("- zeta (1 tool)\n"));
    assert!(description.contains(
        "tools.zeta.only_tool(input: {\n  /** Subject to look up */\n  topic: string,\n}): Promise<unknown>"
    ));
    assert!(description.contains("tools.$codemode.search("));
    assert!(description.contains("  limit?: number,\n  offset?: number,"));
    assert!(description.contains("  remaining: number,\n  next: {"));
    assert!(description.contains("      offset: number,\n    } | null,"));
    assert!(description.contains(
        "1. If needed, discover tools: `return await tools.$codemode.search({ query: \"<intent + key nouns>\" })`."
    ));
    assert!(description.contains(
        "- Browse one namespace: `await tools.$codemode.search({ query: \"\", namespace: \"<name>\" })`."
    ));
    assert!(!description.contains("total_count"));
    assert!(description.contains("tools.alpha.op_0("));
    assert!(!description.contains("tools.alpha.op_99("));
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn hard_denied_tool_never_enters_the_catalog() {
    let tools = vec![
        tool("github_create_issue", "create_issue"),
        tool("github_list_issues", "list_issues"),
    ];
    let description = describe_for(
        &tools,
        &["github".to_string()],
        &[PermissionRule::deny("github_create_issue")],
    )
    .expect("code-mode ported");
    assert!(description.contains("tools.github.list_issues("));
    assert!(!description.contains("create_issue"));
    assert!(description.contains("- github (1 tool)"));
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn ask_level_tool_stays_fully_visible() {
    let tools = vec![
        tool("github_create_issue", "create_issue"),
        tool("github_list_issues", "list_issues"),
    ];
    let description = describe_for(
        &tools,
        &["github".to_string()],
        &[PermissionRule::ask("github_create_issue")],
    )
    .expect("code-mode ported");
    assert!(description.contains("tools.github.create_issue("));
    assert!(description.contains("tools.github.list_issues("));
    assert!(description.contains("- github (2 tools)"));
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn visible_tools_hides_only_hard_denies() {
    let tools = vec![
        tool("a_tool", "a"),
        tool("b_tool", "b"),
        tool("c_tool", "c"),
    ];
    let mut c_deny = PermissionRule::deny("c_tool");
    c_deny.pattern = "something".to_string();
    let visible = visible_tools(
        &tools,
        &[
            PermissionRule::deny("a_tool"),
            PermissionRule::ask("b_tool"),
            c_deny,
        ],
    )
    .expect("code-mode ported");
    let keys: Vec<String> = visible.into_iter().map(|tool| tool.key).collect();
    assert_eq!(keys, vec!["b_tool".to_string(), "c_tool".to_string()]);
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn structured_content_is_exposed_natively() {
    let projected = project_mcp_result(
        &[McpBlock::Text("ignored".to_string())],
        Some(json!({ "sum": 3 })),
    )
    .expect("code-mode ported");
    assert_eq!(projected.value, json!({ "sum": 3 }));
    assert!(projected.attachments.is_empty());
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn media_only_result_returns_a_marker() {
    let projected = project_mcp_result(
        &[McpBlock::Image {
            data: "PNGDATA".to_string(),
            mime: "image/png".to_string(),
        }],
        None,
    )
    .expect("code-mode ported");
    assert_eq!(projected.value, json!("[1 image attached to the result]"));
    assert_eq!(
        projected.attachments,
        vec![attachment(
            "image/png",
            "data:image/png;base64,PNGDATA",
            None
        )]
    );
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn media_only_markers_distinguish_images_from_mixed_files() {
    let images = project_mcp_result(
        &[
            McpBlock::Image {
                data: "PNG1".to_string(),
                mime: "image/png".to_string(),
            },
            McpBlock::Image {
                data: "PNG2".to_string(),
                mime: "image/png".to_string(),
            },
        ],
        None,
    )
    .expect("code-mode ported");
    assert_eq!(images.value, json!("[2 images attached to the result]"));

    let mixed = project_mcp_result(
        &[
            McpBlock::Image {
                data: "PNG3".to_string(),
                mime: "image/png".to_string(),
            },
            McpBlock::Resource {
                uri: "file:///tmp/report.pdf".to_string(),
                mime: Some("application/pdf".to_string()),
                blob: "PDF1".to_string(),
            },
        ],
        None,
    )
    .expect("code-mode ported");
    assert_eq!(mixed.value, json!("[2 files attached to the result]"));
    assert_eq!(
        mixed.attachments,
        vec![
            attachment("image/png", "data:image/png;base64,PNG3", None),
            attachment(
                "application/pdf",
                "data:application/pdf;base64,PDF1",
                Some("report.pdf")
            ),
        ]
    );
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn resource_links_flow_to_the_program_as_text() {
    let projected = project_mcp_result(
        &[
            McpBlock::ResourceLink {
                uri: "https://example.com/guide.pdf".to_string(),
                name: "guide.pdf".to_string(),
            },
            McpBlock::ResourceLink {
                uri: "file:///tmp/notes.md".to_string(),
                name: "notes.md".to_string(),
            },
        ],
        None,
    )
    .expect("code-mode ported");
    assert_eq!(
        projected.value,
        json!("guide.pdf: https://example.com/guide.pdf\nnotes.md: file:///tmp/notes.md")
    );
    assert!(projected.attachments.is_empty());
}

#[test]
#[ignore = "porting: code-mode not implemented"]
fn empty_object_input_is_omitted_from_call_metadata() {
    assert_eq!(metadata_input(&json!({})).expect("code-mode ported"), None);
    assert_eq!(
        metadata_input(&json!({ "name": "Ada" })).expect("code-mode ported"),
        Some(json!({ "name": "Ada" }))
    );
    assert_eq!(
        metadata_input(&json!([1, 2])).expect("code-mode ported"),
        Some(json!({ "input": [1, 2] }))
    );
}
