//! Conformance tests: exercise the Rust server over a real socket using the
//! same client the CLI uses.
//!
//! When `REFERENCE_BASE_URL` points at a running reference opencode server, the
//! responses are compared. Without it, the tests only assert self-consistency.

use futures::StreamExt;
use opencode_client::Client;
use opencode_core::EventBus;
use opencode_schema::{EventEnvelope, EventType};
use opencode_server::{router, AppState};
use std::net::SocketAddr;
use std::time::Duration;

async fn spawn(state: AppState) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = router(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

#[tokio::test]
async fn health_matches_contract() {
    let addr = spawn(AppState::new()).await;
    let client = Client::new(format!("http://{addr}"));
    let health = client.health().await.expect("health request");
    assert!(health.healthy);

    if let Ok(reference) = std::env::var("REFERENCE_BASE_URL") {
        let reference_health = Client::new(reference)
            .health()
            .await
            .expect("reference health request");
        assert_eq!(
            reference_health.healthy, health.healthy,
            "health disagrees with reference"
        );
    }
}

#[tokio::test]
async fn event_stream_delivers_a_published_event() {
    let state = AppState::new();
    let addr = spawn(state.clone()).await;
    let client = Client::new(format!("http://{addr}"));

    let mut events = Box::pin(client.events());

    // Publish repeatedly so the test does not depend on subscription timing.
    let bus = state.bus.clone();
    let publisher = tokio::spawn(async move {
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            bus.publish(EventEnvelope::new(
                EventType::SessionCreated,
                serde_json::json!({ "sessionID": "ses_conformance" }),
            ));
        }
    });

    // The stream opens with a `server.connected` bootstrap event; skip it.
    let connected = tokio::time::timeout(Duration::from_secs(5), events.next())
        .await
        .expect("timed out waiting for server.connected")
        .expect("stream ended")
        .expect("client error");
    assert_eq!(connected.name(), "server.connected");

    let event = tokio::time::timeout(Duration::from_secs(5), events.next())
        .await
        .expect("timed out waiting for an event")
        .expect("stream ended")
        .expect("client error");

    publisher.abort();
    assert_eq!(event.name(), "session.created");
    assert_eq!(event.data["sessionID"], "ses_conformance");
}
