//! Builtin LSP selection.
//!
//! Ports the observable behaviour of the builtin-server decision in
//! `packages/opencode/src/lsp/lsp.ts` and `server.ts`.

use serde_json::Value;

/// A typed LSP index error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspIndexError {
    /// The behaviour has not been ported yet.
    NotImplemented(&'static str),
}

impl std::fmt::Display for LspIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "not implemented: {what}"),
        }
    }
}

impl std::error::Error for LspIndexError {}

/// The builtin LSP id selected for `file`, if any.
pub fn builtin_server(
    file: &str,
    inside_instance: bool,
    lsp_config: &Value,
    experimental_ty: bool,
) -> Result<Option<String>, LspIndexError> {
    if !inside_instance {
        return Ok(None);
    }
    let enabled = lsp_config.as_bool() == Some(true) || lsp_config.is_object();
    if !enabled {
        return Ok(None);
    }
    let server = if file.ends_with(".ts") {
        Some("typescript")
    } else if file.ends_with(".py") {
        Some(if experimental_ty { "ty" } else { "pyright" })
    } else {
        None
    };
    Ok(server.map(str::to_string))
}

/// Whether any builtin client would be selected for `file`.
pub fn has_clients(
    file: &str,
    inside_instance: bool,
    lsp_config: &Value,
) -> Result<bool, LspIndexError> {
    Ok(builtin_server(file, inside_instance, lsp_config, false)?.is_some())
}
