//! OAuth callback page (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/oauth/page.ts`: the
//! callback page embeds bootstrap options in an inline script and escapes `<`
//! and `"` so user-controlled values cannot break out of the script tag. Only
//! the single real closing `</script>` tag remains in the document.

use crate::{CoreError, CoreResult};

/// The OAuth callback page builder.
#[derive(Debug, Default)]
pub struct OauthCallbackPage;

impl OauthCallbackPage {
    /// Build the callback HTML with `provider` and `token_path` embedded in the
    /// bootstrap script.
    pub fn bootstrap(provider: &str, token_path: &str) -> CoreResult<String> {
        let script = format!(
            "var PROVIDER={};\nvar TOKEN_URL=new URL({},window.location.origin).href;",
            Self::escape(provider)?,
            Self::escape(token_path)?
        );
        Ok(format!(
            "<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\" />\n    <meta name=\"robots\" content=\"noindex\" />\n    <title>Finishing sign-in · OpenCode</title>\n  </head>\n  <body>\n    <main class=\"card\" id=\"oc-card\" data-status=\"pending\">\n      <h1 class=\"headline\" id=\"oc-headline\">Finishing sign-in</h1>\n      <p class=\"message\" id=\"oc-message\">Completing authorization.</p>\n      <pre class=\"detail\" id=\"oc-detail\" hidden></pre>\n      <p class=\"footnote\" id=\"oc-footnote\">You can close this window once sign-in finishes.</p>\n    </main>\n    <script>{script}</script>\n  </body>\n</html>"
        ))
    }

    /// Escape a value for safe embedding inside an inline script.
    pub fn escape(value: &str) -> CoreResult<String> {
        let json = serde_json::to_string(value).map_err(|error| {
            CoreError::Invalid(format!("failed to encode script value: {error}"))
        })?;
        Ok(json.replace('<', "\\u003c"))
    }
}
