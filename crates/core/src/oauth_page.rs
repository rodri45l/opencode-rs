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
    pub fn bootstrap(_provider: &str, _token_path: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "oauth_page::OauthCallbackPage::bootstrap",
        ))
    }

    /// Escape a value for safe embedding inside an inline script.
    pub fn escape(_value: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "oauth_page::OauthCallbackPage::escape",
        ))
    }
}
