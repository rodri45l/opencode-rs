//! Repository reference parsing and cache identity.
//!
//! Ports the observable behaviour of `packages/opencode/src/util/repository.ts`:
//! GitHub shorthand, `host/path`, scp and URL remote references, local `file:`
//! references, typed parse errors, and branch-name validation.

use regex::Regex;
use url::Url;

/// A remote git repository reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteReference {
    /// Lower-cased host.
    pub host: String,
    /// Normalised `owner/repo` path.
    pub path: String,
    /// Path segments.
    pub segments: Vec<String>,
    /// Owner when the path has exactly two segments.
    pub owner: String,
    /// Final path segment.
    pub repo: String,
    /// Clone URL.
    pub remote: String,
    /// Human-readable label.
    pub label: String,
}

/// A local `file:` repository reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalReference {
    /// Always `file`.
    pub host: String,
    /// Always `file:`.
    pub protocol: String,
    /// The filesystem path.
    pub label: String,
}

/// Either a remote or a local repository reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    /// A remote repository.
    Remote(RemoteReference),
    /// A local file repository.
    Local(LocalReference),
}

/// A typed repository error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    /// The input is not a valid repository reference.
    InvalidReference,
    /// Local file repositories are not supported.
    UnsupportedLocal,
    /// The branch name is invalid.
    InvalidBranch(String),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidReference => write!(f, "invalid repository reference"),
            Self::UnsupportedLocal => write!(f, "Local file repositories are not supported"),
            Self::InvalidBranch(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for RepositoryError {}

fn normalize(input: &str) -> String {
    let trimmed = input.trim();
    let without_git_plus = trimmed.strip_prefix("git+").unwrap_or(trimmed);
    let without_fragment = without_git_plus
        .split_once('#')
        .map(|(head, _)| head)
        .unwrap_or(without_git_plus);
    without_fragment.trim_end_matches('/').to_string()
}

fn trim_git_suffix(input: &str) -> String {
    input.strip_suffix(".git").unwrap_or(input).to_string()
}

fn parts(input: &str) -> Vec<String> {
    input
        .split('/')
        .map(|item| trim_git_suffix(item.trim()))
        .filter(|item| !item.is_empty())
        .collect()
}

fn safe_host(input: &str) -> bool {
    !input.is_empty() && !input.starts_with('-') && !input.contains([' ', '/', '\\'])
}

fn safe_segment(input: &str) -> bool {
    input != "." && input != ".." && !input.contains(':') && !input.contains([' ', '/', '\\'])
}

fn host_like(input: &str) -> bool {
    input.contains('.') || input.contains(':') || input == "localhost"
}

fn github_remote(pathname: &str) -> String {
    match std::env::var("OPENCODE_REPO_CLONE_GITHUB_BASE_URL") {
        Ok(base) if !base.is_empty() => {
            let base = base.trim_end_matches('/');
            format!("{base}/{pathname}.git")
        }
        _ => format!("https://github.com/{pathname}.git"),
    }
}

fn build_remote(
    host: &str,
    segments: &[String],
    remote: Option<String>,
) -> Option<RemoteReference> {
    let segments: Vec<String> = segments
        .iter()
        .map(|segment| trim_git_suffix(segment))
        .filter(|segment| !segment.is_empty())
        .collect();
    if !safe_host(host) || segments.is_empty() || segments.iter().any(|s| !safe_segment(s)) {
        return None;
    }
    let pathname = segments.join("/");
    let repo = segments[segments.len() - 1].clone();
    let host = host.to_lowercase();
    let owner = if segments.len() == 2 {
        segments[0].clone()
    } else {
        String::new()
    };
    let remote = remote.unwrap_or_else(|| {
        if host == "github.com" {
            github_remote(&pathname)
        } else {
            format!("https://{host}/{pathname}.git")
        }
    });
    let label = if host == "github.com" && segments.len() == 2 {
        pathname.clone()
    } else {
        format!("{host}/{pathname}")
    };
    Some(RemoteReference {
        host,
        path: pathname,
        segments,
        owner,
        repo,
        remote,
        label,
    })
}

fn parse_scp(input: &str) -> Option<(String, String)> {
    let re = Regex::new(r"^(?:[^@/\s]+@)?([^:/\s]+):(.+)$").ok()?;
    let captures = re.captures(input)?;
    Some((
        captures.get(1)?.as_str().to_string(),
        captures.get(2)?.as_str().to_string(),
    ))
}

/// Parse any repository reference (remote or local).
pub fn parse_reference(input: &str) -> Result<Reference, RepositoryError> {
    let cleaned = normalize(input);
    if cleaned.is_empty() {
        return Err(RepositoryError::InvalidReference);
    }

    if let Some(rest) = cleaned.strip_prefix("github:") {
        let direct = parts(rest);
        if direct.len() == 2 {
            return build_remote("github.com", &direct, None)
                .map(Reference::Remote)
                .ok_or(RepositoryError::InvalidReference);
        }
    }

    if !cleaned.contains("://") {
        if let Some((host, path)) = parse_scp(&cleaned) {
            return build_remote(&host, &parts(&path), Some(cleaned))
                .map(Reference::Remote)
                .ok_or(RepositoryError::InvalidReference);
        }
        let direct = parts(&cleaned);
        if direct.len() >= 2 && host_like(&direct[0]) {
            return build_remote(&direct[0], &direct[1..], None)
                .map(Reference::Remote)
                .ok_or(RepositoryError::InvalidReference);
        }
        if direct.len() == 2 {
            return build_remote("github.com", &direct, None)
                .map(Reference::Remote)
                .ok_or(RepositoryError::InvalidReference);
        }
    }

    if let Ok(url) = Url::parse(&cleaned) {
        if url.scheme() == "file" {
            return Ok(Reference::Local(LocalReference {
                host: "file".to_string(),
                protocol: "file:".to_string(),
                label: url.path().to_string(),
            }));
        }
        let pathname = parts(url.path());
        let host = url.host_str().unwrap_or_default().to_string();
        let remote = if host == "github.com" {
            github_remote(&pathname.join("/"))
        } else {
            cleaned.clone()
        };
        return build_remote(&host, &pathname, Some(remote))
            .map(Reference::Remote)
            .ok_or(RepositoryError::InvalidReference);
    }

    Err(RepositoryError::InvalidReference)
}

/// Parse a remote reference, rejecting local `file:` references.
pub fn parse_remote(input: &str) -> Result<RemoteReference, RepositoryError> {
    match parse_reference(input)? {
        Reference::Remote(reference) => Ok(reference),
        Reference::Local(_) => Err(RepositoryError::UnsupportedLocal),
    }
}

/// Whether the reference points at a local file repository.
pub fn is_file_reference(reference: &Reference) -> bool {
    matches!(reference, Reference::Local(_))
}

/// Whether the reference points at a remote repository.
pub fn is_remote_reference(reference: &Reference) -> bool {
    matches!(reference, Reference::Remote(_))
}

/// The cache directory for a reference.
pub fn cache_path(reference: &RemoteReference) -> String {
    format!(
        "repos/{}/{}",
        reference.host.replace(':', "/"),
        reference.path
    )
}

/// The cache identity independent of input spelling.
pub fn cache_identity(reference: &RemoteReference) -> String {
    format!("{}/{}", reference.host, reference.path)
}

/// Whether two references resolve to the same cache identity.
pub fn same_reference(left: &RemoteReference, right: &RemoteReference) -> bool {
    cache_identity(left) == cache_identity(right)
}

/// Validate a git branch name.
pub fn validate_branch(branch: &str) -> Result<(), RepositoryError> {
    let re = Regex::new(r"^[A-Za-z0-9/_.-]+$").expect("valid branch pattern");
    if !re.is_match(branch) || branch.starts_with('-') || branch.contains("..") {
        return Err(RepositoryError::InvalidBranch(
            "Branch must contain only alphanumeric characters, /, _, ., and -, and cannot start with - or contain .."
                .to_string(),
        ));
    }
    Ok(())
}
