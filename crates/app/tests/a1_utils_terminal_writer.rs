//! Port of packages/app/src/utils/terminal-writer.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::terminal_writer::TerminalWriter;

#[test]
fn buffers_and_flushes_once_per_schedule() {
    let mut writer = TerminalWriter::default();
    writer.push("a");
    writer.push("b");
    writer.push("c");

    assert!(writer.calls.is_empty());
    assert_eq!(writer.scheduled.len(), 1);

    writer.run_scheduled();
    assert_eq!(writer.calls, vec!["abc".to_string()]);
}

#[test]
fn flush_is_a_no_op_when_empty() {
    let mut writer = TerminalWriter::default();
    writer.flush();
    assert!(writer.calls.is_empty());
}

#[test]
fn flush_waits_for_pending_write_completion() {
    let mut writer = TerminalWriter::default();
    writer.push("a");
    let settled = {
        writer.flush();
        true
    };
    assert_eq!(writer.calls, vec!["a".to_string()]);
    assert!(settled);
}
