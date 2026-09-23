//! Port of packages/opencode/test/plugin/cloudflare.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Cloudflare AI Gateway plugin registers exactly one
//! auth method and no longer exposes a `chat.params` hook (OpenAI models ride
//! the Responses API passthrough).

use opencode_server::port::plugin::cloudflare_ai_gateway_auth_plugin;

#[test]
fn registers_the_cloudflare_ai_gateway_auth_method() {
    let hooks = cloudflare_ai_gateway_auth_plugin();
    assert_eq!(
        hooks.auth_provider.as_deref(),
        Some("cloudflare-ai-gateway")
    );
    assert_eq!(hooks.auth_methods.len(), 1);
}

#[test]
fn no_longer_drops_max_output_tokens() {
    let hooks = cloudflare_ai_gateway_auth_plugin();
    assert!(!hooks.has_chat_params);
}
