//! Port of packages/enterprise/test/core/share.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/enterprise/src/core/share.ts. The reference
//! Effect service is re-derived as an owned in-memory service; the legacy
//! `share_event` migration case is deferred (see docs/TEST-PORT.md).

use opencode_integrations::{Share, ShareData, ShareError, ShareRef};

fn part(session_id: &str, id: &str, message_id: &str, text: &str) -> ShareData {
    ShareData::new(
        "part",
        serde_json::json!({
            "id": id,
            "sessionID": session_id,
            "messageID": message_id,
            "type": "text",
            "text": text,
        }),
    )
}

fn reference(info: &opencode_integrations::ShareInfo) -> ShareRef {
    ShareRef {
        id: info.id.clone(),
        secret: info.secret.clone(),
    }
}

#[test]
fn creates_a_share() {
    let mut share = Share::new();
    let session_id = "ses_share_create";
    let info = share.create(session_id);

    assert_eq!(info.session_id, session_id);
    assert!(!info.secret.is_empty());
    assert!(share.remove(&reference(&info)).is_ok());
}

#[test]
fn removes_a_share_as_admin() {
    let mut share = Share::new();
    let info = share.create("ses_admin");
    share.remove_admin(&info.id);
    assert!(share.get(&info.id).is_none());
}

#[test]
fn syncs_data_to_a_share() {
    let mut share = Share::new();
    let session_id = "ses_sync";
    let info = share.create(session_id);

    share
        .sync(
            &reference(&info),
            &[part(session_id, "part1", "msg1", "Hello")],
        )
        .unwrap();
    assert_eq!(share.data(&info.id).len(), 1);
    assert!(share.remove(&reference(&info)).is_ok());
}

#[test]
fn syncs_multiple_batches_of_data() {
    let mut share = Share::new();
    let session_id = "ses_batches";
    let info = share.create(session_id);

    share
        .sync(
            &reference(&info),
            &[part(session_id, "part1", "msg1", "Hello")],
        )
        .unwrap();
    share
        .sync(
            &reference(&info),
            &[part(session_id, "part2", "msg1", "World")],
        )
        .unwrap();
    assert_eq!(share.data(&info.id).len(), 2);
    assert!(share.remove(&reference(&info)).is_ok());
}

#[test]
fn retrieves_synced_data() {
    let mut share = Share::new();
    let session_id = "ses_retrieve";
    let info = share.create(session_id);

    share
        .sync(
            &reference(&info),
            &[
                part(session_id, "part1", "msg1", "Hello"),
                part(session_id, "part2", "msg1", "World"),
            ],
        )
        .unwrap();

    let result = share.data(&info.id);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].r#type, "part");
    assert_eq!(result[1].r#type, "part");
    assert!(share.remove(&reference(&info)).is_ok());
}

#[test]
fn retrieves_data_from_multiple_syncs() {
    let mut share = Share::new();
    let session_id = "ses_multi";
    let info = share.create(session_id);

    share
        .sync(
            &reference(&info),
            &[part(session_id, "part1", "msg1", "Hello")],
        )
        .unwrap();
    share
        .sync(
            &reference(&info),
            &[part(session_id, "part2", "msg2", "World")],
        )
        .unwrap();
    share
        .sync(&reference(&info), &[part(session_id, "part3", "msg3", "!")])
        .unwrap();

    let result = share.data(&info.id);
    assert_eq!(result.len(), 3);
    assert_eq!(
        result.iter().filter(|item| item.r#type == "part").count(),
        3
    );
    assert!(share.remove(&reference(&info)).is_ok());
}

#[test]
fn returns_latest_data_when_syncing_duplicate_parts() {
    let mut share = Share::new();
    let session_id = "ses_duplicate";
    let info = share.create(session_id);

    share
        .sync(
            &reference(&info),
            &[part(session_id, "part1", "msg1", "Hello")],
        )
        .unwrap();
    share
        .sync(
            &reference(&info),
            &[part(session_id, "part1", "msg1", "Hello Updated")],
        )
        .unwrap();

    let result = share.data(&info.id);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].r#type, "part");
    assert_eq!(result[0].data["text"], "Hello Updated");
    assert!(share.remove(&reference(&info)).is_ok());
}

#[test]
fn returns_empty_array_for_share_with_no_data() {
    let mut share = Share::new();
    let info = share.create("ses_empty");
    assert!(share.data(&info.id).is_empty());
    assert!(share.remove(&reference(&info)).is_ok());
}

#[test]
fn throws_error_for_invalid_secret() {
    let mut share = Share::new();
    let session_id = "ses_invalid_secret";
    let info = share.create(session_id);

    let result = share.sync(
        &ShareRef {
            id: info.id.clone(),
            secret: "invalid-secret".to_string(),
        },
        &[part(session_id, "part1", "msg1", "Test")],
    );
    assert_eq!(result, Err(ShareError::InvalidSecret));
    assert!(share.remove(&reference(&info)).is_ok());
}

#[test]
fn throws_error_for_non_existent_share() {
    let mut share = Share::new();
    let result = share.sync(
        &ShareRef {
            id: "non-existent-id".to_string(),
            secret: "some-secret".to_string(),
        },
        &[part("ses_missing", "part1", "msg1", "Test")],
    );
    assert_eq!(result, Err(ShareError::NotFound));
}

#[test]
fn handles_different_data_types() {
    let mut share = Share::new();
    let session_id = "ses_types";
    let info = share.create(session_id);

    let data = vec![
        ShareData::new(
            "session",
            serde_json::json!({ "id": session_id, "status": "running" }),
        ),
        ShareData::new(
            "message",
            serde_json::json!({ "id": "msg1", "sessionID": session_id }),
        ),
        part(session_id, "part1", "msg1", "Hello"),
    ];
    share.sync(&reference(&info), &data).unwrap();

    let result = share.data(&info.id);
    assert_eq!(result.len(), 3);
    assert!(result.iter().any(|item| item.r#type == "session"));
    assert!(result.iter().any(|item| item.r#type == "message"));
    assert!(result.iter().any(|item| item.r#type == "part"));
    assert!(share.remove(&reference(&info)).is_ok());
}
