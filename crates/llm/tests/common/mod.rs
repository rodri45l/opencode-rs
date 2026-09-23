//! Shared helpers for the ported `packages/llm/test` suite.
#![allow(dead_code)]

use serde_json::{json, Value};
use std::path::PathBuf;

/// Reference weather tool name used by the recorded scenarios.
pub const WEATHER_TOOL_NAME: &str = "get_weather";

/// The reference `weatherTool` definition.
pub fn weather_tool() -> Value {
    json!({
        "name": WEATHER_TOOL_NAME,
        "description": "Get the current weather for a city.",
        "inputSchema": {
            "type": "object",
            "properties": { "city": { "type": "string" } },
            "required": ["city"]
        }
    })
}

/// The reference `weatherToolLoopRequest`.
pub fn weather_tool_loop_request() -> Value {
    json!({
        "model": {},
        "system": "Call tools exactly as requested.",
        "prompt": "What is the weather in Paris?",
        "tools": [weather_tool()],
    })
}

/// Absolute path to the copied recording fixtures.
pub fn recordings_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata")
        .join("recordings")
}

/// Whether a copied recording cassette exists.
pub fn recording_exists(group: &str, name: &str) -> bool {
    recordings_dir()
        .join(group)
        .join(format!("{name}.json"))
        .exists()
}

/// Load a copied recording cassette by group and name.
pub fn recording(group: &str, name: &str) -> Value {
    let path = recordings_dir().join(group).join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).expect("valid cassette JSON")
}

/// A large system prompt that crosses provider prefix-cache thresholds.
pub fn large_cacheable_system() -> String {
    "You are a helpful assistant. ".repeat(400)
}
