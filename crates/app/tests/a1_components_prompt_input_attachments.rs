//! Port of packages/app/src/components/prompt-input/attachments.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::prompt_attachments::{
    attachment_mime, paste_mode, pick_attachment_files, AttachmentFile, PickOutcome,
};

fn file(name: &str, mime: &str, bytes: &[u8]) -> AttachmentFile {
    AttachmentFile {
        name: name.into(),
        browser_mime: mime.into(),
        bytes: bytes.to_vec(),
    }
}

#[test]
fn keeps_pdfs_when_the_browser_reports_the_mime() {
    assert_eq!(
        attachment_mime(&file("guide.pdf", "application/pdf", b"%PDF-1.7")),
        Some("application/pdf".to_string())
    );
}

#[test]
fn normalizes_structured_text_types_to_text_plain() {
    assert_eq!(
        attachment_mime(&file("data.json", "application/json", b"{\"ok\":true}\n")),
        Some("text/plain".to_string())
    );
}

#[test]
fn accepts_text_files_even_with_a_misleading_browser_mime() {
    assert_eq!(
        attachment_mime(&file("main.ts", "video/mp2t", b"export const x = 1\n")),
        Some("text/plain".to_string())
    );
}

#[test]
fn rejects_binary_files() {
    assert_eq!(
        attachment_mime(&file(
            "blob.bin",
            "application/octet-stream",
            &[0, 255, 1, 2]
        )),
        None
    );
}

#[test]
fn reads_the_current_project_directory_for_every_native_picker_invocation() {
    let mut out = PickOutcome::default();
    let mut fallback = 0;
    pick_attachment_files(
        "C:\\Projects\\LoremIpsum",
        true,
        false,
        &mut fallback,
        &mut out,
    );
    pick_attachment_files(
        "C:\\Projects\\DolorSit",
        true,
        false,
        &mut fallback,
        &mut out,
    );
    assert_eq!(
        out.picked_paths,
        vec![
            "C:\\Projects\\LoremIpsum".to_string(),
            "C:\\Projects\\DolorSit".to_string()
        ]
    );
}

#[test]
fn uses_the_browser_file_input_when_no_native_picker_exists() {
    let mut out = PickOutcome::default();
    let mut fallback = 0;
    pick_attachment_files(
        "/projects/consectetur-adipiscing",
        false,
        false,
        &mut fallback,
        &mut out,
    );
    assert_eq!(fallback, 1);
}

#[test]
fn reports_native_picker_failures_without_rejecting() {
    let mut out = PickOutcome::default();
    let mut fallback = 0;
    pick_attachment_files(
        "C:\\Projects\\LoremIpsum",
        true,
        true,
        &mut fallback,
        &mut out,
    );
    assert_eq!(out.errors, vec!["picker unavailable".to_string()]);
}

#[test]
fn uses_native_paste_for_short_single_line_text() {
    assert_eq!(paste_mode("hello world"), "native");
}

#[test]
fn uses_manual_paste_for_multiline_text() {
    assert_eq!(paste_mode("{\n  \"ok\": true\n}"), "manual");
    assert_eq!(paste_mode("a\r\nb"), "manual");
}

#[test]
fn uses_manual_paste_for_large_text() {
    assert_eq!(paste_mode(&"x".repeat(8000)), "manual");
}
