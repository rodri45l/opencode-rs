//! Shared test helpers for the port.
//!
//! Writer agents must use these instead of re-implementing base64, fixture
//! loading, JSON reading, or SSE frame parsing in each crate. Add a helper here
//! (once) rather than copying it per crate.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The repository root, resolved at compile time.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

/// `tests/fixtures` at the repo root.
pub fn fixtures_dir() -> PathBuf {
    repo_root().join("tests/fixtures")
}

/// A path inside `tests/fixtures`.
pub fn fixture(relative: &str) -> PathBuf {
    fixtures_dir().join(relative)
}

/// Read a JSON value from a path.
pub fn read_json(path: impl AsRef<Path>) -> Value {
    let raw = std::fs::read_to_string(path.as_ref())
        .unwrap_or_else(|e| panic!("read {}: {e}", path.as_ref().display()));
    serde_json::from_str(&raw).expect("parse json")
}

/// Read a fixture JSON value.
pub fn fixture_json(relative: &str) -> Value {
    read_json(fixture(relative))
}

/// The pinned reference checkout, from `OPENCODE_REFERENCE` or the default.
pub fn reference_root() -> Option<PathBuf> {
    let candidate = std::env::var_os("OPENCODE_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/opencode/loc/opencode"));
    candidate.is_dir().then_some(candidate)
}

/// base64url without padding (matches the reference `Encoding.encodeBase64Url`).
pub fn encode_b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Decode base64url without padding.
pub fn decode_b64url(value: &str) -> Result<Vec<u8>, base64::DecodeError> {
    URL_SAFE_NO_PAD.decode(value)
}

/// Extract the `data:` payloads from an SSE body, ignoring comments/keep-alives.
pub fn sse_data_payloads(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for line in body.lines() {
        if line.is_empty() {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("data:") {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(rest.trim_start());
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// The first SSE `data:` payload parsed as JSON.
pub fn first_sse_json(body: &str) -> Option<Value> {
    sse_data_payloads(body)
        .first()
        .and_then(|d| serde_json::from_str(d).ok())
}
