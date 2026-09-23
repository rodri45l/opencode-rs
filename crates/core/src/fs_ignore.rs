//! Default ignore matching.
//!
//! Ports the observable behaviour of `packages/core/src/filesystem/ignore.ts`:
//! the default ignore set matches `node_modules` at any depth, with or without
//! a trailing slash and with nested descendants.

use crate::fs_util::FSUtil;
use crate::CoreResult;

const FOLDERS: &[&str] = &[
    "node_modules",
    "bower_components",
    ".pnpm-store",
    "vendor",
    ".npm",
    "dist",
    "build",
    "out",
    ".next",
    "target",
    "bin",
    "obj",
    ".git",
    ".svn",
    ".hg",
    ".vscode",
    ".idea",
    ".turbo",
    ".output",
    "desktop",
    ".sst",
    ".cache",
    ".webkit-cache",
    "__pycache__",
    ".pytest_cache",
    "mypy_cache",
    ".history",
    ".gradle",
];

const FILES: &[&str] = &[
    "**/*.swp",
    "**/*.swo",
    "**/*.pyc",
    "**/.DS_Store",
    "**/Thumbs.db",
    "**/logs/**",
    "**/tmp/**",
    "**/temp/**",
    "**/*.log",
    "**/coverage/**",
    "**/.nyc_output/**",
];

/// Default ignore matching.
#[derive(Debug, Default)]
pub struct Ignore;

impl Ignore {
    /// Whether `path` matches the default ignore set.
    pub fn match_path(path: &str) -> CoreResult<bool> {
        for part in path.split(['/', '\\']) {
            if FOLDERS.contains(&part) {
                return Ok(true);
            }
        }
        for pattern in FILES {
            if FSUtil::glob_match(pattern, path)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The default ignore patterns (files first, then folders).
    pub fn patterns() -> CoreResult<Vec<String>> {
        let mut patterns: Vec<String> = FILES.iter().map(|value| (*value).to_string()).collect();
        patterns.extend(FOLDERS.iter().map(|value| (*value).to_string()));
        Ok(patterns)
    }
}
