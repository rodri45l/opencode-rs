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
    pub fn sanitize(_spec: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented("npm::Npm::sanitize"))
    }

    /// Materialize a package and resolve its entrypoint.
    pub fn add(_spec: &str) -> CoreResult<NpmEntry> {
        Err(CoreError::NotImplemented("npm::Npm::add"))
    }

    /// Install a project's dependencies.
    pub fn install(_directory: &str) -> CoreResult<()> {
        Err(CoreError::NotImplemented("npm::Npm::install"))
    }
}
