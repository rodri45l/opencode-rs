//! Port of packages/core/test/pty/info-schema.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `Pty.Info` accepts pid `0` (Windows ConPTY assigns the pid
//! asynchronously) and positive pids, rejects negative pids, and accepts an exit
//! code for retained exited sessions.

use opencode_core::pty::{Info, PtyStatus};
use serde_json::json;

const NOTE: &str = "porting: pty info schema not implemented";

fn sample(pid: i64) -> serde_json::Value {
    json!({
        "id": "pty_01J5Y5H0AH4Q4NXJ6P4C3P5V2K",
        "title": "demo",
        "command": "cmd.exe",
        "args": [],
        "cwd": "C:\\",
        "status": "running",
        "pid": pid,
    })
}

#[test]
fn accepts_pid_zero_assigned_asynchronously() {
    let info = Info::decode(&sample(0)).expect(NOTE);
    assert_eq!(info.pid, 0);
}

#[test]
fn accepts_a_positive_pid() {
    let info = Info::decode(&sample(48012)).expect(NOTE);
    assert_eq!(info.pid, 48012);
}

#[test]
fn rejects_a_negative_pid() {
    assert!(Info::decode(&sample(-1)).is_err());
}

#[test]
fn accepts_an_exit_code_for_retained_exited_sessions() {
    let mut value = sample(48012);
    value["status"] = json!("exited");
    value["exitCode"] = json!(4);

    let info = Info::decode(&value).expect(NOTE);
    assert_eq!(info.status, PtyStatus::Exited);
    assert_eq!(info.exit_code, Some(4));
}
