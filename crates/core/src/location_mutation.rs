//! Location-scoped mutation path resolution.
//!
//! Ports the observable behaviour of `packages/core/src/location-mutation.ts`:
//! resolve relative, prospective, absolute and external targets into canonical
//! paths and resources, reject lexical escapes, and require external-directory
//! authorization for explicit external absolute targets.

use serde_json::Value;

use crate::path::AbsolutePath;
use crate::{CoreError, CoreResult};

/// An external directory that requires authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalDirectory {
    /// Stable existing directory.
    pub directory: AbsolutePath,
    /// Resource glob under that directory.
    pub resource: String,
}

/// A resolved mutation target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationTarget {
    /// Canonical target path.
    pub canonical: AbsolutePath,
    /// Resource name relative to the location (or the path for externals).
    pub resource: String,
    /// External-directory authorization, when required.
    pub external_directory: Option<ExternalDirectory>,
}

/// The input accepted by [`LocationMutation::resolve`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveInput {
    /// The requested path.
    pub path: String,
    /// Optional mutation kind.
    pub kind: Option<String>,
}

impl ResolveInput {
    /// Decode a resolve input, ignoring unknown fields.
    pub fn decode(value: &Value) -> CoreResult<Self> {
        let path = value
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| CoreError::Invalid("resolve input missing path".into()))?;
        let kind = value
            .get("kind")
            .and_then(Value::as_str)
            .map(str::to_string);
        Ok(Self {
            path: path.to_string(),
            kind,
        })
    }
}

/// Location-scoped path resolution.
#[derive(Debug, Default)]
pub struct LocationMutation;

impl LocationMutation {
    /// Resolve `input` against `directory`.
    pub fn resolve(directory: &AbsolutePath, input: &ResolveInput) -> CoreResult<MutationTarget> {
        let base = lexical(directory.as_path());
        let requested = std::path::Path::new(&input.path);
        if requested.is_absolute() {
            let canonical = lexical(requested);
            if canonical.starts_with(&base) {
                let resource = relative_resource(&base, &canonical);
                return Ok(MutationTarget {
                    canonical: AbsolutePath::new(canonical),
                    resource,
                    external_directory: None,
                });
            }
            let root = canonical
                .parent()
                .map(lexical)
                .unwrap_or_else(|| canonical.clone());
            let root_string = root.to_string_lossy().replace('\\', "/");
            let resource = format!(
                "{root_string}/{}",
                canonical.file_name().unwrap_or_default().to_string_lossy()
            );
            return Ok(MutationTarget {
                canonical: AbsolutePath::new(canonical),
                resource,
                external_directory: Some(ExternalDirectory {
                    directory: AbsolutePath::new(root.clone()),
                    resource: format!("{root_string}/*"),
                }),
            });
        }

        let joined = lexical(&base.join(requested));
        if !joined.starts_with(&base) {
            return Err(CoreError::Invalid(format!(
                "path escapes the active location: {}",
                input.path
            )));
        }
        let resource = relative_resource(&base, &joined);
        Ok(MutationTarget {
            canonical: AbsolutePath::new(joined),
            resource,
            external_directory: None,
        })
    }
}

fn lexical(path: &std::path::Path) -> std::path::PathBuf {
    let mut normalized = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn relative_resource(base: &std::path::Path, canonical: &std::path::Path) -> String {
    canonical
        .strip_prefix(base)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| canonical.to_string_lossy().replace('\\', "/"))
}
