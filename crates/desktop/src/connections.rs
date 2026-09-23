//! WSL connection projection.
//!
//! Port of `packages/desktop/src/renderer/wsl/connections.ts` (upstream 18ef3cc):
//! a server is published only once its runtime is ready, and startup is not
//! blocked on a configured WSL default that is not ready.

/// The WSL runtime lifecycle kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeKind {
    Starting,
    Ready,
    Failed,
    Stopped,
}

/// A WSL runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Runtime {
    pub kind: RuntimeKind,
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// A configured WSL server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub id: String,
    pub distro: String,
}

/// A WSL server with its runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerState {
    pub config: ServerConfig,
    pub runtime: Runtime,
}

/// The WSL servers state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WslServersState {
    pub servers: Vec<ServerState>,
}

/// A published connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    pub display_name: String,
    pub label: String,
}

/// Project the ready WSL servers into connections.
pub fn ready_wsl_connections(state: &WslServersState, label: Option<&str>) -> Vec<Connection> {
    let label = label.unwrap_or("WSL");
    state
        .servers
        .iter()
        .filter(|server| server.runtime.kind == RuntimeKind::Ready)
        .map(|server| Connection {
            display_name: server.config.distro.clone(),
            label: label.to_string(),
        })
        .collect()
}

/// The startup server key: the configured WSL default when ready, else the sidecar.
pub fn available_startup_server(key: &str, state: Option<&WslServersState>) -> String {
    if !key.starts_with("wsl:") {
        return key.to_string();
    }
    let ready = state
        .map(|state| {
            state
                .servers
                .iter()
                .any(|server| server.config.id == key && server.runtime.kind == RuntimeKind::Ready)
        })
        .unwrap_or(false);
    if ready {
        key.to_string()
    } else {
        "sidecar".to_string()
    }
}
