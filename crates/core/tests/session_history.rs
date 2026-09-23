//! Port of packages/core/test/session-history.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a session with no events returns an exhausted page, `after`
//! is an exclusive aggregate sequence, pages are returned in aggregate order with
//! correct `hasMore`, and a missing session fails with a not-found error.
//! Re-derived: the Database/EventV2/projector wiring is replaced by an in-memory
//! history with explicit `append` calls.

use opencode_core::session_history::SessionHistory;

const NOTE: &str = "porting: session history not implemented";

fn seqs(page: &opencode_core::session_history::HistoryPage) -> Vec<u64> {
    page.events.iter().map(|event| event.seq).collect()
}

#[test]
fn returns_an_exhausted_page_for_a_session_with_no_events() {
    let mut history = SessionHistory::new();
    history.create("ses_empty_history").expect(NOTE);
    let page = history.history("ses_empty_history", None, 10).expect(NOTE);
    assert!(page.events.is_empty());
    assert!(!page.has_more);
}

#[test]
fn treats_after_as_an_exclusive_aggregate_sequence() {
    let mut history = SessionHistory::new();
    history.create("ses_history").expect(NOTE);
    history
        .append("ses_history", "session.next.agent.switched.1")
        .expect(NOTE);
    history
        .append("ses_history", "session.next.agent.switched.1")
        .expect(NOTE);

    let page = history.history("ses_history", Some(1), 10).expect(NOTE);
    assert_eq!(seqs(&page), vec![2]);
    assert!(!page.has_more);
}

#[test]
fn paginates_in_aggregate_order_without_duplicates() {
    let mut history = SessionHistory::new();
    history.create("ses_pages").expect(NOTE);
    for _ in 0..3 {
        history
            .append("ses_pages", "session.next.agent.switched.1")
            .expect(NOTE);
    }

    let first = history.history("ses_pages", None, 2).expect(NOTE);
    assert!(first.has_more);
    assert_eq!(seqs(&first), vec![1, 2]);

    let second = history
        .history("ses_pages", Some(*seqs(&first).last().expect(NOTE)), 2)
        .expect(NOTE);
    assert!(!second.has_more);
    assert_eq!(seqs(&second), vec![3]);
}

#[test]
fn fails_with_not_found_for_a_missing_session() {
    let history = SessionHistory::new();
    assert!(history.history("ses_missing", None, 10).is_err());
}
