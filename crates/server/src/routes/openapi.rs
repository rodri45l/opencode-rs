//! Served OpenAPI document (`GET /doc`).

use serde_json::{json, Map, Value};

fn ref_schema(name: &str) -> Value {
    json!({ "$ref": format!("#/components/schemas/{name}") })
}

fn error_response(name: &str) -> Value {
    json!({
        "description": name,
        "content": { "application/json": { "schema": ref_schema(name) } },
    })
}

/// A `200`-with-JSON operation, plus the shared `/api` auth responses.
fn api_op() -> Value {
    json!({
        "security": [],
        "responses": {
            "200": { "description": "OK" },
            "401": { "description": "Unauthorized" },
        },
    })
}

fn plain_op() -> Value {
    json!({ "responses": { "200": { "description": "OK" } } })
}

fn op_with_not_found(name: &str) -> Value {
    json!({
        "responses": {
            "200": { "description": "OK" },
            "404": error_response(name),
        },
    })
}

/// Build the OpenAPI document served at `/doc`.
pub fn document() -> Value {
    let mut schemas = Map::new();
    for name in [
        "CredentialValue",
        "IntegrationInputs",
        "IntegrationMethod",
        "IntegrationRef",
        "SkillV2Source",
        "PermissionNotFoundError",
        "QuestionNotFoundError",
        "SessionNotFoundError",
        "McpServerNotFoundError",
        "PtyNotFoundError",
        "PtyForbiddenError",
        "ProjectNotFoundError",
    ] {
        schemas.insert(name.to_string(), json!({ "type": "object" }));
    }
    schemas.insert(
        "SyncEventSessionCreated".to_string(),
        json!({
            "type": "object",
            "required": ["type", "id", "syncEvent"],
            "properties": {
                "type": { "enum": ["sync"] },
                "id": { "type": "string" },
                "syncEvent": {
                    "type": "object",
                    "required": ["type", "id", "seq", "aggregateID", "data"],
                    "properties": {
                        "type": { "enum": ["session.created.1"] },
                        "id": { "type": "string" },
                        "seq": { "type": "number" },
                        "aggregateID": { "type": "string" },
                        "data": { "type": "object" },
                    },
                },
            },
        }),
    );
    schemas.insert(
        "V2Event".to_string(),
        json!({ "anyOf": [ref_schema("SyncEventSessionCreated")] }),
    );
    schemas.insert("V2EventStream".to_string(), json!({ "type": "string" }));

    let mut paths = Map::new();
    paths.insert("/global/health".into(), json!({ "get": plain_op() }));
    paths.insert("/session".into(), json!({ "get": plain_op() }));
    paths.insert(
        "/api/event".into(),
        json!({
            "get": {
                "security": [],
                "responses": {
                    "200": {
                        "description": "OK",
                        "content": {
                            "text/event-stream": { "schema": ref_schema("V2Event") }
                        }
                    },
                    "401": { "description": "Unauthorized" },
                },
            }
        }),
    );
    paths.insert("/api/reference".into(), json!({ "get": api_op() }));
    paths.insert(
        "/api/fs/read/*".into(),
        json!({ "get": {
            "security": [],
            "parameters": [
                { "name": "directory", "in": "query" },
                { "name": "workspace", "in": "query" },
            ],
            "responses": {
                "200": { "description": "OK" },
                "401": { "description": "Unauthorized" },
            },
        } }),
    );
    paths.insert(
        "/api/fs/list".into(),
        json!({ "get": {
            "security": [],
            "parameters": [
                { "name": "directory", "in": "query" },
                { "name": "workspace", "in": "query" },
            ],
            "responses": {
                "200": { "description": "OK" },
                "401": { "description": "Unauthorized" },
            },
        } }),
    );
    for path in [
        "/api/session/{sessionID}/prompt",
        "/api/session/{sessionID}/permission/{requestID}/reply",
        "/api/session/{sessionID}/question/{requestID}/reply",
        "/api/session/{sessionID}/question/{requestID}/reject",
    ] {
        let mut op = api_op();
        op["requestBody"] = json!({ "required": true });
        if path.contains("question") {
            op["responses"]["404"] = json!({
                "description": "Not found",
                "content": {
                    "application/json": {
                        "schema": {
                            "anyOf": [
                                ref_schema("QuestionNotFoundError"),
                                ref_schema("SessionNotFoundError"),
                            ]
                        }
                    }
                },
            });
        }
        paths.insert(path.into(), json!({ "post": op }));
    }

    paths.insert(
        "/permission/{requestID}/reply".into(),
        json!({ "post": op_with_not_found("PermissionNotFoundError") }),
    );
    for path in [
        "/question/{requestID}/reply",
        "/question/{requestID}/reject",
    ] {
        paths.insert(
            path.into(),
            json!({ "post": op_with_not_found("QuestionNotFoundError") }),
        );
    }
    for path in [
        "/mcp/{name}/auth",
        "/mcp/{name}/auth/authenticate",
        "/mcp/{name}/auth/callback",
        "/mcp/{name}/connect",
        "/mcp/{name}/disconnect",
    ] {
        paths.insert(
            path.into(),
            json!({ "post": op_with_not_found("McpServerNotFoundError") }),
        );
    }
    paths["/mcp/{name}/auth"]["delete"] = op_with_not_found("McpServerNotFoundError");

    let mut pty_item = Map::new();
    for method in ["get", "put", "delete"] {
        pty_item.insert(method.to_string(), op_with_not_found("PtyNotFoundError"));
    }
    paths.insert("/pty/{ptyID}".into(), Value::Object(pty_item));
    paths.insert(
        "/pty/{ptyID}/connect-token".into(),
        json!({ "post": {
            "responses": {
                "200": { "description": "OK" },
                "403": error_response("PtyForbiddenError"),
                "404": error_response("PtyNotFoundError"),
            },
        } }),
    );
    paths.insert(
        "/pty/{ptyID}/connect".into(),
        json!({ "get": {
            "parameters": [
                { "name": "directory", "in": "query" },
                { "name": "workspace", "in": "query" },
                { "name": "cursor", "in": "query" },
                { "name": "ticket", "in": "query" },
            ],
            "responses": { "200": { "description": "OK" } },
        } }),
    );
    paths.insert(
        "/project/{projectID}".into(),
        json!({ "patch": op_with_not_found("ProjectNotFoundError") }),
    );

    json!({
        "openapi": "3.1.0",
        "info": { "title": "opencode", "version": "1.0.0" },
        "paths": Value::Object(paths),
        "components": { "schemas": Value::Object(schemas) },
    })
}
