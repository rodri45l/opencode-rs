//! Prompt submit in-flight guard.
//!
//! Port of the `submit` fix in packages/tui/src/component/prompt/index.tsx,
//! pinned by packages/tui/test/cli/tui/prompt-submit-race.test.ts (upstream
//! 18ef3cc). A shared in-flight flag makes concurrent submits a no-op so the
//! user's text cannot be lost to a double Enter.

use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};

/// Submit a prompt once, guarded against concurrent submissions.
///
/// Returns `true` when a submission carrying `input` was sent.
pub async fn submit_prompt<C, Create, S, Send>(
    guard: &AtomicBool,
    input: String,
    create_session: C,
    send_prompt: S,
) -> bool
where
    C: FnOnce() -> Create,
    Create: Future<Output = String>,
    S: FnOnce(String, String) -> Send,
    Send: Future<Output = ()>,
{
    if guard.swap(true, Ordering::SeqCst) {
        return false;
    }
    let result = if input.is_empty() {
        false
    } else {
        let session_id = create_session().await;
        send_prompt(session_id, input).await;
        true
    };
    guard.store(false, Ordering::SeqCst);
    result
}
