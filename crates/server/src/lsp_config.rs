//! LSP config validation.
//!
//! Re-derived from `packages/core/src/v1/config/lsp.ts` (upstream 18ef3cc). The
//! reference refinement requires custom (non-builtin) LSP servers to declare an
//! `extensions` array; builtin ids and explicitly disabled entries are exempt.

use serde_json::Value;

/// Builtin server ids, aligned with the LSP runtime. Custom servers must
/// declare `extensions` because the runtime cannot infer them.
pub const BUILTIN_SERVER_IDS: &[&str] = &[
    "deno",
    "typescript",
    "vue",
    "eslint",
    "oxlint",
    "biome",
    "gopls",
    "ruby-lsp",
    "ty",
    "pyright",
    "elixir-ls",
    "zls",
    "csharp",
    "razor",
    "fsharp",
    "sourcekit-lsp",
    "rust",
    "clangd",
    "svelte",
    "astro",
    "jdtls",
    "kotlin-ls",
    "yaml-ls",
    "lua-ls",
    "php intelephense",
    "prisma",
    "dart",
    "ocaml-lsp",
    "bash",
    "terraform",
    "texlab",
    "dockerfile",
    "gleam",
    "clojure-lsp",
    "nixd",
    "tinymist",
    "haskell-language-server",
    "julials",
];

/// The refinement error message.
pub const MISSING_EXTENSIONS: &str = "For custom LSP servers, 'extensions' array is required.";

/// Validate the decoded `lsp` config value.
///
/// A top-level boolean toggles LSP and always passes. Otherwise the value is a
/// map of server id to entry: disabled entries and builtin ids pass without
/// `extensions`; every other entry must carry an `extensions` key (an empty
/// array counts, mirroring the reference's truthy check).
pub fn validate_lsp_config(value: &Value) -> Result<(), &'static str> {
    match value {
        Value::Bool(_) => Ok(()),
        Value::Object(servers) => {
            for (id, entry) in servers {
                let Some(entry) = entry.as_object() else {
                    return Err(MISSING_EXTENSIONS);
                };
                if entry.get("disabled").and_then(Value::as_bool) == Some(true) {
                    continue;
                }
                if BUILTIN_SERVER_IDS.contains(&id.as_str()) {
                    continue;
                }
                if entry.contains_key("extensions") {
                    continue;
                }
                return Err(MISSING_EXTENSIONS);
            }
            Ok(())
        }
        _ => Err(MISSING_EXTENSIONS),
    }
}
