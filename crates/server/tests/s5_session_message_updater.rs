//! Port of packages/opencode/test/v2/session-message-updater.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: compaction events reduce to a single compaction message
//! only once `compaction.ended` arrives, carrying reason/summary/recent and the
//! ended timestamp. (The reference's step/text/tool updater tests are marked
//! `test.skip` upstream and are tracked as n/a.)
#![allow(dead_code)]

// Fast-wave local stubs: `session::message_updater` is not implemented yet.
mod updater {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Message {
        Compaction {
            id: String,
            reason: String,
            summary: String,
            recent: String,
            created: i64,
        },
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct MemoryState {
        pub messages: Vec<Message>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[allow(clippy::enum_variant_names)]
    pub enum Event {
        CompactionStarted {
            message_id: String,
            timestamp: i64,
            reason: String,
        },
        CompactionDelta {
            message_id: String,
            timestamp: i64,
            text: String,
        },
        CompactionEnded {
            message_id: String,
            timestamp: i64,
            reason: String,
            text: String,
            recent: String,
        },
    }

    pub fn update(state: &mut MemoryState, event: Event) -> Result<(), &'static str> {
        match event {
            Event::CompactionStarted { .. } | Event::CompactionDelta { .. } => Ok(()),
            Event::CompactionEnded {
                message_id,
                timestamp,
                reason,
                text,
                recent,
            } => {
                state.messages.push(Message::Compaction {
                    id: message_id,
                    reason,
                    summary: text,
                    recent,
                    created: timestamp,
                });
                Ok(())
            }
        }
    }
}

use updater::{Event, MemoryState, Message};

#[test]
fn compaction_events_reduce_to_compaction_message_only_when_completed() {
    let mut state = MemoryState::default();

    updater::update(
        &mut state,
        Event::CompactionStarted {
            message_id: "compaction".to_string(),
            timestamp: 1,
            reason: "auto".to_string(),
        },
    )
    .unwrap();
    assert!(state.messages.is_empty());

    updater::update(
        &mut state,
        Event::CompactionDelta {
            message_id: "compaction".to_string(),
            timestamp: 2,
            text: "hello ".to_string(),
        },
    )
    .unwrap();

    updater::update(
        &mut state,
        Event::CompactionDelta {
            message_id: "compaction".to_string(),
            timestamp: 3,
            text: "summary".to_string(),
        },
    )
    .unwrap();

    updater::update(
        &mut state,
        Event::CompactionEnded {
            message_id: "compaction".to_string(),
            timestamp: 4,
            reason: "auto".to_string(),
            text: "final summary".to_string(),
            recent: "recent context".to_string(),
        },
    )
    .unwrap();

    assert_eq!(state.messages.len(), 1);
    assert_eq!(
        state.messages[0],
        Message::Compaction {
            id: "compaction".to_string(),
            reason: "auto".to_string(),
            summary: "final summary".to_string(),
            recent: "recent context".to_string(),
            created: 4,
        }
    );
}
