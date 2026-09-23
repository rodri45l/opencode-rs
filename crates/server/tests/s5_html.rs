//! Port of packages/opencode/test/util/html.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `escapeHtml` escapes `&`, `<`, `>`, `"` and `'` and
//! leaves safe text untouched.
#![allow(dead_code)]

use opencode_server::html_util::escape_html;

#[test]
fn escapes_html_metacharacters() {
    assert_eq!(
        escape_html("</div><script>alert(1)</script><div class=\"x\">"),
        "&lt;/div&gt;&lt;script&gt;alert(1)&lt;/script&gt;&lt;div class=&quot;x&quot;&gt;"
    );
    assert_eq!(escape_html("a & b"), "a &amp; b");
    assert_eq!(escape_html("it's fine"), "it&#39;s fine");
    assert_eq!(escape_html("invalid_grant"), "invalid_grant");
    assert_eq!(escape_html(""), "");
    assert_eq!(escape_html("&<"), "&amp;&lt;");
}
