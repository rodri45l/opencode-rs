//! Port of packages/core/test/pty/pty-session.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: operating on a missing session fails with a typed
//! not-found error carrying the pty id; attaching after exit fails with a typed
//! exited error; a fresh attachment replays the whole buffer and reports the
//! buffer cursor, while a tail attachment (`cursor = -1`) replays nothing but
//! reports the same cursor; exit events carry the process exit code; and pty
//! creation defaults the command to the configured shell with login args and the
//! cwd from the location.
//!
//! Re-derived (dropped): the live `Pty`/`EventV2`/`Config`/`Location` service
//! wiring, the native pty spawn/attach/detach lifecycle, the event queue, and
//! the timeout-driven output/isolation tests.

#![allow(dead_code)]

const NOTE: &str = "porting: pty session not implemented";

#[derive(Debug, PartialEq, Eq)]
enum PtyError {
    NotImplemented,
    NotFound(String),
    Exited(String),
}

#[derive(Debug, PartialEq, Eq)]
enum PtyOp {
    Get,
    Update,
    Remove,
    Write,
    Attach,
}

#[derive(Debug, PartialEq, Eq)]
struct Replay {
    data: String,
    cursor: i64,
}

#[derive(Debug, PartialEq, Eq)]
struct ExitEvent {
    exit_code: Option<i32>,
}

#[derive(Debug, PartialEq, Eq)]
struct CreateDefaults {
    command: String,
    args: Vec<String>,
    cwd: String,
}

#[allow(dead_code)]
#[derive(Default)]
struct OutputBuffer {
    data: String,
}

impl OutputBuffer {
    fn new() -> Self {
        Self::default()
    }

    fn append(&mut self, _chunk: &str) {}

    fn replay(&self, _cursor: Option<i64>) -> Result<Replay, PtyError> {
        Err(PtyError::NotImplemented)
    }
}

fn op_on_missing(_op: PtyOp, _id: &str) -> Result<(), PtyError> {
    Err(PtyError::NotImplemented)
}

fn attach_after_exit(_id: &str, _exited: bool) -> Result<(), PtyError> {
    Err(PtyError::NotImplemented)
}

fn exit_event(_exit_code: Option<i32>) -> Result<ExitEvent, PtyError> {
    Err(PtyError::NotImplemented)
}

fn create_defaults(
    _configured_shell: Option<&str>,
    _cwd: &str,
) -> Result<CreateDefaults, PtyError> {
    Err(PtyError::NotImplemented)
}

#[test]
#[ignore = "porting: pty session not implemented"]
fn missing_sessions_return_typed_not_found_errors() {
    for op in [
        PtyOp::Get,
        PtyOp::Update,
        PtyOp::Remove,
        PtyOp::Write,
        PtyOp::Attach,
    ] {
        assert_eq!(
            op_on_missing(op, "pty_missing"),
            Err(PtyError::NotFound("pty_missing".into()))
        );
    }
}

#[test]
#[ignore = "porting: pty session not implemented"]
fn rejects_attach_after_the_session_has_exited() {
    assert_eq!(
        attach_after_exit("pty_1", true),
        Err(PtyError::Exited("pty_1".into()))
    );
    assert!(attach_after_exit("pty_1", false).is_ok());
}

#[test]
#[ignore = "porting: pty session not implemented"]
fn fresh_attachments_replay_the_buffered_output() {
    let mut buffer = OutputBuffer::new();
    buffer.append("AAA\n");
    buffer.append("BBB\n");

    let replay = buffer.replay(None).expect(NOTE);
    assert_eq!(replay.data, "AAA\nBBB\n");
    assert_eq!(replay.cursor, 8);
}

#[test]
#[ignore = "porting: pty session not implemented"]
fn tail_attachments_skip_the_buffer_but_keep_the_cursor() {
    let mut buffer = OutputBuffer::new();
    buffer.append("AAA\n");
    buffer.append("BBB\n");

    let full = buffer.replay(None).expect(NOTE);
    let tail = buffer.replay(Some(-1)).expect(NOTE);
    assert_eq!(tail.data, "");
    assert_eq!(tail.cursor, full.cursor);
}

#[test]
#[ignore = "porting: pty session not implemented"]
fn exit_events_carry_the_process_exit_code() {
    assert_eq!(
        exit_event(Some(3)).expect(NOTE),
        ExitEvent { exit_code: Some(3) }
    );
    assert_eq!(
        exit_event(Some(0)).expect(NOTE),
        ExitEvent { exit_code: Some(0) }
    );
}

#[test]
#[ignore = "porting: pty session not implemented"]
fn create_defaults_the_shell_login_args_and_cwd() {
    let defaults = create_defaults(Some("/bin/bash"), "/tmp").expect(NOTE);
    assert_eq!(
        defaults,
        CreateDefaults {
            command: "/bin/bash".into(),
            args: vec!["-l".into()],
            cwd: "/tmp".into(),
        }
    );
}
