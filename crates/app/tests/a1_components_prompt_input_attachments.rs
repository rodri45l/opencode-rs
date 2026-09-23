//! Port of packages/app/src/components/prompt-input/attachments.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct AttachmentFile {
    name: String,
    browser_mime: String,
    bytes: Vec<u8>,
}

// Local stubs (fast wave): real module lands later.
fn attachment_mime(_file: &AttachmentFile) -> Option<String> {
    None
}

#[derive(Default)]
struct PickOutcome {
    picked_paths: Vec<String>,
    files: Vec<AttachmentFile>,
    fallback: usize,
    errors: Vec<String>,
}

fn pick_attachment_files(
    _directory: &str,
    _has_picker: bool,
    _picker_fails: bool,
    _fallback: &mut usize,
    _out: &mut PickOutcome,
) {
}

fn paste_mode(_text: &str) -> String {
    String::new()
}

fn file(name: &str, mime: &str, bytes: &[u8]) -> AttachmentFile {
    AttachmentFile {
        name: name.into(),
        browser_mime: mime.into(),
        bytes: bytes.to_vec(),
    }
}

#[test]
#[ignore = "porting: components/prompt-input/attachments not implemented"]
fn keeps_pdfs_when_the_browser_reports_the_mime() {
    assert_eq!(
        attachment_mime(&file("guide.pdf", "application/pdf", b"%PDF-1.7")),
        Some("application/pdf".to_string())
    );
}

#[test]
#[ignore = "porting: components/prompt-input/attachments not implemented"]
fn normalizes_structured_text_types_to_text_plain() {
    assert_eq!(
        attachment_mime(&file("data.json", "application/json", b"{\"ok\":true}\n")),
        Some("text/plain".to_string())
    );
}

#[test]
#[ignore = "porting: components/prompt-input/attachments not implemented"]
fn accepts_text_files_even_with_a_misleading_browser_mime() {
    assert_eq!(
        attachment_mime(&file("main.ts", "video/mp2t", b"export const x = 1\n")),
        Some("text/plain".to_string())
    );
}

#[test]
#[ignore = "porting: components/prompt-input/attachments not implemented"]
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
#[ignore = "porting: components/prompt-input/attachments not implemented"]
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
#[ignore = "porting: components/prompt-input/attachments not implemented"]
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
#[ignore = "porting: components/prompt-input/attachments not implemented"]
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
#[ignore = "porting: components/prompt-input/attachments not implemented"]
fn uses_native_paste_for_short_single_line_text() {
    assert_eq!(paste_mode("hello world"), "native");
}

#[test]
#[ignore = "porting: components/prompt-input/attachments not implemented"]
fn uses_manual_paste_for_multiline_text() {
    assert_eq!(paste_mode("{\n  \"ok\": true\n}"), "manual");
    assert_eq!(paste_mode("a\r\nb"), "manual");
}

#[test]
#[ignore = "porting: components/prompt-input/attachments not implemented"]
fn uses_manual_paste_for_large_text() {
    assert_eq!(paste_mode(&"x".repeat(8000)), "manual");
}
