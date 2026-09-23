//! Port of packages/opencode/test/session/session-schema.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: undefined optional session fields are omitted from the
//! encoded wire shape, including nested summary/revert/project optionals.

use opencode_server::session::{
    encode_global, encode_info, GlobalSessionInfo, ProjectInfo, Revert, SessionInfo,
    SessionSummary, SessionTime, TokenCache, Tokens,
};
fn tokens() -> Tokens {
    Tokens {
        input: 0,
        output: 0,
        reasoning: 0,
        cache: TokenCache { read: 0, write: 0 },
    }
}

fn info() -> SessionInfo {
    SessionInfo {
        id: "ses_test".to_string(),
        slug: "test-session".to_string(),
        project_id: "proj-global".to_string(),
        workspace_id: None,
        directory: "/tmp/opencode".to_string(),
        parent_id: None,
        summary: None,
        cost: 0.0,
        tokens: tokens(),
        share: None,
        title: "Test session".to_string(),
        version: "1.0.0".to_string(),
        time: SessionTime {
            created: 1,
            updated: Some(2),
            compacting: None,
            archived: None,
        },
        permission: None,
        revert: None,
    }
}

#[test]
#[ignore = "porting: session schema not implemented"]
fn encodes_undefined_optional_session_fields_as_omitted_keys() {
    let encoded = encode_info(&info());

    for key in [
        "workspaceID",
        "parentID",
        "summary",
        "share",
        "permission",
        "revert",
    ] {
        assert!(encoded.get(key).is_none(), "expected {key} to be omitted");
    }
    assert!(encoded["time"].get("compacting").is_none());
    assert!(encoded["time"].get("archived").is_none());
    assert!(!encoded.to_string().contains("parentID"));
}

#[test]
#[ignore = "porting: session schema not implemented"]
fn encodes_undefined_optional_global_project_fields_as_omitted_keys() {
    let encoded = encode_global(&GlobalSessionInfo {
        info: info(),
        project: Some(ProjectInfo {
            id: "proj-global".to_string(),
            name: None,
            worktree: "/tmp/opencode".to_string(),
        }),
    });

    assert!(encoded.get("parentID").is_none());
    assert!(encoded["project"].get("name").is_none());
}

#[test]
#[ignore = "porting: session schema not implemented"]
fn encodes_nested_undefined_optional_session_fields_as_omitted_keys() {
    let mut value = info();
    value.summary = Some(SessionSummary {
        additions: 1,
        deletions: 2,
        files: 3,
        diffs: None,
    });
    value.revert = Some(Revert {
        message_id: "msg_test".to_string(),
        part_id: None,
        snapshot: None,
        diff: None,
    });

    let encoded = encode_info(&value);

    assert!(encoded["summary"].get("diffs").is_none());
    for key in ["partID", "snapshot", "diff"] {
        assert!(
            encoded["revert"].get(key).is_none(),
            "expected revert.{key} to be omitted"
        );
    }
}
