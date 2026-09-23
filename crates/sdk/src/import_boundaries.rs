//! Import-boundary membership predicate.
//!
//! Re-derived from `packages/sdk-next/test/import-boundaries.test.ts`
//! (upstream 18ef3cc): the `bun build` invocation and its metafile are not
//! portable, so only the pure membership predicate is modelled.

/// The message used by the ported test when expecting success.
pub const NOTE: &str = "sdk-next import boundaries";

/// Return the inputs that live at or below `directory`.
pub fn within_directory(inputs: &[String], directory: &str) -> Vec<String> {
    let prefix = format!("{}/", directory.trim_end_matches('/'));
    inputs
        .iter()
        .filter(|input| {
            let value = input.as_str();
            value == directory || value.starts_with(&prefix)
        })
        .cloned()
        .collect()
}
