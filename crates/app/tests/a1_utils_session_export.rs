//! Port of packages/app/src/utils/session-export.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct Session {
    id: String,
    title: Option<String>,
    slug: Option<String>,
}

// Local stubs (fast wave): real module lands later.
fn session_export_filename(_session: &Session) -> String {
    String::new()
}

fn fetch_session_export(_session_id: &str, _client: &Client) -> Result<Export, String> {
    Ok(Export {
        info: None,
        messages: Vec::new(),
    })
}

struct Client;

#[derive(Clone, Debug, PartialEq)]
struct Export {
    info: Option<Session>,
    messages: Vec<String>,
}

#[test]
#[ignore = "porting: utils/session-export not implemented"]
fn generates_filename_from_title() {
    let session = Session {
        id: "ses_123".into(),
        title: Some("Clone PR in worktree from fork".into()),
        slug: None,
    };
    assert_eq!(
        session_export_filename(&session),
        "clone-pr-in-worktree-from-fork.json"
    );
}

#[test]
#[ignore = "porting: utils/session-export not implemented"]
fn generates_filename_from_slug_when_title_missing() {
    let session = Session {
        id: "ses_123".into(),
        title: None,
        slug: Some("my-session-slug".into()),
    };
    assert_eq!(session_export_filename(&session), "my-session-slug.json");
}

#[test]
#[ignore = "porting: utils/session-export not implemented"]
fn falls_back_to_id_when_title_and_slug_are_empty() {
    let session = Session {
        id: "ses_123".into(),
        title: None,
        slug: None,
    };
    assert_eq!(session_export_filename(&session), "ses_123.json");
}

#[test]
#[ignore = "porting: utils/session-export not implemented"]
fn fetches_full_transcript_from_client() {
    let session = Session {
        id: "ses_1".into(),
        title: Some("Test Session".into()),
        slug: None,
    };
    let result = fetch_session_export("ses_1", &Client);
    assert_eq!(
        result,
        Ok(Export {
            info: Some(session),
            messages: vec!["msg_1".to_string()],
        })
    );
}

#[test]
#[ignore = "porting: utils/session-export not implemented"]
fn throws_when_session_not_found() {
    let result = fetch_session_export("ses_missing", &Client);
    assert_eq!(result, Err("Session not found: ses_missing".to_string()));
}
