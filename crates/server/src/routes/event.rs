//! `GET /api/event` and `GET /event` — the SSE event stream.

use crate::state::AppState;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;
use axum::Router;
use futures::stream::Stream;
use opencode_core::EventBus;
use opencode_protocol::routes::path;
use opencode_schema::EventEnvelope;
use std::convert::Infallible;

/// Routes for the event group.
pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(path::EVENT, get(subscribe))
        .route(path::EVENT_LEGACY, get(subscribe))
}

/// Stream server events as `text/event-stream`.
pub async fn subscribe(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let subscription = state.bus.subscribe();
    let stream = futures::stream::unfold(subscription, |mut sub| async move {
        let event = sub.recv().await?;
        Some((Ok::<_, Infallible>(to_sse(&event)), sub))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Encode an envelope as an SSE event.
pub fn to_sse(event: &EventEnvelope) -> Event {
    let data = serde_json::to_string(event).unwrap_or_else(|_| "{}".into());
    Event::default()
        .id(event.id.as_str())
        .event(event.name())
        .data(data)
}
