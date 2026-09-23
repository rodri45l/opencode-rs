//! `GET /api/event` and `GET /event` — the SSE event stream.

use crate::state::AppState;
use axum::extract::State;
use axum::http::HeaderValue;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use opencode_core::EventBus;
use opencode_protocol::routes::path;
use opencode_schema::EventEnvelope;
use serde_json::json;
use std::convert::Infallible;

/// Routes for the event group.
pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(path::EVENT, get(subscribe))
        .route(path::EVENT_LEGACY, get(subscribe))
}

/// Stream server events as `text/event-stream`.
pub async fn subscribe(State(state): State<AppState>) -> Response {
    let subscription = state.bus.subscribe();
    let bootstrap = futures::stream::once(async {
        Ok::<_, Infallible>(
            Event::default().data(
                serde_json::to_string(&json!({
                    "id": opencode_schema::EventId::generate(),
                    "type": "server.connected",
                    "properties": {},
                    "data": {},
                }))
                .unwrap_or_default(),
            ),
        )
    });
    let events = futures::stream::unfold(subscription, |mut sub| async move {
        let event = sub.recv().await?;
        Some((Ok::<_, Infallible>(to_sse(&event)), sub))
    });
    let stream = futures::StreamExt::chain(bootstrap, events);
    let mut response = Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response();
    let headers = response.headers_mut();
    headers.insert(
        "cache-control",
        HeaderValue::from_static("no-cache, no-transform"),
    );
    headers.insert("x-accel-buffering", HeaderValue::from_static("no"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    response
}

/// Encode an envelope as an SSE event.
pub fn to_sse(event: &EventEnvelope) -> Event {
    let data = serde_json::to_string(event).unwrap_or_else(|_| "{}".into());
    Event::default()
        .id(event.id.as_str())
        .event(event.name())
        .data(data)
}
