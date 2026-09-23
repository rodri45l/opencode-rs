//! Port of packages/http-recorder/test/record-replay.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/http-recorder/src/{redaction,redactor,cassette}.ts.
//! The pure redaction/cassette-listing subset is ported green; the Effect-based
//! record/replay and WebSocket suites are re-derived `#[ignore]` pending the
//! recorder runtime (see docs/TEST-PORT.md).

use opencode_http_recorder::{
    has_cassette_sync, list_cassettes, redact_headers, redact_url, redacted_error_request,
    secret_findings, Json, Redactor, RedactorOptions, RequestSnapshot,
};
use std::collections::BTreeMap;

fn obj(fields: Vec<(&str, Json)>) -> Json {
    Json::Obj(
        fields
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

fn s(value: &str) -> Json {
    Json::Str(value.to_string())
}

fn n(value: f64) -> Json {
    Json::Num(value)
}

fn arr(items: Vec<Json>) -> Json {
    Json::Arr(items)
}

fn headers(entries: &[(&'static str, &'static str)]) -> Vec<(&'static str, &'static str)> {
    entries.to_vec()
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("http-recorder-{label}-{unique}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn redacts_sensitive_url_query_parameters() {
    assert_eq!(
        redact_url(
            "https://example.test/path?key=secret-google-key&api_key=secret-openai-key&safe=value&X-Amz-Signature=secret-signature",
            None,
            None,
        ),
        "https://example.test/path?key=%5BREDACTED%5D&api_key=%5BREDACTED%5D&safe=value&X-Amz-Signature=%5BREDACTED%5D"
    );
}

#[test]
fn redacts_url_credentials() {
    assert_eq!(
        redact_url(
            "https://user:password@example.test/path?safe=value",
            None,
            None
        ),
        "https://%5BREDACTED%5D:%5BREDACTED%5D@example.test/path?safe=value"
    );
}

#[test]
fn applies_custom_url_redaction_after_built_in_redaction() {
    let custom = |url: &str| url.replace("/accounts/real-account/", "/accounts/{account}/");
    let redactor: &dyn Fn(&str) -> String = &custom;
    assert_eq!(
        redact_url(
            "https://example.test/accounts/real-account/path?key=secret-key",
            None,
            Some(redactor),
        ),
        "https://example.test/accounts/{account}/path?key=%5BREDACTED%5D"
    );
}

#[test]
fn redacts_sensitive_headers_when_allow_listed() {
    let result = redact_headers(
        &headers(&[
            ("authorization", "Bearer secret-token"),
            ("content-type", "application/json"),
            ("x-custom-token", "custom-secret"),
            ("x-api-key", "secret-key"),
            ("x-goog-api-key", "secret-google-key"),
        ]),
        &[
            "authorization",
            "content-type",
            "x-api-key",
            "x-goog-api-key",
            "x-custom-token",
        ],
        &["x-custom-token"],
    );

    let mut expected = BTreeMap::new();
    expected.insert("authorization".to_string(), "[REDACTED]".to_string());
    expected.insert("content-type".to_string(), "application/json".to_string());
    expected.insert("x-api-key".to_string(), "[REDACTED]".to_string());
    expected.insert("x-custom-token".to_string(), "[REDACTED]".to_string());
    expected.insert("x-goog-api-key".to_string(), "[REDACTED]".to_string());
    assert_eq!(result, expected);
}

#[test]
fn redacts_error_requests_without_retaining_headers_params_or_body() {
    let request =
        redacted_error_request("POST", "https://example.test/path?api_key=super-secret-key");
    assert_eq!(request.url, "https://example.test/path");
    assert!(request.headers.is_empty());
    assert!(request.body.is_empty());
}

#[test]
fn detects_secret_looking_values_without_returning_the_secret() {
    let value = obj(vec![
        ("version", n(1.0)),
        (
            "interactions",
            arr(vec![obj(vec![
                ("transport", s("http")),
                (
                    "request",
                    obj(vec![
                        ("method", s("POST")),
                        (
                            "url",
                            s("https://example.test/path?key=sk-123456789012345678901234"),
                        ),
                        ("headers", obj(vec![])),
                        (
                            "body",
                            s(r#"{"nested":"AIzaSyDHibiBRvJZLsFnPYPoiTwxY4ztQ55yqCE"}"#),
                        ),
                    ]),
                ),
                (
                    "response",
                    obj(vec![
                        ("status", n(200.0)),
                        ("headers", obj(vec![])),
                        ("body", s("Bearer abcdefghijklmnopqrstuvwxyz")),
                    ]),
                ),
            ])]),
        ),
    ]);

    let findings = secret_findings(&value);
    let pairs: Vec<(&str, &str)> = findings
        .iter()
        .map(|finding| (finding.path.as_str(), finding.reason.as_str()))
        .collect();
    assert_eq!(
        pairs,
        vec![
            ("interactions[0].request.url", "API key"),
            ("interactions[0].request.body", "Google API key"),
            ("interactions[0].response.body", "bearer token"),
        ]
    );
}

#[test]
fn detects_secret_looking_values_inside_metadata() {
    let value = obj(vec![
        ("version", n(1.0)),
        (
            "metadata",
            obj(vec![("token", s("sk-123456789012345678901234"))]),
        ),
        ("interactions", arr(vec![])),
    ]);
    let findings = secret_findings(&value);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].path, "metadata.token");
    assert_eq!(findings[0].reason, "API key");
}

#[test]
fn redacts_configured_and_common_sensitive_json_fields() {
    let redactor = Redactor::make(RedactorOptions {
        json_fields: vec!["account_id".to_string()],
        ..Default::default()
    });
    let mut request_headers = BTreeMap::new();
    request_headers.insert("content-type".to_string(), "application/json".to_string());
    let request = redactor.request(&RequestSnapshot {
        method: "POST".to_string(),
        url: "https://example.test/path".to_string(),
        headers: request_headers,
        body: r#"{"password":"secret-password","accessToken":"access-token","nested":{"account_id":"account-123","safe":"visible"}}"#
            .to_string(),
    });

    let parsed: serde_json::Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(
        parsed,
        serde_json::json!({
            "password": "[REDACTED]",
            "accessToken": "[REDACTED]",
            "nested": { "account_id": "[REDACTED]", "safe": "visible" },
        })
    );
}

#[test]
fn extends_default_header_redaction_and_allow_lists() {
    let redactor = Redactor::make(RedactorOptions {
        headers: vec!["x-custom-token".to_string()],
        allow_request_headers: vec![
            "anthropic-version".to_string(),
            "x-custom-token".to_string(),
        ],
        ..Default::default()
    });
    let mut request_headers = BTreeMap::new();
    request_headers.insert("authorization".to_string(), "Bearer secret".to_string());
    request_headers.insert("content-type".to_string(), "application/json".to_string());
    request_headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
    request_headers.insert("x-custom-token".to_string(), "secret".to_string());

    let request = redactor.request(&RequestSnapshot {
        method: "GET".to_string(),
        url: "https://example.test/path".to_string(),
        headers: request_headers,
        body: String::new(),
    });

    let mut expected = BTreeMap::new();
    expected.insert("anthropic-version".to_string(), "2023-06-01".to_string());
    expected.insert("content-type".to_string(), "application/json".to_string());
    expected.insert("x-custom-token".to_string(), "[REDACTED]".to_string());
    assert_eq!(request.headers, expected);
}

#[test]
fn rejects_cassette_paths_outside_the_recordings_directory() {
    let directory = temp_dir("path");
    assert_eq!(
        has_cassette_sync("../outside", &directory),
        Err(opencode_http_recorder::CassetteError::InvalidName)
    );
    assert_eq!(
        has_cassette_sync("C:\\outside", &directory),
        Err(opencode_http_recorder::CassetteError::InvalidName)
    );
    std::fs::remove_dir_all(&directory).ok();
}

#[test]
fn cassette_list_enumerates_recorded_cassette_names() {
    let directory = temp_dir("list");
    std::fs::create_dir_all(directory.join("alpha")).unwrap();
    std::fs::write(directory.join("alpha/one.json"), "{}").unwrap();
    std::fs::write(directory.join("beta.json"), "{}").unwrap();

    assert_eq!(
        list_cassettes(&directory),
        vec!["alpha/one".to_string(), "beta".to_string()]
    );
    std::fs::remove_dir_all(&directory).ok();
}
