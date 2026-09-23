//! Server basic-auth configuration.
//!
//! Mirrors the reference `ServerAuth.Config`: an optional password plus a
//! username that defaults to `opencode`. The middleware that *enforces* this
//! config lives on the router (not yet ported); this type only captures the
//! observable decision — whether auth is required and whether a decoded
//! credential pair is accepted.

/// Basic-auth configuration for one server instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthConfig {
    /// The configured password, or `None` when auth is disabled.
    pub password: Option<String>,
    /// The accepted username (`opencode` by default).
    pub username: String,
}

impl AuthConfig {
    /// Auth disabled, username defaults to `opencode`.
    pub fn none() -> Self {
        Self {
            password: None,
            username: "opencode".to_string(),
        }
    }

    /// Auth enabled with the default username.
    pub fn with_password(password: impl Into<String>) -> Self {
        Self {
            password: Some(password.into()),
            username: "opencode".to_string(),
        }
    }

    /// Auth enabled with explicit credentials.
    pub fn with_credentials(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            password: Some(password.into()),
            username: username.into(),
        }
    }

    /// Whether a password is configured (and thus auth is required).
    pub fn required(&self) -> bool {
        self.password.is_some()
    }

    /// Whether a decoded credential pair is accepted.
    ///
    /// With no configured password every request is authorized. Otherwise both
    /// the username and the password must match exactly.
    pub fn authorized(&self, username: &str, password: &str) -> bool {
        match &self.password {
            None => true,
            Some(expected) => username == self.username && password == expected,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self::none()
    }
}
