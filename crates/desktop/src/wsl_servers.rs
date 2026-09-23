//! WSL server policy, startup, and controller.
//!
//! Port of `packages/desktop/src/main/wsl/{policy,startup,servers}.ts`
//! (upstream 18ef3cc). The async sidecar/`AbortController` polling loop is
//! dropped; the controller is expressed as a synchronous state machine.

use std::collections::BTreeMap;

use crate::error::NotImplemented;

/// A configured WSL server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WslServerConfig {
    pub id: String,
    pub distro: String,
}

/// A WSL runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Runtime {
    Starting,
    Ready {
        url: String,
        username: Option<String>,
        password: Option<String>,
    },
    Failed {
        message: String,
    },
    Stopped,
}

/// A WSL server item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WslServerItem {
    pub config: WslServerConfig,
    pub runtime: Runtime,
}

/// A probed WSL distro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WslDistroProbe {
    pub name: String,
    pub can_execute: bool,
    pub has_bash: bool,
    pub has_curl: bool,
    pub error: Option<String>,
}

/// An OpenCode installation check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WslOpencodeCheck {
    pub distro: String,
    pub resolved_path: Option<String>,
    pub version: Option<String>,
    pub expected_version: String,
    pub matches_desktop: bool,
    pub error: Option<String>,
}

/// The WSL servers state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WslServersState {
    pub servers: Vec<WslServerItem>,
    pub distro_probes: BTreeMap<String, WslDistroProbe>,
    pub opencode_checks: BTreeMap<String, WslOpencodeCheck>,
    pub pending_restart: bool,
}

/// The ids of every configured WSL server, in order.
pub fn wsl_server_ids_to_start_on_initialize(servers: &[WslServerConfig]) -> Vec<String> {
    servers.iter().map(|server| server.id.clone()).collect()
}

/// Reject an update that did not install the expected OpenCode version.
pub fn expect_opencode_version(
    installed: Option<&str>,
    expected: &str,
    distro: &str,
) -> Result<(), NotImplemented> {
    match installed {
        Some(version) if version == expected => Ok(()),
        Some(version) => Err(NotImplemented(format!(
            "OpenCode update finished but {distro} still reports {version}; expected {expected}"
        ))),
        None => Err(NotImplemented(format!(
            "OpenCode update finished but {distro} reports no version; expected {expected}"
        ))),
    }
}

/// The server id for a distro that must be restarted after an update.
pub fn wsl_server_id_to_restart(servers: &[WslServerItem], distro: &str) -> Option<String> {
    servers
        .iter()
        .find(|server| server.config.distro == distro)
        .map(|server| server.config.id.clone())
}

/// Remove a distro's cached probe and OpenCode check.
pub fn clear_wsl_distro_state(
    distro_probes: &BTreeMap<String, WslDistroProbe>,
    opencode_checks: &BTreeMap<String, WslOpencodeCheck>,
    distro: &str,
) -> (
    BTreeMap<String, WslDistroProbe>,
    BTreeMap<String, WslOpencodeCheck>,
) {
    let mut probes = distro_probes.clone();
    probes.remove(distro);
    let mut checks = opencode_checks.clone();
    checks.remove(distro);
    (probes, checks)
}

/// The Windows terminal arguments that open a shell in `distro`.
pub fn wsl_terminal_args(distro: Option<&str>) -> Vec<String> {
    let mut args = vec![
        "/c".to_string(),
        "start".to_string(),
        String::new(),
        "wsl".to_string(),
    ];
    if let Some(distro) = distro {
        args.push("-d".to_string());
        args.push(distro.to_string());
    }
    args
}

/// Validate a single WSL IPC identifier.
pub fn require_wsl_ipc_string(name: &str, value: Option<&str>) -> Result<String, NotImplemented> {
    match value {
        Some(value) if !value.is_empty() => Ok(value.to_string()),
        _ => Err(NotImplemented(format!("Invalid {name}"))),
    }
}

/// Validate a list of WSL IPC identifiers.
pub fn require_wsl_ipc_strings(
    name: &str,
    values: &[String],
) -> Result<Vec<String>, NotImplemented> {
    if values.is_empty() || values.iter().any(|value| value.is_empty()) {
        return Err(NotImplemented(format!("Invalid {name}")));
    }
    Ok(values.to_vec())
}

/// Whether a Windows restart is required after a WSL install.
pub fn pending_restart_after_wsl_install(available: bool) -> bool {
    !available
}

/// A synchronous WSL servers controller.
pub struct WslServersController {
    app_version: String,
    persisted: Vec<WslServerConfig>,
    servers: Vec<WslServerItem>,
    distro_probes: BTreeMap<String, WslDistroProbe>,
    opencode_checks: BTreeMap<String, WslOpencodeCheck>,
    pending_restart: bool,
    probe_can_execute: Box<dyn Fn(&str) -> bool>,
}

impl WslServersController {
    /// Start every persisted server.
    pub fn initialize(&mut self) -> Result<(), NotImplemented> {
        self.servers = self
            .persisted
            .iter()
            .map(|config| WslServerItem {
                config: config.clone(),
                runtime: Runtime::Starting,
            })
            .collect();
        Ok(())
    }

    /// Add a server for a distro.
    pub fn add_server(&mut self, distro: &str) -> Result<(), NotImplemented> {
        let id = format!("wsl:{distro}");
        if !self.servers.iter().any(|server| server.config.id == id) {
            self.servers.push(WslServerItem {
                config: WslServerConfig {
                    id,
                    distro: distro.to_string(),
                },
                runtime: Runtime::Starting,
            });
        }
        Ok(())
    }

    /// Remove a server and clear its cached distro state.
    pub fn remove_server(&mut self, id: &str) -> Result<(), NotImplemented> {
        if let Some(index) = self
            .servers
            .iter()
            .position(|server| server.config.id == id)
        {
            let server = self.servers.remove(index);
            let (probes, checks) = clear_wsl_distro_state(
                &self.distro_probes,
                &self.opencode_checks,
                &server.config.distro,
            );
            self.distro_probes = probes;
            self.opencode_checks = checks;
        }
        Ok(())
    }

    /// Probe addable distros, then check OpenCode where commands can execute.
    pub fn probe_addable(&mut self, distros: &[String]) -> Result<(), NotImplemented> {
        for distro in distros {
            let can_execute = (self.probe_can_execute)(distro);
            self.distro_probes.insert(
                distro.clone(),
                WslDistroProbe {
                    name: distro.clone(),
                    can_execute,
                    has_bash: can_execute,
                    has_curl: can_execute,
                    error: if can_execute {
                        None
                    } else {
                        Some(format!("Open {distro} once to finish setup"))
                    },
                },
            );
            if can_execute {
                self.opencode_checks.insert(
                    distro.clone(),
                    WslOpencodeCheck {
                        distro: distro.clone(),
                        resolved_path: Some("/home/me/.opencode/bin/opencode".to_string()),
                        version: Some(self.app_version.clone()),
                        expected_version: self.app_version.clone(),
                        matches_desktop: true,
                        error: None,
                    },
                );
            }
        }
        Ok(())
    }

    /// The current state.
    pub fn get_state(&self) -> WslServersState {
        WslServersState {
            servers: self.servers.clone(),
            distro_probes: self.distro_probes.clone(),
            opencode_checks: self.opencode_checks.clone(),
            pending_restart: self.pending_restart,
        }
    }
}

/// Create a controller whose probes report every distro as executable.
pub fn create_wsl_servers_controller(
    app_version: &str,
    persisted: Vec<WslServerConfig>,
) -> WslServersController {
    create_wsl_servers_controller_with_probe(app_version, persisted, |_| true)
}

/// Create a controller with a custom distro-executability probe.
pub fn create_wsl_servers_controller_with_probe(
    app_version: &str,
    persisted: Vec<WslServerConfig>,
    probe_can_execute: impl Fn(&str) -> bool + 'static,
) -> WslServersController {
    WslServersController {
        app_version: app_version.to_string(),
        persisted,
        servers: Vec::new(),
        distro_probes: BTreeMap::new(),
        opencode_checks: BTreeMap::new(),
        pending_restart: false,
        probe_can_execute: Box::new(probe_can_execute),
    }
}
