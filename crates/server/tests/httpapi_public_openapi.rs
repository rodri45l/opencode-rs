//! Port of packages/opencode/test/server/httpapi-public-openapi.test.ts (upstream 18ef3cc).
//!
//! Re-derived: the reference builds the OpenAPI document directly from the
//! `PublicApi` value; the Rust port fetches the served `/doc` document and
//! asserts the same contract surface (component names, v2 error refs, route
//! parameters, and required request bodies).

mod common;

use axum::body::Body;
use axum::http::StatusCode;
use common::{json, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-httpapi-public-openapi-port";

fn component_ref(schema: &serde_json::Value) -> Option<String> {
    schema["$ref"]
        .as_str()
        .map(|value| value.replace("#/components/schemas/", ""))
}

fn component_names(response: &serde_json::Value) -> Vec<String> {
    let schema = &response["content"]["application/json"]["schema"];
    let mut names: Vec<String> = Vec::new();
    if let Some(name) = component_ref(schema) {
        names.push(name);
    }
    if let Some(items) = schema["anyOf"].as_array() {
        for item in items {
            if let Some(name) = component_ref(item) {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }
    }
    names
}

async fn spec() -> serde_json::Value {
    let app = router(AppState::new());
    let req = common::header(
        request("GET", "/doc").body(Body::empty()).unwrap(),
        "x-opencode-directory",
        DIRECTORY,
    );
    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::OK);
    json(res).await
}

#[tokio::test]
#[ignore = "porting: OpenAPI document route not implemented"]
async fn includes_plugin_facing_core_schemas() {
    let spec = spec().await;
    let schemas = spec["components"]["schemas"]
        .as_object()
        .expect("component schemas");
    for name in [
        "CredentialValue",
        "IntegrationInputs",
        "IntegrationMethod",
        "IntegrationRef",
        "SkillV2Source",
    ] {
        assert!(schemas.contains_key(name), "missing schema {name}");
    }
}

#[tokio::test]
#[ignore = "porting: OpenAPI document route not implemented"]
async fn documents_nested_legacy_global_sync_events() {
    let spec = spec().await;
    let schema = &spec["components"]["schemas"]["SyncEventSessionCreated"];

    assert_eq!(
        schema["required"],
        serde_json::json!(["type", "id", "syncEvent"])
    );
    assert_eq!(
        schema["properties"]["type"]["enum"],
        serde_json::json!(["sync"])
    );
    assert_eq!(
        schema["properties"]["syncEvent"]["required"],
        serde_json::json!(["type", "id", "seq", "aggregateID", "data"])
    );
    assert_eq!(
        schema["properties"]["syncEvent"]["properties"]["type"]["enum"],
        serde_json::json!(["session.created.1"])
    );
    assert_eq!(
        schema["properties"]["syncEvent"]["properties"]["aggregateID"]["type"],
        "string"
    );
}

#[tokio::test]
#[ignore = "porting: OpenAPI document route not implemented"]
async fn names_the_v2_event_union_without_the_sse_string_wrapper_collision() {
    let spec = spec().await;
    assert!(spec["components"]["schemas"]["V2Event1"].is_null());
    assert!(!spec["components"]["schemas"]["V2Event"]["anyOf"]
        .as_array()
        .expect("anyOf")
        .is_empty());
    assert_eq!(
        spec["components"]["schemas"]["V2EventStream"]["type"],
        "string"
    );
    assert_eq!(
        spec["paths"]["/api/event"]["get"]["responses"]["200"]["content"]["text/event-stream"]
            ["schema"],
        serde_json::json!({ "$ref": "#/components/schemas/V2Event" })
    );
}

#[tokio::test]
#[ignore = "porting: OpenAPI document route not implemented"]
async fn preserves_api_auth_responses() {
    let spec = spec().await;
    let paths = spec["paths"].as_object().expect("paths");
    for (path, item) in paths {
        if !path.starts_with("/api/") {
            continue;
        }
        for method in ["get", "post", "put", "delete", "patch"] {
            let Some(operation) = item.get(method) else {
                continue;
            };
            assert!(
                operation["responses"]["401"].is_object(),
                "{method} {path} missing 401"
            );
            assert_eq!(operation["security"], serde_json::json!([]));
        }
    }
}

#[tokio::test]
#[ignore = "porting: OpenAPI document route not implemented"]
async fn documents_references_separately_from_filesystem_routes() {
    let spec = spec().await;
    for path in ["/api/fs/read/*", "/api/fs/list"] {
        let params = spec["paths"][path]["get"]["parameters"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert!(
            !params.iter().any(|param| param["name"] == "reference"),
            "{path} must not expose the reference param"
        );
    }
    assert!(spec["paths"]["/api/reference"]["get"].is_object());
}

#[tokio::test]
#[ignore = "porting: OpenAPI document route not implemented"]
async fn preserves_required_request_bodies_for_v2_mutations() {
    let spec = spec().await;
    for path in [
        "/api/session/{sessionID}/prompt",
        "/api/session/{sessionID}/permission/{requestID}/reply",
        "/api/session/{sessionID}/question/{requestID}/reply",
    ] {
        assert_eq!(
            spec["paths"][path]["post"]["requestBody"]["required"], true,
            "{path}"
        );
    }
}

#[tokio::test]
#[ignore = "porting: OpenAPI document route not implemented"]
async fn documents_permission_and_question_not_found_errors() {
    let spec = spec().await;
    assert_eq!(
        component_ref(
            &spec["paths"]["/permission/{requestID}/reply"]["post"]["responses"]["404"]["content"]
                ["application/json"]["schema"]
        )
        .as_deref(),
        Some("PermissionNotFoundError")
    );
    for path in [
        "/question/{requestID}/reply",
        "/question/{requestID}/reject",
    ] {
        assert_eq!(
            component_ref(
                &spec["paths"][path]["post"]["responses"]["404"]["content"]["application/json"]
                    ["schema"]
            )
            .as_deref(),
            Some("QuestionNotFoundError")
        );
    }
    for path in [
        "/api/session/{sessionID}/question/{requestID}/reply",
        "/api/session/{sessionID}/question/{requestID}/reject",
    ] {
        assert_eq!(
            component_names(&spec["paths"][path]["post"]["responses"]["404"]),
            vec![
                "QuestionNotFoundError".to_string(),
                "SessionNotFoundError".to_string()
            ]
        );
    }
}

#[tokio::test]
#[ignore = "porting: OpenAPI document route not implemented"]
async fn documents_mcp_server_not_found_errors() {
    let spec = spec().await;
    for path in [
        "/mcp/{name}/auth",
        "/mcp/{name}/auth/authenticate",
        "/mcp/{name}/auth/callback",
        "/mcp/{name}/connect",
        "/mcp/{name}/disconnect",
    ] {
        assert_eq!(
            component_ref(
                &spec["paths"][path]["post"]["responses"]["404"]["content"]["application/json"]
                    ["schema"]
            )
            .as_deref(),
            Some("McpServerNotFoundError"),
            "{path}"
        );
    }
    assert_eq!(
        component_ref(
            &spec["paths"]["/mcp/{name}/auth"]["delete"]["responses"]["404"]["content"]
                ["application/json"]["schema"]
        )
        .as_deref(),
        Some("McpServerNotFoundError")
    );
}

#[tokio::test]
#[ignore = "porting: OpenAPI document route not implemented"]
async fn documents_pty_resource_and_ticket_errors() {
    let spec = spec().await;
    for (method, path) in [
        ("get", "/pty/{ptyID}"),
        ("put", "/pty/{ptyID}"),
        ("delete", "/pty/{ptyID}"),
        ("post", "/pty/{ptyID}/connect-token"),
    ] {
        assert_eq!(
            component_ref(
                &spec["paths"][path][method]["responses"]["404"]["content"]["application/json"]
                    ["schema"]
            )
            .as_deref(),
            Some("PtyNotFoundError"),
            "{method} {path}"
        );
    }
    assert_eq!(
        component_ref(
            &spec["paths"]["/pty/{ptyID}/connect-token"]["post"]["responses"]["403"]["content"]
                ["application/json"]["schema"]
        )
        .as_deref(),
        Some("PtyForbiddenError")
    );
    let query_params: Vec<&str> = spec["paths"]["/pty/{ptyID}/connect"]["get"]["parameters"]
        .as_array()
        .expect("parameters")
        .iter()
        .filter(|param| param["in"] == "query")
        .filter_map(|param| param["name"].as_str())
        .collect();
    assert_eq!(
        query_params,
        vec!["directory", "workspace", "cursor", "ticket"]
    );
}

#[tokio::test]
#[ignore = "porting: OpenAPI document route not implemented"]
async fn documents_project_not_found_errors() {
    let spec = spec().await;
    assert_eq!(
        component_ref(
            &spec["paths"]["/project/{projectID}"]["patch"]["responses"]["404"]["content"]
                ["application/json"]["schema"]
        )
        .as_deref(),
        Some("ProjectNotFoundError")
    );
}
