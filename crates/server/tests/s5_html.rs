//! Port of packages/opencode/test/util/html.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `escapeHtml` escapes `&`, `<`, `>`, `"` and `'` and
//! leaves safe text untouched.
#![allow(dead_code)]

// Fast-wave local stubs: `util::html` is not implemented in this crate yet.
mod html {
    pub fn escape_html(_input: &str) -> Result<String, &'static str> {
        Err("porting: html::escapeHtml not implemented")
    }
}

#[test]
#[ignore = "porting: html not implemented"]
fn escapes_html_metacharacters() {
    assert_eq!(
        html::escape_html("</div><script>alert(1)</script><div class=\"x\">").unwrap(),
        "&lt;/div&gt;&lt;script&gt;alert(1)&lt;/script&gt;&lt;div class=&quot;x&quot;&gt;"
    );
    assert_eq!(html::escape_html("a & b").unwrap(), "a &amp; b");
    assert_eq!(html::escape_html("it's fine").unwrap(), "it&#39;s fine");
    assert_eq!(html::escape_html("invalid_grant").unwrap(), "invalid_grant");
    assert_eq!(html::escape_html("").unwrap(), "");
    assert_eq!(html::escape_html("&<").unwrap(), "&amp;&lt;");
}
