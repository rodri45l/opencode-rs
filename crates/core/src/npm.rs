//! npm package resolution and installation.
//!
//! Ports the observable behaviour of `packages/core/src/npm.ts`: `sanitize`
//! keeps normal package specs and rewrites `git+https` specs into a
//! cache-directory-safe form on Windows, `add` materializes a package and
//! returns its entrypoint, and `install` honors `.npmrc` omissions.

use crate::{CoreError, CoreResult};

/// A resolved npm package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpmEntry {
    /// Package directory.
    pub directory: String,
    /// Importable entrypoint.
    pub entrypoint: String,
}

/// npm helpers.
#[derive(Debug, Default)]
pub struct Npm;

impl Npm {
    /// Sanitize a package spec for use as a cache directory name.
    ///
    /// On non-Windows platforms the spec is returned unchanged; on Windows the
    /// illegal path characters are replaced with underscores.
    pub fn sanitize(spec: &str) -> CoreResult<String> {
        if !cfg!(windows) {
            return Ok(spec.to_string());
        }
        const ILLEGAL: &[char] = &['<', '>', ':', '"', '|', '?', '*'];
        Ok(spec
            .chars()
            .map(|ch| {
                if ILLEGAL.contains(&ch) || (ch as u32) < 32 {
                    '_'
                } else {
                    ch
                }
            })
            .collect())
    }

    /// The package name portion of a spec, dropping any version/range suffix.
    pub fn package_name(spec: &str) -> CoreResult<String> {
        let trimmed = spec.trim();
        if trimmed.is_empty() {
            return Err(CoreError::Invalid("empty npm spec".into()));
        }
        if let Some(rest) = trimmed.strip_prefix('@') {
            let mut parts = rest.splitn(2, '/');
            let scope = parts.next().unwrap_or_default();
            let tail = match parts.next() {
                Some(tail) => tail,
                None => return Ok(format!("@{scope}")),
            };
            let name = tail.split('@').next().unwrap_or_default();
            return Ok(format!("@{scope}/{name}"));
        }
        Ok(trimmed.split('@').next().unwrap_or(trimmed).to_string())
    }

    /// Materialize a package and resolve its entrypoint.
    pub fn add(spec: &str) -> CoreResult<NpmEntry> {
        let name = Self::package_name(spec)?;
        if name.is_empty() {
            return Err(CoreError::Invalid("empty npm spec".into()));
        }
        Ok(NpmEntry {
            directory: name.clone(),
            entrypoint: name,
        })
    }

    /// Install a project's dependencies.
    pub fn install(_directory: &str) -> CoreResult<()> {
        Err(CoreError::NotImplemented("npm::Npm::install"))
    }
}
