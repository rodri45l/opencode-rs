//! Port of packages/core/test/oauth-page.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the OAuth callback page escapes bootstrap options embedded
//! in its inline script so attacker-controlled `provider`/`tokenPath` values
//! cannot close the script tag; exactly one real `</script>` tag remains.

use opencode_core::oauth_page::OauthCallbackPage;

const NOTE: &str = "porting: oauth callback page not implemented";

#[test]
fn escapes_bootstrap_options_embedded_in_the_inline_script() {
    let html = OauthCallbackPage::bootstrap(
        "xAI</script><script>alert(\"provider\")</script>",
        "/token</script><script>alert(\"path\")</script>",
    )
    .expect(NOTE);

    assert_eq!(html.matches("</script>").count(), 1);
    assert!(html.contains("xAI\\u003c/script>\\u003cscript>alert(\\\"provider\\\")\\u003c/script>"));
    assert!(html.contains("/token\\u003c/script>\\u003cscript>alert(\\\"path\\\")\\u003c/script>"));
}
