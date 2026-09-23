//! Port of packages/desktop/src/main/attachment-picker.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/attachment-picker.ts: the media
//! ingest limit is 20 MB, a selection is rejected before any file is read, and
//! picked-file authorizations are scoped per renderer and per picker token.
//! Re-derived: the async `ReadableStream`/`ArrayBuffer` plumbing is expressed as
//! owned byte buffers.

#[allow(dead_code)]
mod attachment_picker {
    use std::fmt;
    use std::path::Path;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: desktop attachment picker not implemented";

    pub const MAX_ATTACHMENT_BYTES: u64 = 20 * 1024 * 1024;

    fn stub<T>() -> PortResult<T> {
        Err(NotImplemented(NOTE))
    }

    pub fn assert_attachment_budget(_sizes: &[u64]) -> PortResult<()> {
        stub()
    }

    pub fn read_attachment(_path: &Path) -> PortResult<Vec<u8>> {
        stub()
    }

    pub struct PickerAuthorizations;

    impl PickerAuthorizations {
        pub fn add(&mut self, _renderer: u64, _paths: &[&str]) -> u64 {
            0
        }

        pub fn read(&self, _renderer: u64, _token: u64, _path: &str) -> PortResult<Vec<u8>> {
            stub()
        }

        pub fn release(&mut self, _renderer: u64, _token: u64) {}
    }

    pub fn create_picked_file_authorizations<F>(_read: F, _budget: u64) -> PickerAuthorizations {
        PickerAuthorizations
    }
}

use attachment_picker::{
    assert_attachment_budget, create_picked_file_authorizations, read_attachment,
    MAX_ATTACHMENT_BYTES, NOTE,
};

#[test]
#[ignore = "porting: desktop attachment picker not implemented"]
fn accepts_selections_within_the_media_ingest_limit() {
    assert!(
        assert_attachment_budget(&[MAX_ATTACHMENT_BYTES / 2, MAX_ATTACHMENT_BYTES / 2]).is_ok()
    );
}

#[test]
#[ignore = "porting: desktop attachment picker not implemented"]
fn rejects_the_selection_before_files_are_read_when_its_total_exceeds_the_limit() {
    let error = assert_attachment_budget(&[MAX_ATTACHMENT_BYTES, 1]).expect_err(NOTE);
    assert!(error.0.contains("20 MB limit"));
}

#[test]
#[ignore = "porting: desktop attachment picker not implemented"]
fn reads_an_approved_file_through_a_bounded_buffer() {
    let directory =
        std::env::temp_dir().join(format!("opencode-attachment-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("temp dir");
    let file = directory.join("example.txt");
    std::fs::write(&file, "lorem ipsum").expect("write file");

    let read = read_attachment(&file).expect(NOTE);
    assert_eq!(read, b"lorem ipsum".to_vec());

    std::fs::remove_dir_all(&directory).ok();
}

#[test]
#[ignore = "porting: desktop attachment picker not implemented"]
fn rejects_an_oversized_file_before_allocating_its_contents() {
    let directory =
        std::env::temp_dir().join(format!("opencode-attachment-over-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("temp dir");
    let file = directory.join("oversized.txt");
    std::fs::write(&file, b"").expect("write file");
    let handle = std::fs::OpenOptions::new()
        .write(true)
        .open(&file)
        .expect("open file");
    handle.set_len(MAX_ATTACHMENT_BYTES + 1).expect("truncate");
    drop(handle);

    let error = read_attachment(&file).expect_err(NOTE);
    assert_eq!(error.to_string(), "20 MB limit");

    std::fs::remove_dir_all(&directory).ok();
}

#[test]
#[ignore = "porting: desktop attachment picker not implemented"]
fn keeps_concurrent_picker_selections_isolated() {
    let mut authorizations = create_picked_file_authorizations(|_path: &str, _max: u64| 0u64, 0);
    let first = authorizations.add(1, &["a.txt", "b.txt"]);
    let second = authorizations.add(1, &["c.txt"]);

    assert_eq!(
        authorizations.read(1, first, "a.txt").expect(NOTE),
        b"a.txt".to_vec()
    );
    assert_eq!(
        authorizations.read(1, second, "c.txt").expect(NOTE),
        b"c.txt".to_vec()
    );
    assert_eq!(
        authorizations.read(1, first, "b.txt").expect(NOTE),
        b"b.txt".to_vec()
    );
}

#[test]
#[ignore = "porting: desktop attachment picker not implemented"]
fn releases_unread_files_for_one_picker_without_affecting_another() {
    let mut authorizations = create_picked_file_authorizations(|_path: &str, _max: u64| 0u64, 0);
    let first = authorizations.add(1, &["a.txt"]);
    let second = authorizations.add(1, &["b.txt"]);
    authorizations.release(1, first);

    let released = authorizations.read(1, first, "a.txt").expect_err(NOTE);
    assert!(released.to_string().contains("not selected"));
    assert_eq!(
        authorizations.read(1, second, "b.txt").expect(NOTE),
        b"b.txt".to_vec()
    );
}

#[test]
#[ignore = "porting: desktop attachment picker not implemented"]
fn keeps_picker_tokens_scoped_to_their_renderer() {
    let mut authorizations = create_picked_file_authorizations(|_path: &str, _max: u64| 0u64, 0);
    let token = authorizations.add(1, &["a.txt"]);

    let scoped = authorizations.read(2, token, "a.txt").expect_err(NOTE);
    assert!(scoped.to_string().contains("not selected"));
}

#[test]
#[ignore = "porting: desktop attachment picker not implemented"]
fn charges_actual_reads_against_the_selection_budget() {
    let mut authorizations = create_picked_file_authorizations(
        |_path: &str, max_bytes: u64| {
            if 6 > max_bytes {
                return 0u64;
            }
            6
        },
        10,
    );
    let token = authorizations.add(1, &["a.txt", "b.txt"]);

    authorizations.read(1, token, "a.txt").expect(NOTE);
    let exhausted = authorizations.read(1, token, "b.txt").expect_err(NOTE);
    assert!(exhausted.to_string().contains("budget exceeded"));
}
