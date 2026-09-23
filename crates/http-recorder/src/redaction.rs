//! Sensitive-value redaction for recorded HTTP interactions.
//!
//! Derived from the observable behaviour pinned by
//! `packages/http-recorder/src/redaction.ts` and `redactor.ts` (upstream 18ef3cc).

use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

/// Placeholder substituted for every redacted value.
pub const REDACTED: &str = "[REDACTED]";

/// A caller-supplied transform applied to a URL after built-in redaction.
pub type UrlRedactor = dyn Fn(&str) -> String;

/// Headers redacted unless the caller narrows the set.
pub const DEFAULT_REDACT_HEADERS: &[&str] = &[
    "authorization",
    "cookie",
    "proxy-authorization",
    "set-cookie",
    "x-api-key",
    "x-amz-security-token",
    "x-goog-api-key",
];

/// Query parameters redacted unless the caller narrows the set.
pub const DEFAULT_REDACT_QUERY: &[&str] = &[
    "access_token",
    "api-key",
    "api_key",
    "apikey",
    "code",
    "key",
    "signature",
    "sig",
    "token",
    "x-amz-credential",
    "x-amz-security-token",
    "x-amz-signature",
];

/// Headers a default request redactor keeps when present.
pub const DEFAULT_REQUEST_HEADERS: &[&str] = &["content-type", "accept", "openai-beta"];

/// Headers a default response redactor keeps when present.
pub const DEFAULT_RESPONSE_HEADERS: &[&str] = &["content-type"];

const DEFAULT_REDACT_JSON_FIELDS: &[&str] = &[
    "access_token",
    "api_key",
    "apikey",
    "client_secret",
    "password",
    "refresh_token",
    "secret",
    "token",
];

const SECRET_PATTERNS: &[(&str, &str)] = &[
    ("bearer token", r"(?i)\bBearer\s+[A-Za-z0-9._~+/=-]{16,}\b"),
    ("API key", r"\bsk-[A-Za-z0-9][A-Za-z0-9_-]{20,}\b"),
    ("Anthropic API key", r"\bsk-ant-[A-Za-z0-9_-]{20,}\b"),
    ("Google API key", r"\bAIza[0-9A-Za-z_-]{20,}\b"),
    ("AWS access key", r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b"),
    ("GitHub token", r"\bgh[pousr]_[A-Za-z0-9_]{20,}\b"),
    ("private key", r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
];

fn compiled_secret_patterns() -> &'static Vec<(&'static str, Regex)> {
    static PATTERNS: OnceLock<Vec<(&'static str, Regex)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        SECRET_PATTERNS
            .iter()
            .filter_map(|(label, pattern)| Regex::new(pattern).ok().map(|regex| (*label, regex)))
            .collect()
    })
}

fn lower_set(values: &[&str], defaults: &[&str]) -> BTreeSet<String> {
    defaults
        .iter()
        .chain(values.iter())
        .map(|value| value.to_lowercase())
        .collect()
}

/// Redact credentials and sensitive query parameters from a URL.
pub fn redact_url(raw: &str, query: Option<&[&str]>, url_redactor: Option<&UrlRedactor>) -> String {
    let parsed = match url::Url::parse(raw) {
        Ok(parsed) => parsed,
        Err(_) => {
            return url_redactor
                .map(|f| f(raw))
                .unwrap_or_else(|| raw.to_string())
        }
    };
    let mut url = parsed;
    if !url.username().is_empty() {
        let _ = url.set_username(REDACTED);
    }
    if url.password().is_some() {
        let _ = url.set_password(Some(REDACTED));
    }

    let redacted = lower_set(query.unwrap_or(&[]), DEFAULT_REDACT_QUERY);
    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    if pairs
        .iter()
        .any(|(key, _)| redacted.contains(&key.to_lowercase()))
    {
        url.set_query(None);
        {
            let mut serializer = url.query_pairs_mut();
            for (key, value) in &pairs {
                if redacted.contains(&key.to_lowercase()) {
                    serializer.append_pair(key, REDACTED);
                } else {
                    serializer.append_pair(key, value);
                }
            }
        }
    }

    let result = url.to_string();
    url_redactor.map(|f| f(&result)).unwrap_or(result)
}

/// Filter headers to the allow-list and redact sensitive names.
pub fn redact_headers(
    headers: &[(&str, &str)],
    allow: &[&str],
    redact: &[&str],
) -> BTreeMap<String, String> {
    let allowed: BTreeSet<String> = allow.iter().map(|name| name.to_lowercase()).collect();
    let redacted = lower_set(redact, DEFAULT_REDACT_HEADERS);
    let mut out = BTreeMap::new();
    for (name, value) in headers {
        let lower = name.to_lowercase();
        if !allowed.contains(&lower) {
            continue;
        }
        let value = if redacted.contains(&lower) {
            REDACTED.to_string()
        } else {
            (*value).to_string()
        };
        out.insert(lower, value);
    }
    out
}

/// A single secret-shaped value found in a cassette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretFinding {
    pub path: String,
    pub reason: String,
}

/// An order-preserving JSON value used to scan cassettes for secrets.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

fn path_for(base: &str, key: &str) -> String {
    if base.is_empty() {
        key.to_string()
    } else {
        format!("{base}.{key}")
    }
}

fn collect_strings<'a>(value: &'a Json, base: &str, out: &mut Vec<(String, &'a str)>) {
    match value {
        Json::Str(text) => out.push((base.to_string(), text.as_str())),
        Json::Arr(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_strings(item, &format!("{base}[{index}]"), out);
            }
        }
        Json::Obj(fields) => {
            for (key, child) in fields {
                collect_strings(child, &path_for(base, key), out);
            }
        }
        Json::Null | Json::Bool(_) | Json::Num(_) => {}
    }
}

fn env_secrets() -> Vec<(String, String)> {
    static SAFE: &[&str] = &["fixture", "test", "test-key"];
    let name_pattern = Regex::new(r"(?i)(?:API|AUTH|BEARER|CREDENTIAL|KEY|PASSWORD|SECRET|TOKEN)")
        .expect("valid env-name pattern");
    std::env::vars()
        .filter(|(name, value)| {
            !value.is_empty()
                && name_pattern.is_match(name)
                && value.len() >= 12
                && !SAFE.contains(&value.to_lowercase().as_str())
        })
        .collect()
}

/// Scan a cassette for secret-looking values without returning the secrets.
pub fn secret_findings(value: &Json) -> Vec<SecretFinding> {
    let mut strings = Vec::new();
    collect_strings(value, "", &mut strings);
    let environment = env_secrets();
    let mut findings = Vec::new();
    for (path, text) in strings {
        for (label, pattern) in compiled_secret_patterns() {
            if pattern.is_match(text) {
                findings.push(SecretFinding {
                    path: path.clone(),
                    reason: (*label).to_string(),
                });
            }
        }
        for (name, secret) in &environment {
            if text.contains(secret) {
                findings.push(SecretFinding {
                    path: path.clone(),
                    reason: format!("environment secret {name}"),
                });
            }
        }
    }
    findings
}

/// A request snapshot before or after redaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestSnapshot {
    pub method: String,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body: String,
}

/// Build an error-request snapshot that retains no headers, query, or body.
pub fn redacted_error_request(method: &str, url: &str) -> RequestSnapshot {
    let stripped = url::Url::parse(url)
        .map(|mut parsed| {
            parsed.set_query(None);
            parsed.to_string()
        })
        .unwrap_or_else(|_| url.to_string());
    RequestSnapshot {
        method: method.to_string(),
        url: stripped,
        headers: BTreeMap::new(),
        body: String::new(),
    }
}

/// A response snapshot before or after redaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseSnapshot {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: String,
}

/// Overrides applied by [`Redactor::make`].
#[derive(Default)]
pub struct RedactorOptions {
    pub headers: Vec<String>,
    pub allow_request_headers: Vec<String>,
    pub allow_response_headers: Vec<String>,
    pub query_parameters: Vec<String>,
    pub json_fields: Vec<String>,
    pub url: Option<Box<UrlRedactor>>,
}

/// A composable request/response redactor.
pub struct Redactor {
    headers: Vec<String>,
    allow_request_headers: Vec<String>,
    allow_response_headers: Vec<String>,
    query_parameters: Vec<String>,
    json_fields: BTreeSet<String>,
    url: Option<Box<UrlRedactor>>,
}

fn normalize_field(field: &str) -> String {
    field
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

fn redact_json_fields(value: &serde_json::Value, fields: &BTreeSet<String>) -> serde_json::Value {
    match value {
        serde_json::Value::Array(items) => serde_json::Value::Array(
            items
                .iter()
                .map(|item| redact_json_fields(item, fields))
                .collect(),
        ),
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.iter()
                .map(|(key, child)| {
                    let value = if fields.contains(&normalize_field(key)) {
                        serde_json::Value::String(REDACTED.to_string())
                    } else {
                        redact_json_fields(child, fields)
                    };
                    (key.clone(), value)
                })
                .collect(),
        ),
        other => other.clone(),
    }
}

fn redact_body(body: &str, fields: &BTreeSet<String>) -> String {
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(parsed) => serde_json::to_string(&redact_json_fields(&parsed, fields))
            .unwrap_or_else(|_| body.to_string()),
        Err(_) => body.to_string(),
    }
}

impl Redactor {
    pub fn make(options: RedactorOptions) -> Self {
        let mut json_fields: BTreeSet<String> = DEFAULT_REDACT_JSON_FIELDS
            .iter()
            .map(|field| normalize_field(field))
            .collect();
        json_fields.extend(
            options
                .json_fields
                .iter()
                .map(|field| normalize_field(field)),
        );
        Self {
            headers: options.headers,
            allow_request_headers: options.allow_request_headers,
            allow_response_headers: options.allow_response_headers,
            query_parameters: options.query_parameters,
            json_fields,
            url: options.url,
        }
    }

    pub fn request(&self, snapshot: &RequestSnapshot) -> RequestSnapshot {
        let mut allow: Vec<&str> = DEFAULT_REQUEST_HEADERS.to_vec();
        allow.extend(self.allow_request_headers.iter().map(String::as_str));
        allow.extend(self.headers.iter().map(String::as_str));
        let redact: Vec<&str> = self.headers.iter().map(String::as_str).collect();
        let headers: Vec<(&str, &str)> = snapshot
            .headers
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        let headers = redact_headers(&headers, &allow, &redact);

        let mut query: Vec<&str> = DEFAULT_REDACT_QUERY.to_vec();
        query.extend(self.query_parameters.iter().map(String::as_str));
        let url = redact_url(&snapshot.url, Some(query.as_slice()), self.url.as_deref());

        RequestSnapshot {
            method: snapshot.method.clone(),
            url,
            headers,
            body: redact_body(&snapshot.body, &self.json_fields),
        }
    }

    pub fn response(&self, snapshot: &ResponseSnapshot) -> ResponseSnapshot {
        let mut allow: Vec<&str> = DEFAULT_RESPONSE_HEADERS.to_vec();
        allow.extend(self.allow_response_headers.iter().map(String::as_str));
        allow.extend(self.headers.iter().map(String::as_str));
        let redact: Vec<&str> = self.headers.iter().map(String::as_str).collect();
        let headers: Vec<(&str, &str)> = snapshot
            .headers
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        let headers = redact_headers(&headers, &allow, &redact);
        ResponseSnapshot {
            status: snapshot.status,
            headers,
            body: redact_body(&snapshot.body, &self.json_fields),
        }
    }
}
