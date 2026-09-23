//! Port of packages/core/test/session-todo.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `update` replaces the persisted todos for a session in the
//! given order and publishes an `updated` payload `{ sessionID, todos }` for each
//! update, `get` returns the current list, and an empty update clears it.
//! Re-derived: the `Database`/`TodoTable`/`EventV2` service wiring and the
//! persisted `position` column assertions are dropped; the in-memory store and
//! published payload shape remain.

use opencode_core::session_todo::{SessionTodoStore, Todo};
use serde_json::json;

const NOTE: &str = "porting: session todo store not implemented";

fn todo(content: &str, status: &str, priority: &str) -> Todo {
    Todo {
        content: content.into(),
        status: status.into(),
        priority: priority.into(),
    }
}

#[test]
#[ignore = "porting: session todo store not implemented"]
fn replaces_persisted_todos_in_order_and_publishes_updates() {
    let mut store = SessionTodoStore::new();
    let session_id = "ses_todo_test";

    let first = vec![
        todo("second", "pending", "low"),
        todo("first", "in_progress", "high"),
    ];
    store.update(session_id, first.clone()).expect(NOTE);
    assert_eq!(store.get(session_id).expect(NOTE), first);

    let replacement = vec![todo("replacement", "completed", "medium")];
    store.update(session_id, replacement.clone()).expect(NOTE);
    assert_eq!(store.get(session_id).expect(NOTE), replacement);

    store.update(session_id, vec![]).expect(NOTE);
    assert!(store.get(session_id).expect(NOTE).is_empty());

    assert_eq!(
        store.published().expect(NOTE),
        vec![
            json!({ "sessionID": session_id, "todos": [ first[0].to_json(), first[1].to_json() ] }),
            json!({ "sessionID": session_id, "todos": [ replacement[0].to_json() ] }),
            json!({ "sessionID": session_id, "todos": [] }),
        ]
    );
}
