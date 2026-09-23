//! Port of packages/core/test/effect/observability.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: the Effect `Logger.layer`/`NodeFileSystem` plumbing is dropped.
//! Pure projections (OTEL resource attribute parsing/decoding, built-in
//! precedence, log-line flattening) keep their observable contract, implemented
//! in `opencode_core::observability`.

use opencode_core::observability::{format_log_line, resource_map, LogValue};

#[test]
fn parses_and_decodes_otel_resource_attributes() {
    let attrs = resource_map(
        Some(
            "service.namespace=anomalyco,team=platform%2Cobservability,label=hello%3Dworld,key%2Fname=value%20here",
        ),
        None,
    );

    assert_eq!(
        attrs.get("service.namespace").map(String::as_str),
        Some("anomalyco")
    );
    assert_eq!(
        attrs.get("team").map(String::as_str),
        Some("platform,observability")
    );
    assert_eq!(attrs.get("label").map(String::as_str), Some("hello=world"));
    assert_eq!(
        attrs.get("key/name").map(String::as_str),
        Some("value here")
    );
}

#[test]
fn drops_otel_resource_attributes_when_any_entry_is_invalid() {
    let attrs = resource_map(Some("service.namespace=anomalyco,broken"), None);

    assert!(!attrs.contains_key("service.namespace"));
    assert!(attrs.contains_key("opencode.client"));
}

#[test]
fn keeps_built_in_attributes_when_env_values_conflict() {
    let attrs = resource_map(
        Some("opencode.client=web,service.instance.id=override,service.namespace=anomalyco"),
        Some("cli"),
    );

    assert_eq!(
        attrs.get("opencode.client").map(String::as_str),
        Some("cli")
    );
    assert_eq!(
        attrs.get("service.namespace").map(String::as_str),
        Some("anomalyco")
    );
    assert_ne!(
        attrs.get("service.instance.id").map(String::as_str),
        Some("override")
    );

    let run = attrs.get("opencode.run").expect("run");
    assert_eq!(run.len(), 8);
    assert!(run
        .chars()
        .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
}

#[test]
fn file_logger_appends_concurrent_runs_with_a_run_on_every_line() {
    let run_a = format_log_line("run-a", "INFO", "entry-0", &[]);
    let run_b = format_log_line("run-b", "INFO", "entry-0", &[]);

    assert!(run_a.starts_with("timestamp="));
    assert!(run_a.contains(" level=INFO "));
    assert!(run_a.contains("run=run-a"));
    assert!(!run_a.contains(" fiber="));
    assert!(!run_a.starts_with('{'));

    assert!(run_b.starts_with("timestamp="));
    assert!(run_b.contains(" level=INFO "));
    assert!(run_b.contains("run=run-b"));
    assert!(!run_b.contains(" fiber="));
    assert!(!run_b.starts_with('{'));
}

#[test]
fn file_logger_flattens_nested_objects() {
    let annotations = vec![
        (
            "request".to_string(),
            LogValue::Object(vec![
                ("method".to_string(), LogValue::Str("GET".to_string())),
                (
                    "timing".to_string(),
                    LogValue::Object(vec![("duration".to_string(), LogValue::Int(42))]),
                ),
            ]),
        ),
        (
            "tags".to_string(),
            LogValue::List(vec!["api".to_string(), "test".to_string()]),
        ),
        (
            "session".to_string(),
            LogValue::Object(vec![(
                "id".to_string(),
                LogValue::Str("session-1".to_string()),
            )]),
        ),
    ];

    let line = format_log_line("run-a", "INFO", "request complete", &annotations);

    assert!(line.contains("message=\"request complete\""));
    assert!(line.contains("request.method=GET"));
    assert!(line.contains("request.timing.duration=42"));
    assert!(line.contains("tags=\"[\\\"api\\\",\\\"test\\\"]\""));
    assert!(line.contains("session.id=session-1"));
    assert!(!line.contains("request={"));
}
