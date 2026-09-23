//! Port of packages/desktop/src/main/external-url.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/external-url.ts: only web
//! (`http`/`https`) and `mailto` links are opened externally, `file:` and
//! unsupported protocols are rejected, and local file URLs must resolve to a
//! local path.

#[allow(dead_code)]
mod external_url {
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: desktop external-url policy not implemented";

    fn stub<T>() -> PortResult<T> {
        Err(NotImplemented(NOTE))
    }

    pub fn resolve_external_url(_input: &str) -> PortResult<Option<String>> {
        stub()
    }

    pub fn resolve_local_file_path(_input: &str) -> PortResult<Option<String>> {
        stub()
    }
}

use external_url::{resolve_external_url, resolve_local_file_path, NOTE};

#[test]
#[ignore = "porting: desktop external-url policy not implemented"]
fn opens_web_urls_externally() {
    assert_eq!(
        resolve_external_url("https://example.com/a?b=c").expect(NOTE),
        Some("https://example.com/a?b=c".to_string())
    );
    assert_eq!(
        resolve_external_url("http://example.com").expect(NOTE),
        Some("http://example.com/".to_string())
    );
}

#[test]
#[ignore = "porting: desktop external-url policy not implemented"]
fn opens_mail_links_externally() {
    assert_eq!(
        resolve_external_url("mailto:hello@opencode.ai").expect(NOTE),
        Some("mailto:hello@opencode.ai".to_string())
    );
}

#[test]
#[ignore = "porting: desktop external-url policy not implemented"]
fn rejects_file_urls_and_unsupported_protocols() {
    assert_eq!(
        resolve_external_url("file:///tmp/index.html").expect(NOTE),
        None
    );
    assert_eq!(
        resolve_external_url("javascript:alert(1)").expect(NOTE),
        None
    );
    assert_eq!(
        resolve_external_url("data:text/html,hello").expect(NOTE),
        None
    );
    assert_eq!(resolve_external_url("not a url").expect(NOTE), None);
}

#[test]
#[ignore = "porting: desktop external-url policy not implemented"]
fn resolves_only_local_file_urls() {
    let path = std::env::current_dir().expect("cwd").join("example.html");
    let url = format!("file://{}", path.display());
    assert_eq!(
        resolve_local_file_path(&url).expect(NOTE).as_deref(),
        path.to_str()
    );
    assert_eq!(
        resolve_local_file_path("file://example.com/share/index.html").expect(NOTE),
        None
    );
    assert_eq!(
        resolve_local_file_path("https://example.com/index.html").expect(NOTE),
        None
    );
}
