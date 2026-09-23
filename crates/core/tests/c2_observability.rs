//! Port of packages/core/test/effect/observability.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: the Effect `Logger.layer`/`NodeFileSystem` plumbing is dropped.
//! Pure projections (OTEL resource attribute parsing/decoding, built-in
//! precedence, log-line flattening) keep their observable contract.

#![allow(dead_code)]

mod observability {
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl std::fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "not implemented: {}", self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum LogValue {
        Str(String),
        Int(i64),
        List(Vec<String>),
        Object(Vec<(String, LogValue)>),
    }

    /// Percent-decode a single OTEL attribute component.
    pub fn decode_component(_input: &str) -> Result<String, NotImplemented> {
        Err(NotImplemented("otlp decodeComponent"))
    }

    /// Parse `OTEL_RESOURCE_ATTRIBUTES`; any entry without `=` invalidates the
    /// whole set.
    pub fn parse_resource_attributes(
        _input: &str,
    ) -> Result<Vec<(String, String)>, NotImplemented> {
        Err(NotImplemented("otlp parseResourceAttributes"))
    }

    /// Resolve the OTLP resource attributes with built-ins taking precedence.
    pub fn resource(
        otel_env: Option<&str>,
        opencode_client: Option<&str>,
    ) -> Result<BTreeMap<String, String>, NotImplemented> {
        let _ = (otel_env, opencode_client);
        Err(NotImplemented("otlp resource"))
    }

    /// Render a structured log line, flattening nested annotations to dotted
    /// keys and quoting message/array values.
    pub fn format_log_line(
        run: &str,
        level: &str,
        message: &str,
        annotations: &[(String, LogValue)],
    ) -> Result<String, NotImplemented> {
        let _ = (run, level, message, annotations);
        Err(NotImplemented("file logger"))
    }
}

const NOTE: &str = "porting: observability not implemented";

#[test]
#[ignore = "porting: observability not implemented"]
fn parses_and_decodes_otel_resource_attributes() {
    let attrs = observability::resource(
        Some(
            "service.namespace=anomalyco,team=platform%2Cobservability,label=hello%3Dworld,key%2Fname=value%20here",
        ),
        None,
    )
    .expect(NOTE);

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
#[ignore = "porting: observability not implemented"]
fn drops_otel_resource_attributes_when_any_entry_is_invalid() {
    let attrs =
        observability::resource(Some("service.namespace=anomalyco,broken"), None).expect(NOTE);

    assert!(!attrs.contains_key("service.namespace"));
    assert!(attrs.contains_key("opencode.client"));
}

#[test]
#[ignore = "porting: observability not implemented"]
fn keeps_built_in_attributes_when_env_values_conflict() {
    let attrs = observability::resource(
        Some("opencode.client=web,service.instance.id=override,service.namespace=anomalyco"),
        Some("cli"),
    )
    .expect(NOTE);

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

    let run = attrs.get("opencode.run").expect(NOTE);
    assert_eq!(run.len(), 8);
    assert!(run
        .chars()
        .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
}

#[test]
#[ignore = "porting: observability not implemented"]
fn file_logger_appends_concurrent_runs_with_a_run_on_every_line() {
    let run_a = observability::format_log_line("run-a", "INFO", "entry-0", &[]).expect(NOTE);
    let run_b = observability::format_log_line("run-b", "INFO", "entry-0", &[]).expect(NOTE);

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
#[ignore = "porting: observability not implemented"]
fn file_logger_flattens_nested_objects() {
    let annotations = vec![
        (
            "request".to_string(),
            observability::LogValue::Object(vec![
                (
                    "method".to_string(),
                    observability::LogValue::Str("GET".to_string()),
                ),
                (
                    "timing".to_string(),
                    observability::LogValue::Object(vec![(
                        "duration".to_string(),
                        observability::LogValue::Int(42),
                    )]),
                ),
            ]),
        ),
        (
            "tags".to_string(),
            observability::LogValue::List(vec!["api".to_string(), "test".to_string()]),
        ),
        (
            "session".to_string(),
            observability::LogValue::Object(vec![(
                "id".to_string(),
                observability::LogValue::Str("session-1".to_string()),
            )]),
        ),
    ];

    let line = observability::format_log_line("run-a", "INFO", "request complete", &annotations)
        .expect(NOTE);

    assert!(line.contains("message=\"request complete\""));
    assert!(line.contains("request.method=GET"));
    assert!(line.contains("request.timing.duration=42"));
    assert!(line.contains("tags=\"[\\\"api\\\",\\\"test\\\"]\""));
    assert!(line.contains("session.id=session-1"));
    assert!(!line.contains("request={"));
}
