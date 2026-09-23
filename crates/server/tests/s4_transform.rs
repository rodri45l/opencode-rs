//! Port of packages/opencode/test/provider/transform.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/provider/transform.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure `ProviderTransform.providerOptions`, `variants`, and
//! `reasoningVariants` projections for the Cloudflare AI Gateway npm rewrites
//! (OpenAI Responses, Anthropic adaptive thinking, unified `/compat`).
//! Dropped: the `options`, `message`, and `schema` cases — `schema` is already
//! covered by `tests/provider_transform.rs`; the rest need the live SDK/catalog.

use serde_json::{json, Value};

use opencode_server::provider_util::{
    provider_options, transform_reasoning_variants as reasoning_variants,
    transform_variants as variants,
};

fn cf_model(api_id: &str) -> Value {
    let npm = if api_id.starts_with("openai/") {
        "@ai-sdk/openai"
    } else if api_id.starts_with("anthropic/") {
        "@ai-sdk/anthropic"
    } else {
        "ai-gateway-provider"
    };
    json!({
        "id": format!("cloudflare-ai-gateway/{api_id}"),
        "providerID": "cloudflare-ai-gateway",
        "name": api_id,
        "api": { "id": api_id, "url": "https://gateway.ai.cloudflare.com/v1/compat", "npm": npm }
    })
}

#[test]
fn openai_provider_options_put_reasoning_effort_on_the_responses_wire() {
    let opts = provider_options(
        &cf_model("openai/gpt-5.4"),
        &json!({ "reasoningEffort": "xhigh" }),
    )
    .unwrap();
    assert_eq!(opts, json!({ "openai": { "reasoningEffort": "xhigh" } }));
}

#[test]
fn reasoning_effort_reaches_the_compat_wire_for_workers_ai_models() {
    let opts = provider_options(
        &cf_model("workers-ai/@cf/moonshotai/kimi-k2.6"),
        &json!({ "reasoningEffort": "high" }),
    )
    .unwrap();
    assert_eq!(
        opts,
        json!({ "openaiCompatible": { "reasoningEffort": "high" } })
    );
}

#[test]
fn variants_output_for_openai_lands_xhigh_on_the_wire() {
    let variants = variants(&cf_model("openai/gpt-5.4")).unwrap();
    assert_eq!(
        variants["xhigh"],
        json!({
            "reasoningEffort": "xhigh",
            "reasoningSummary": "auto",
            "include": ["reasoning.encrypted_content"]
        })
    );
}

#[test]
fn reasoning_effort_variants_for_anthropic_land_as_native_adaptive_thinking() {
    let model = cf_model("anthropic/claude-sonnet-4-6");
    let variants = reasoning_variants(
        &json!([{ "type": "effort", "values": ["low", "medium", "high"] }]),
        &model,
    )
    .unwrap()
    .unwrap();
    assert_eq!(variants["high"], json!({ "effort": "high" }));
    let opts = provider_options(&model, &variants["high"]).unwrap();
    assert_eq!(
        opts.as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        vec!["anthropic"]
    );
}
