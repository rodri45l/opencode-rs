//! The public event manifest.
//!
//! Mirrors `packages/schema/src/event-manifest.ts`: the registry composes every
//! module's definitions, selects the latest per type, and indexes durable
//! definitions by `type.version`.

use crate::definitions::{
    filesystem, integration, permission, project, reference, session_event, session_todo,
    workspace_event,
};
use crate::event::{define, Definition, EventRegistryError};
use crate::legacy_event;
use crate::permission_v1;
use crate::question_v1;
use crate::session_v1;
use std::collections::BTreeMap;

/// `packages/schema/src/catalog.ts`.
const CATALOG_UPDATED: Definition = define("catalog.updated", None);
/// `packages/schema/src/models-dev.ts`.
const MODELS_DEV_REFRESHED: Definition = define("models-dev.refreshed", None);
/// `packages/schema/src/plugin.ts`.
const PLUGIN_ADDED: Definition = define("plugin.added", None);
/// `packages/schema/src/project-directories.ts`.
const PROJECT_DIRECTORIES_UPDATED: Definition = define("project.directories.updated", None);
/// `packages/schema/src/filesystem-watcher.ts`.
const FILE_WATCHER_UPDATED: Definition = define("file.watcher.updated", None);
/// `packages/schema/src/pty.ts`.
const PTY_CREATED: Definition = define("pty.created", None);
const PTY_UPDATED: Definition = define("pty.updated", None);
const PTY_EXITED: Definition = define("pty.exited", None);
const PTY_DELETED: Definition = define("pty.deleted", None);
/// `packages/schema/src/question.ts`.
const QUESTION_V2_ASKED: Definition = define("question.v2.asked", None);
const QUESTION_V2_REPLIED: Definition = define("question.v2.replied", None);
const QUESTION_V2_REJECTED: Definition = define("question.v2.rejected", None);
/// `packages/schema/src/installation-event.ts`.
const INSTALLATION_UPDATED: Definition = define("installation.updated", None);
const INSTALLATION_UPDATE_AVAILABLE: Definition = define("installation.update-available", None);
/// `packages/schema/src/lsp-event.ts`.
const LSP_UPDATED: Definition = define("lsp.updated", None);
/// `packages/schema/src/tui-event.ts`.
const TUI_PROMPT_APPEND: Definition = define("tui.prompt.append", None);
const TUI_COMMAND_EXECUTE: Definition = define("tui.command.execute", None);
const TUI_TOAST_SHOW: Definition = define("tui.toast.show", None);
const TUI_SESSION_SELECT: Definition = define("tui.session.select", None);
/// `packages/schema/src/mcp-event.ts`.
const MCP_TOOLS_CHANGED: Definition = define("mcp.tools.changed", None);
const MCP_BROWSER_OPEN_FAILED: Definition = define("mcp.browser.open.failed", None);
/// `packages/schema/src/session-status-event.ts`.
const SESSION_STATUS: Definition = define("session.status", None);
const SESSION_IDLE: Definition = define("session.idle", None);
/// `packages/schema/src/session-compaction-event.ts`.
const SESSION_COMPACTED: Definition = define("session.compacted", None);
/// `packages/schema/src/vcs-event.ts`.
const VCS_BRANCH_UPDATED: Definition = define("vcs.branch.updated", None);
/// `packages/schema/src/worktree-event.ts`.
const WORKTREE_READY: Definition = define("worktree.ready", None);
const WORKTREE_FAILED: Definition = define("worktree.failed", None);
/// `packages/schema/src/server-event.ts`.
const SERVER_CONNECTED: Definition = define("server.connected", None);
const GLOBAL_DISPOSED: Definition = define("global.disposed", None);

fn session_v1_durable() -> Vec<&'static Definition> {
    session_v1::event::DEFINITIONS
        .iter()
        .filter(|definition| definition.durable.is_some())
        .collect()
}

fn session_v1_live() -> Vec<&'static Definition> {
    session_v1::event::DEFINITIONS
        .iter()
        .filter(|definition| definition.durable.is_none())
        .collect()
}

fn feature_definitions() -> Vec<&'static Definition> {
    let mut definitions: Vec<&'static Definition> = Vec::new();
    definitions.extend(filesystem::event::DEFINITIONS.iter());
    definitions.extend(reference::event::DEFINITIONS.iter());
    definitions.extend(permission::event::DEFINITIONS.iter());
    definitions.push(&PLUGIN_ADDED);
    definitions.push(&PROJECT_DIRECTORIES_UPDATED);
    definitions.push(&FILE_WATCHER_UPDATED);
    definitions.extend([&PTY_CREATED, &PTY_UPDATED, &PTY_EXITED, &PTY_DELETED]);
    definitions.extend([
        &QUESTION_V2_ASKED,
        &QUESTION_V2_REPLIED,
        &QUESTION_V2_REJECTED,
    ]);
    definitions
}

fn foundation_definitions() -> Vec<&'static Definition> {
    let mut definitions: Vec<&'static Definition> = Vec::new();
    definitions.push(&MODELS_DEV_REFRESHED);
    definitions.extend(integration::event::DEFINITIONS.iter());
    definitions.push(&CATALOG_UPDATED);
    definitions.extend(session_v1_durable());
    definitions.extend(session_event::DEFINITIONS.iter());
    definitions
}

/// The `ServerDefinitions` inventory (foundation + feature + todo).
pub fn server_definitions() -> Result<Vec<&'static Definition>, EventRegistryError> {
    let mut definitions = foundation_definitions();
    definitions.extend(feature_definitions());
    definitions.extend(session_todo::event::DEFINITIONS.iter());
    Ok(definitions)
}

/// The `Definitions` inventory.
pub fn definitions() -> Result<Vec<&'static Definition>, EventRegistryError> {
    let mut definitions = foundation_definitions();
    definitions.extend(session_v1_live());
    definitions.extend([&INSTALLATION_UPDATED, &INSTALLATION_UPDATE_AVAILABLE]);
    definitions.extend(feature_definitions());
    definitions.extend(session_todo::event::DEFINITIONS.iter());
    definitions.push(&LSP_UPDATED);
    definitions.extend(permission_v1::event::DEFINITIONS.iter());
    definitions.extend([
        &TUI_PROMPT_APPEND,
        &TUI_COMMAND_EXECUTE,
        &TUI_TOAST_SHOW,
        &TUI_SESSION_SELECT,
    ]);
    definitions.extend([&MCP_TOOLS_CHANGED, &MCP_BROWSER_OPEN_FAILED]);
    definitions.extend(legacy_event::DEFINITIONS.iter());
    definitions.extend(project::event::DEFINITIONS.iter());
    definitions.extend([&SESSION_STATUS, &SESSION_IDLE]);
    definitions.extend(question_v1::event::DEFINITIONS.iter());
    definitions.push(&SESSION_COMPACTED);
    definitions.push(&VCS_BRANCH_UPDATED);
    definitions.extend(workspace_event::DEFINITIONS.iter());
    definitions.extend([&WORKTREE_READY, &WORKTREE_FAILED]);
    definitions.extend([&SERVER_CONNECTED, &GLOBAL_DISPOSED]);
    Ok(definitions)
}

/// The `Latest` index (highest durable version per type).
pub fn latest() -> Result<BTreeMap<&'static str, &'static Definition>, EventRegistryError> {
    crate::event::latest(&definitions()?)
}

/// The `Durable` index, keyed by `type.version`.
pub fn durable() -> Result<BTreeMap<String, &'static Definition>, EventRegistryError> {
    crate::event::durable(&definitions()?)
}
