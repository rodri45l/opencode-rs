//! Port of packages/cli/src/commands/handlers/api.test.ts (upstream 18ef3cc).
//!
//! The reference test is colocated with the handler under `src/` rather than in
//! a `test/` directory; the path above is the pinned reference location.

use std::collections::BTreeMap;

use opencode_cli::handlers::api::{raw_request, resolve_operation, ApiError, OpenApi, Operation};

fn session_get_spec() -> OpenApi {
    let mut operations = BTreeMap::new();
    operations.insert(
        "get".to_string(),
        Operation {
            operation_id: Some("v2.session.get".to_string()),
        },
    );
    let mut paths = BTreeMap::new();
    paths.insert("/api/session/{sessionID}".to_string(), operations);
    OpenApi { paths }
}

#[test]
fn resolves_an_operation_id_with_path_and_query_parameters() {
    let resolved = resolve_operation(
        &session_get_spec(),
        "v2.session.get",
        &[("sessionID", "ses/a"), ("workspace", "work")],
    )
    .expect("resolve");

    assert_eq!(resolved.method, "GET");
    assert_eq!(resolved.path, "/api/session/ses%2Fa?workspace=work");
}

#[test]
fn rejects_a_missing_path_parameter() {
    let error = resolve_operation(&session_get_spec(), "v2.session.get", &[]).unwrap_err();

    assert_eq!(
        error,
        ApiError::MissingPathParameter("sessionID".to_string())
    );
    assert_eq!(error.to_string(), "Missing path parameter: sessionID");
}

#[test]
fn resolves_curl_like_method_and_path_input() {
    assert_eq!(
        raw_request(&["post".to_string(), "/api/foo".to_string()]),
        Some(opencode_cli::handlers::api::ResolvedRequest {
            method: "POST".to_string(),
            path: "/api/foo".to_string(),
        })
    );
    assert_eq!(raw_request(&["v2.session.list".to_string()]), None);
}
