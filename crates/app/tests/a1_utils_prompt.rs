//! Port of packages/app/src/utils/prompt.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
enum Part {
    Text {
        id: String,
        text: String,
        session_id: String,
        message_id: String,
    },
    File {
        id: String,
        mime: String,
        url: String,
        filename: String,
        session_id: String,
        message_id: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
enum PromptItem {
    Text {
        content: String,
    },
    Image {
        filename: String,
        mime: String,
        blob_id: String,
    },
}

// Local stub (fast wave): real module lands later.
fn extract_prompt_from_parts(_parts: &[Part]) -> Vec<PromptItem> {
    Vec::new()
}

#[test]
#[ignore = "porting: utils/prompt not implemented"]
fn restores_multiple_uploaded_attachments() {
    let parts = vec![
        Part::Text {
            id: "text_1".into(),
            text: "check these".into(),
            session_id: "ses_1".into(),
            message_id: "msg_1".into(),
        },
        Part::File {
            id: "file_1".into(),
            mime: "image/png".into(),
            url: "data:image/png;base64,AAA".into(),
            filename: "a.png".into(),
            session_id: "ses_1".into(),
            message_id: "msg_1".into(),
        },
        Part::File {
            id: "file_2".into(),
            mime: "application/pdf".into(),
            url: "data:application/pdf;base64,BBB".into(),
            filename: "b.pdf".into(),
            session_id: "ses_1".into(),
            message_id: "msg_1".into(),
        },
    ];

    let result = extract_prompt_from_parts(&parts);

    assert_eq!(result.len(), 3);
    assert_eq!(
        result[0],
        PromptItem::Text {
            content: "check these".into()
        }
    );
    assert!(matches!(
        &result[1],
        PromptItem::Image { filename, mime, .. } if filename == "a.png" && mime == "image/png"
    ));
    assert!(matches!(
        &result[2],
        PromptItem::Image { filename, mime, .. } if filename == "b.pdf" && mime == "application/pdf"
    ));
}
