//! Observability resource attributes (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/observability/otlp.ts`
//! and `observability/logging.ts`: `OTEL_RESOURCE_ATTRIBUTES` is parsed as
//! comma-separated `key=value` pairs with percent-decoding, and if any entry is
//! invalid the whole user-supplied attribute set is dropped (built-ins remain).
//! Built-in attributes (`opencode.client`, `service.instance.id`,
//! `opencode.run`) win over conflicting environment values. The Effect logger
//! layer and file logger are dropped.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{CoreError, CoreResult};

/// Environment inputs for resource resolution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObservabilityEnv {
    /// `OTEL_RESOURCE_ATTRIBUTES`.
    pub otel_resource_attributes: Option<String>,
    /// `OPENCODE_CLIENT`.
    pub opencode_client: Option<String>,
}

/// A resolved OTLP resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    /// The resolved attributes.
    pub attributes: BTreeMap<String, String>,
}

/// Parse and decode `OTEL_RESOURCE_ATTRIBUTES` into key/value pairs.
pub fn parse_otel_attributes(raw: &str) -> CoreResult<BTreeMap<String, String>> {
    let mut attributes = BTreeMap::new();
    for entry in raw.split(',') {
        let index = entry.find('=');
        match index {
            Some(index) if index >= 1 => {
                let key = percent_decode(&entry[..index]);
                let value = percent_decode(&entry[index + 1..]);
                attributes.insert(key, value);
            }
            _ => {
                return Err(CoreError::Invalid(format!(
                    "invalid OTEL_RESOURCE_ATTRIBUTES entry: {entry}"
                )))
            }
        }
    }
    Ok(attributes)
}

/// Resolve the OTLP resource attributes for the given environment.
pub fn resource(env: &ObservabilityEnv) -> CoreResult<Resource> {
    let mut attributes = match env.otel_resource_attributes.as_deref() {
        Some(raw) => parse_otel_attributes(raw).unwrap_or_default(),
        None => BTreeMap::new(),
    };
    let run = run_id();
    attributes.insert(
        "opencode.client".to_string(),
        env.opencode_client
            .clone()
            .unwrap_or_else(|| "cli".to_string()),
    );
    attributes.insert("opencode.run".to_string(), run.clone());
    attributes.insert("service.instance.id".to_string(), run);
    Ok(Resource { attributes })
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = &value[index + 1..index + 3];
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                decoded.push(byte);
                index += 3;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

static RUN_COUNTER: AtomicU64 = AtomicU64::new(0);

fn run_id() -> String {
    let counter = RUN_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);
    let mixed = nanos ^ (counter.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    format!("{:08x}", (mixed ^ (mixed >> 32)) as u32)
}

/// A structured log annotation value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogValue {
    /// A string value.
    Str(String),
    /// An integer value.
    Int(i64),
    /// A list of strings.
    List(Vec<String>),
    /// A nested object.
    Object(Vec<(String, LogValue)>),
}

/// Percent-decode a single OTEL attribute component.
pub fn decode_component(input: &str) -> String {
    percent_decode(input)
}

/// Parse `OTEL_RESOURCE_ATTRIBUTES` into ordered key/value pairs; any entry
/// without `=` invalidates the whole set.
pub fn parse_resource_attributes(input: &str) -> CoreResult<Vec<(String, String)>> {
    let mut attributes = Vec::new();
    for entry in input.split(',') {
        match entry.find('=') {
            Some(index) if index >= 1 => {
                attributes.push((
                    percent_decode(&entry[..index]),
                    percent_decode(&entry[index + 1..]),
                ));
            }
            _ => {
                return Err(CoreError::Invalid(format!(
                    "invalid OTEL_RESOURCE_ATTRIBUTES entry: {entry}"
                )))
            }
        }
    }
    Ok(attributes)
}

/// Resolve the OTLP resource attributes with built-ins taking precedence.
pub fn resource_map(
    otel_env: Option<&str>,
    opencode_client: Option<&str>,
) -> BTreeMap<String, String> {
    let mut attributes: BTreeMap<String, String> = otel_env
        .and_then(|raw| parse_resource_attributes(raw).ok())
        .map(|pairs| pairs.into_iter().collect())
        .unwrap_or_default();
    let run = run_id();
    attributes.insert(
        "opencode.client".to_string(),
        opencode_client.unwrap_or("cli").to_string(),
    );
    attributes.insert("opencode.run".to_string(), run.clone());
    attributes.insert("service.instance.id".to_string(), run);
    attributes
}

/// Render a structured log line, flattening nested annotations to dotted keys
/// and quoting message/array values.
pub fn format_log_line(
    run: &str,
    level: &str,
    message: &str,
    annotations: &[(String, LogValue)],
) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let mut fields = Vec::new();
    flatten_annotations("", annotations, &mut fields);
    format!(
        "timestamp={timestamp} level={level} run={run} message={} {}",
        render_value(&LogValue::Str(message.to_string())),
        fields.join(" ")
    )
    .trim_end()
    .to_string()
}

fn flatten_annotations(prefix: &str, annotations: &[(String, LogValue)], out: &mut Vec<String>) {
    for (key, value) in annotations {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        match value {
            LogValue::Object(entries) => flatten_annotations(&path, entries, out),
            other => out.push(format!("{path}={}", render_value(other))),
        }
    }
}

fn render_value(value: &LogValue) -> String {
    match value {
        LogValue::Int(number) => number.to_string(),
        LogValue::Str(text) => quote_if_needed(text),
        LogValue::List(items) => {
            let json = serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string());
            quote_if_needed(&json)
        }
        LogValue::Object(_) => "{}".to_string(),
    }
}

fn quote_if_needed(text: &str) -> String {
    if text.is_empty() || text.chars().any(|c| c.is_whitespace() || c == '"') {
        format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        text.to_string()
    }
}
