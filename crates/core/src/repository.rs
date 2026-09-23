//! Repository reference parsing and cache paths.
//!
//! Ports the observable behaviour of `packages/core/src/repository.ts`: parse
//! GitHub shorthand, host/path, scp-style and file remotes, derive explicit-root
//! cache paths (with branch suffix encoding), validate branches, and compare
//! cache identity independent of spelling.

use crate::{CoreError, CoreResult};

/// A parsed repository reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// Host (`github.com`, `file`, ...).
    pub host: String,
    /// Path within the host (`owner/repo`).
    pub path: String,
    /// Path segments.
    pub segments: Vec<String>,
    /// Owner segment, when present.
    pub owner: Option<String>,
    /// Repository name.
    pub repo: String,
    /// Clone remote.
    pub remote: String,
    /// Display label.
    pub label: String,
    /// URL protocol, when parsed from a URL.
    pub protocol: Option<String>,
}

/// Repository reference helpers.
#[derive(Debug, Default)]
pub struct Repository;

impl Repository {
    /// Parse an explicit remote URL (`https://`, `ssh://`, `file:`, scp).
    pub fn parse(url: &str) -> CoreResult<Reference> {
        let trimmed = url.trim();
        if let Some(rest) = trimmed.strip_prefix("file:") {
            let path = rest.trim_start_matches("//").trim_start_matches('/');
            let path = path.to_string();
            let segments: Vec<String> = path
                .trim_end_matches(".git")
                .split('/')
                .filter(|segment| !segment.is_empty())
                .map(str::to_string)
                .collect();
            let repo = segments.last().cloned().unwrap_or_default();
            return Ok(Reference {
                host: "file".to_string(),
                path,
                segments,
                owner: None,
                repo: repo.clone(),
                remote: trimmed.to_string(),
                label: repo,
                protocol: Some("file:".to_string()),
            });
        }
        if let Some((scheme, rest)) = trimmed.split_once("://") {
            let (host, path) = rest
                .split_once('/')
                .ok_or_else(|| CoreError::Invalid(format!("invalid repository url: {url}")))?;
            let path = path.trim_end_matches(".git").to_string();
            let segments: Vec<String> = path
                .split('/')
                .filter(|segment| !segment.is_empty())
                .map(str::to_string)
                .collect();
            Self::validate_segments(&segments)?;
            let repo = segments.last().cloned().unwrap_or_default();
            let owner = if segments.len() >= 2 {
                Some(segments[segments.len() - 2].clone())
            } else {
                None
            };
            let label = if host == "github.com" {
                path.clone()
            } else {
                format!("{host}/{path}")
            };
            return Ok(Reference {
                host: host.to_string(),
                path,
                segments,
                owner,
                repo,
                remote: trimmed.to_string(),
                label,
                protocol: Some(format!("{scheme}:")),
            });
        }
        Self::parse_remote(trimmed)
    }

    /// Parse a remote shorthand (`owner/repo`, `host/group/repo`, scp).
    pub fn parse_remote(input: &str) -> CoreResult<Reference> {
        let trimmed = input.trim();
        if trimmed.is_empty() || trimmed.starts_with("file:") {
            return Err(CoreError::Invalid(format!(
                "not a remote repository reference: {input}"
            )));
        }
        if trimmed.contains("://") {
            return Self::parse(trimmed);
        }
        if let Some(rest) = trimmed.strip_prefix("git@") {
            let (host, path) = rest
                .split_once(':')
                .ok_or_else(|| CoreError::Invalid(format!("invalid scp reference: {input}")))?;
            let segments = split_segments(path)?;
            let repo = segments.last().cloned().unwrap_or_default();
            let owner = segments.get(segments.len().wrapping_sub(2)).cloned();
            let joined = segments.join("/");
            let label = if host == "github.com" {
                joined.clone()
            } else {
                format!("{host}/{joined}")
            };
            return Ok(Reference {
                host: host.to_string(),
                path: joined,
                segments,
                owner,
                repo,
                remote: trimmed.to_string(),
                label,
                protocol: None,
            });
        }
        let parts: Vec<&str> = trimmed.split('/').collect();
        if parts.len() < 2 {
            return Err(CoreError::Invalid(format!(
                "not a repository reference: {input}"
            )));
        }
        let looks_hosted = parts[0].contains('.');
        let (host, path_parts) = if looks_hosted {
            (parts[0].to_string(), parts[1..].to_vec())
        } else {
            ("github.com".to_string(), parts.clone())
        };
        if path_parts.len() < 2 {
            return Err(CoreError::Invalid(format!(
                "not a repository reference: {input}"
            )));
        }
        let segments = split_segments(&path_parts.join("/"))?;
        let repo = segments.last().cloned().unwrap_or_default();
        let owner = segments.get(segments.len().wrapping_sub(2)).cloned();
        let joined = segments.join("/");
        let label = if host == "github.com" {
            joined.clone()
        } else {
            format!("{host}/{joined}")
        };
        Ok(Reference {
            host: host.clone(),
            path: joined.clone(),
            segments,
            owner,
            repo,
            remote: format!("https://{host}/{joined}.git"),
            label,
            protocol: None,
        })
    }

    fn validate_segments(segments: &[String]) -> CoreResult<()> {
        if segments.is_empty() {
            return Err(CoreError::Invalid("empty repository path".into()));
        }
        for segment in segments {
            if segment.is_empty() || segment == "." || segment == ".." {
                return Err(CoreError::Invalid(format!(
                    "unsafe repository path segment: {segment}"
                )));
            }
        }
        Ok(())
    }

    /// Build an explicit-root cache path for a reference and optional branch.
    pub fn cache_path(
        root: &str,
        reference: &Reference,
        branch: Option<&str>,
    ) -> CoreResult<String> {
        let mut segments = vec![reference.host.clone()];
        segments.extend(reference.segments.iter().cloned());
        let last = segments.pop().unwrap_or_default();
        let leaf = match branch.filter(|branch| !branch.is_empty()) {
            Some(branch) => format!("{last}@{}", encode_branch(branch)),
            None => last,
        };
        segments.push(leaf);
        let mut path = std::path::Path::new(root).to_path_buf();
        for segment in segments {
            path.push(segment);
        }
        Ok(path.to_string_lossy().to_string())
    }

    /// The stable identity used to compare cache entries.
    pub fn cache_identity(reference: &Reference) -> CoreResult<String> {
        Ok(format!("{}/{}", reference.host, reference.path))
    }

    /// Whether the reference is a local file repository.
    pub fn is_file(reference: &Reference) -> CoreResult<bool> {
        Ok(reference.host == "file")
    }

    /// Whether the reference is a remote repository.
    pub fn is_remote(reference: &Reference) -> CoreResult<bool> {
        Ok(reference.host != "file")
    }

    /// Validate a branch name.
    pub fn validate_branch(branch: &str) -> CoreResult<()> {
        if branch.is_empty()
            || branch.starts_with('-')
            || branch.contains("..")
            || branch.contains(' ')
            || branch.contains('\n')
            || branch.contains('\t')
            || branch.starts_with('/')
            || branch.ends_with('/')
        {
            return Err(CoreError::Invalid(format!("invalid branch: {branch}")));
        }
        Ok(())
    }

    /// Whether two references share a cache identity.
    pub fn same(a: &Reference, b: &Reference) -> CoreResult<bool> {
        Ok(Self::cache_identity(a)? == Self::cache_identity(b)?)
    }
}

fn split_segments(path: &str) -> CoreResult<Vec<String>> {
    let path = path.trim_end_matches('/').trim_end_matches(".git");
    let segments: Vec<String> = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect();
    Repository::validate_segments(&segments)?;
    Ok(segments)
}

fn encode_branch(branch: &str) -> String {
    branch.replace('/', "%2F")
}
