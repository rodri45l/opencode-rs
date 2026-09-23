//! Port of packages/app/src/utils/terminal-writer.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Default)]
struct TerminalWriter {
    calls: Vec<String>,
    scheduled: Vec<bool>,
}

impl TerminalWriter {
    // Local stubs (fast wave): real module lands later.
    fn push(&mut self, _data: &str) {}
    fn flush(&mut self) {}
    fn run_scheduled(&mut self) {}
}

#[test]
#[ignore = "porting: utils/terminal-writer not implemented"]
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
#[ignore = "porting: utils/terminal-writer not implemented"]
fn flush_is_a_no_op_when_empty() {
    let mut writer = TerminalWriter::default();
    writer.flush();
    assert!(writer.calls.is_empty());
}

#[test]
#[ignore = "porting: utils/terminal-writer not implemented"]
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
