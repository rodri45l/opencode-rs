//! Port of packages/core/test/plugin/provider-snowflake-cortex.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the plugin is registered before the openai-compatible
//! fallback, tokens resolve from the Snowflake env vars with an options
//! fallback, usage is requested, `max_tokens` is rewritten to
//! `max_completion_tokens` while unknown bodies pass through, a `400`
//! "Conversation complete" body becomes a stop response, and streaming chunks
//! rewrite an empty role to `assistant`. Re-derived: the `AISDK` service wiring
//! and SDK mocking are replaced by direct plugin helpers.

use std::collections::BTreeMap;

use opencode_core::provider_snowflake_cortex::{CortexOptions, SnowflakeCortexPlugin};

const NOTE: &str = "porting: snowflake cortex plugin not implemented";

fn env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

#[test]
#[ignore = "porting: snowflake cortex plugin not implemented"]
fn is_registered_before_openai_compatible() {
    let snowflake = SnowflakeCortexPlugin::registry_index("snowflake-cortex").expect(NOTE);
    let fallback = SnowflakeCortexPlugin::registry_index("openai-compatible").expect(NOTE);
    assert!(snowflake < fallback);
}

#[test]
#[ignore = "porting: snowflake cortex plugin not implemented"]
fn resolves_tokens_from_env_with_options_fallback() {
    let pat = env(&[("SNOWFLAKE_CORTEX_PAT", "test-pat")]);
    assert_eq!(
        SnowflakeCortexPlugin::resolve_token(&pat, &CortexOptions::default()).expect(NOTE),
        Some("test-pat".to_string())
    );

    let token = env(&[("SNOWFLAKE_CORTEX_TOKEN", "oauth-token")]);
    assert_eq!(
        SnowflakeCortexPlugin::resolve_token(&token, &CortexOptions::default()).expect(NOTE),
        Some("oauth-token".to_string())
    );

    let empty = env(&[]);
    assert_eq!(
        SnowflakeCortexPlugin::resolve_token(
            &empty,
            &CortexOptions {
                api_key: Some("options-pat".into()),
                ..CortexOptions::default()
            }
        )
        .expect(NOTE),
        Some("options-pat".to_string())
    );
    assert_eq!(
        SnowflakeCortexPlugin::resolve_token(
            &empty,
            &CortexOptions {
                token: Some("options-token".into()),
                ..CortexOptions::default()
            }
        )
        .expect(NOTE),
        Some("options-token".to_string())
    );
}

#[test]
#[ignore = "porting: snowflake cortex plugin not implemented"]
fn requests_usage_on_the_sdk_options() {
    let pat = env(&[("SNOWFLAKE_CORTEX_PAT", "test-pat")]);
    assert!(SnowflakeCortexPlugin::include_usage(&pat, &CortexOptions::default()).expect(NOTE));
}

#[test]
#[ignore = "porting: snowflake cortex plugin not implemented"]
fn rewrites_max_tokens_and_preserves_other_bodies() {
    let rewritten = SnowflakeCortexPlugin::rewrite_request_body(
        r#"{"model":"claude-sonnet-4-6","max_tokens":1024}"#,
    )
    .expect(NOTE);
    let value: serde_json::Value = serde_json::from_str(&rewritten).expect(NOTE);
    assert_eq!(value["max_completion_tokens"], serde_json::json!(1024));
    assert!(value.get("max_tokens").is_none());

    let original = r#"{"model":"claude-sonnet-4-6","temperature":0.7}"#;
    assert_eq!(
        SnowflakeCortexPlugin::rewrite_request_body(original).expect(NOTE),
        original
    );

    let invalid = "{ not json }";
    assert_eq!(
        SnowflakeCortexPlugin::rewrite_request_body(invalid).expect(NOTE),
        invalid
    );
}

#[test]
#[ignore = "porting: snowflake cortex plugin not implemented"]
fn treats_conversation_complete_as_a_stop_response() {
    let (status, body) =
        SnowflakeCortexPlugin::rewrite_response(400, r#"{"message":"Conversation complete"}"#)
            .expect(NOTE);
    assert_eq!(status, 200);
    let value: serde_json::Value = serde_json::from_str(&body).expect(NOTE);
    assert_eq!(
        value["choices"][0]["finish_reason"],
        serde_json::json!("stop")
    );

    let (status, _) =
        SnowflakeCortexPlugin::rewrite_response(400, r#"{"message":"Invalid model"}"#).expect(NOTE);
    assert_eq!(status, 400);
    let (status, _) = SnowflakeCortexPlugin::rewrite_response(401, "Unauthorized").expect(NOTE);
    assert_eq!(status, 401);
}

#[test]
#[ignore = "porting: snowflake cortex plugin not implemented"]
fn rewrites_empty_streaming_roles_to_assistant() {
    let chunk =
        "data: {\"choices\":[{\"delta\":{\"role\":\"\",\"content\":\"Hi\"},\"index\":0}]}\n\n";
    let rewritten = SnowflakeCortexPlugin::rewrite_streaming_role(chunk).expect(NOTE);
    assert!(rewritten.contains("\"role\":\"assistant\""));
    assert!(!rewritten.contains("\"role\":\"\""));
}
