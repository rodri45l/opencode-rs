//! Port of packages/desktop/src/main/external-url.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/external-url.ts: only web
//! (`http`/`https`) and `mailto` links are opened externally, `file:` and
//! unsupported protocols are rejected, and local file URLs must resolve to a
//! local path.

use opencode_desktop::external_url::{resolve_external_url, resolve_local_file_path};

#[test]
fn opens_web_urls_externally() {
    assert_eq!(
        resolve_external_url("https://example.com/a?b=c"),
        Some("https://example.com/a?b=c".to_string())
    );
    assert_eq!(
        resolve_external_url("http://example.com"),
        Some("http://example.com/".to_string())
    );
}

#[test]
fn opens_mail_links_externally() {
    assert_eq!(
        resolve_external_url("mailto:hello@opencode.ai"),
        Some("mailto:hello@opencode.ai".to_string())
    );
}

#[test]
fn rejects_file_urls_and_unsupported_protocols() {
    assert_eq!(resolve_external_url("file:///tmp/index.html"), None);
    assert_eq!(resolve_external_url("javascript:alert(1)"), None);
    assert_eq!(resolve_external_url("data:text/html,hello"), None);
    assert_eq!(resolve_external_url("not a url"), None);
}

#[test]
fn resolves_only_local_file_urls() {
    let path = std::env::current_dir().expect("cwd").join("example.html");
    let url = format!("file://{}", path.display());
    assert_eq!(resolve_local_file_path(&url).as_deref(), path.to_str());
    assert_eq!(
        resolve_local_file_path("file://example.com/share/index.html"),
        None
    );
    assert_eq!(
        resolve_local_file_path("https://example.com/index.html"),
        None
    );
}
