//! Stable identifiers for reusable public schemas.
//!
//! Ports the `ast.annotations.identifier` invariant checked by
//! `packages/schema/test/contract-hygiene.test.ts`: every reusable public schema
//! exposes a non-empty, unique identifier.

/// Identifiers of the reusable public schemas.
pub const REUSABLE: &[&str] = &[
    "Agent.Color",
    "FileSystem.Submatch",
    "Model.Ref",
    "Model.Capabilities",
    "Model.Cost",
    "Model.Api",
    "Project.Icon",
    "Project.Commands",
    "Project.Time",
    "Project.Info",
    "Pty.Info",
    "Session.ListAnchor",
];
