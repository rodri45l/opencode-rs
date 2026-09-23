//! Renderer initialization gating.
//!
//! Port of `packages/desktop/src/renderer/initialization.ts` (upstream 18ef3cc):
//! an initialization error is surfaced before rendering server providers,
//! Electron's remote-invocation wrapper is stripped, and a pending initialization
//! is awaited without reading its data.

/// A renderer initialization failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitError {
    pub message: String,
    pub local_server_startup: bool,
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for InitError {}

impl InitError {
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn local_server_startup(&self) -> bool {
        self.local_server_startup
    }
}

/// An initialization invocation.
pub struct Invocation<D> {
    pub error: Option<String>,
    pub loading: bool,
    pub produce: Option<Box<dyn FnOnce() -> D>>,
}

/// Return the initialized data, or the (unwrapped) initialization error.
pub fn initialization_data<D>(invocation: Invocation<D>) -> Result<D, InitError> {
    if let Some(error) = invocation.error {
        return Err(InitError {
            message: strip_invocation_wrapper(&error),
            local_server_startup: true,
        });
    }
    Ok((invocation.produce.expect("initialization produce"))())
}

/// Whether initialization has completed, without reading pending data.
pub fn initialization_ready<D>(invocation: Invocation<D>) -> Result<bool, InitError> {
    if invocation.loading {
        return Ok(false);
    }
    initialization_data(invocation)?;
    Ok(true)
}

fn strip_invocation_wrapper(message: &str) -> String {
    const PREFIX: &str = "Error invoking remote method 'await-initialization': Error: ";
    message.strip_prefix(PREFIX).unwrap_or(message).to_string()
}
