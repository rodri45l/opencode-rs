//! Port of packages/core/test/pty/ticket.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: tickets are consumed exactly once, reject a different
//! request scope (directory or workspace), and expire after the TTL.

use std::time::Duration;

use opencode_core::pty::{PtyId, PtyTicket, TicketScope};

const NOTE: &str = "porting: pty ticket not implemented";

fn scope(pty_id: &str, directory: Option<&str>) -> TicketScope {
    TicketScope {
        pty_id: pty_id.into(),
        directory: directory.map(str::to_string),
        workspace_id: None,
    }
}

#[test]
fn consumes_tickets_once() {
    let tickets = PtyTicket::default();
    let id = PtyId::ascending();
    let scope = scope(id.as_str(), Some("/tmp/a"));
    let issued = tickets.issue(&scope).expect(NOTE);

    assert!(tickets.consume(&scope, &issued.ticket).expect(NOTE));
    assert!(!tickets.consume(&scope, &issued.ticket).expect(NOTE));
}

#[test]
fn rejects_tickets_scoped_to_a_different_request() {
    let tickets = PtyTicket::default();
    let id = PtyId::ascending();
    let issued = tickets
        .issue(&scope(id.as_str(), Some("/tmp/a")))
        .expect(NOTE);

    assert!(!tickets
        .consume(&scope(id.as_str(), Some("/tmp/b")), &issued.ticket)
        .expect(NOTE));
    assert!(tickets
        .consume(&scope(id.as_str(), Some("/tmp/a")), &issued.ticket)
        .expect(NOTE));
}

#[test]
fn rejects_tickets_scoped_to_a_different_workspace() {
    let tickets = PtyTicket::default();
    let id = PtyId::ascending();
    let mut issue_scope = scope(id.as_str(), None);
    issue_scope.workspace_id = Some("wrk_a".into());
    let issued = tickets.issue(&issue_scope).expect(NOTE);

    let mut other = scope(id.as_str(), None);
    other.workspace_id = Some("wrk_b".into());
    assert!(!tickets.consume(&other, &issued.ticket).expect(NOTE));
    assert!(tickets.consume(&issue_scope, &issued.ticket).expect(NOTE));
}

#[test]
fn rejects_tickets_after_the_ttl_elapses() {
    let tickets = PtyTicket::new(Duration::from_millis(5));
    let id = PtyId::ascending();
    let scope = scope(id.as_str(), None);
    let issued = tickets.issue(&scope).expect(NOTE);

    std::thread::sleep(Duration::from_millis(25));
    assert!(!tickets.consume(&scope, &issued.ticket).expect(NOTE));
}
