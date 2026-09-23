//! Port of packages/opencode/test/mcp/auth.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: concurrent token/client-info updates for one server merge
//! into a single entry rather than clobbering each other. The reference drives
//! two service instances over a shared JSON file; this port drives two threads
//! over a shared in-memory store.

use opencode_server::port::mcp::{ClientInfo, McpAuth, OAuthTokens};
use std::sync::{Arc, Mutex};

#[test]
fn serializes_concurrent_auth_updates() {
    let auth = Arc::new(Mutex::new(McpAuth::new()));
    let url = "https://mcp.posthog.com/mcp";

    let first = {
        let auth = Arc::clone(&auth);
        std::thread::spawn(move || {
            auth.lock().expect("lock").update_tokens(
                "posthog",
                OAuthTokens {
                    access_token: "access-token".to_string(),
                    refresh_token: None,
                    expires_at: None,
                    scope: None,
                },
                url,
            );
        })
    };

    let second = {
        let auth = Arc::clone(&auth);
        std::thread::spawn(move || {
            auth.lock().expect("lock").update_client_info(
                "posthog",
                ClientInfo {
                    client_id: "client-id".to_string(),
                    client_secret: None,
                },
                url,
            );
        })
    };

    first.join().expect("first update");
    second.join().expect("second update");

    let store = auth.lock().expect("lock");
    let entry = store.get("posthog").expect("entry");
    assert_eq!(
        entry
            .tokens
            .as_ref()
            .map(|tokens| tokens.access_token.as_str()),
        Some("access-token")
    );
    assert_eq!(
        entry
            .client_info
            .as_ref()
            .map(|info| info.client_id.as_str()),
        Some("client-id")
    );
    assert_eq!(entry.server_url.as_deref(), Some(url));
}
