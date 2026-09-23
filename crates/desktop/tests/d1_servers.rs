//! Port of packages/desktop/src/main/wsl/servers.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/wsl/{policy,startup,servers}.ts:
//! startup ids are the configured server ids; an update that does not install the
//! expected version is rejected; the matching distro server id is restarted after
//! an update; cached distro probes are cleared on removal; terminals open with a
//! `-d` distro argument (spaces preserved); IPC identifiers are validated at the
//! boundary; a missing WSL runtime requires a Windows restart; and the servers
//! controller drops stale background checks and probes addable distros in
//! parallel, skipping OpenCode checks for distros that cannot execute.
//! Re-derived: the async sidecar/`AbortController` polling loop is dropped; the
//! controller is expressed as a synchronous stub. `pollWslHealth` is not ported
//! (real timer/abort semantics).

#[allow(dead_code)]
mod wsl {
    use std::collections::BTreeMap;
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: desktop WSL servers not implemented";

    fn stub<T>() -> PortResult<T> {
        Err(NotImplemented(NOTE))
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct WslServerConfig {
        pub id: String,
        pub distro: String,
    }

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

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct WslServerItem {
        pub config: WslServerConfig,
        pub runtime: Runtime,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct WslDistroProbe {
        pub name: String,
        pub can_execute: bool,
        pub has_bash: bool,
        pub has_curl: bool,
        pub error: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct WslOpencodeCheck {
        pub distro: String,
        pub resolved_path: Option<String>,
        pub version: Option<String>,
        pub expected_version: String,
        pub matches_desktop: bool,
        pub error: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct WslServersState {
        pub servers: Vec<WslServerItem>,
        pub distro_probes: BTreeMap<String, WslDistroProbe>,
        pub opencode_checks: BTreeMap<String, WslOpencodeCheck>,
        pub pending_restart: bool,
    }

    pub fn wsl_server_ids_to_start_on_initialize(
        _servers: &[WslServerConfig],
    ) -> PortResult<Vec<String>> {
        stub()
    }

    pub fn expect_opencode_version(
        _installed: Option<&str>,
        _expected: &str,
        _distro: &str,
    ) -> PortResult<()> {
        stub()
    }

    pub fn wsl_server_id_to_restart(
        _servers: &[WslServerItem],
        _distro: &str,
    ) -> PortResult<Option<String>> {
        stub()
    }

    pub fn clear_wsl_distro_state(
        _distro_probes: &BTreeMap<String, WslDistroProbe>,
        _opencode_checks: &BTreeMap<String, WslOpencodeCheck>,
        _distro: &str,
    ) -> PortResult<(
        BTreeMap<String, WslDistroProbe>,
        BTreeMap<String, WslOpencodeCheck>,
    )> {
        stub()
    }

    pub fn wsl_terminal_args(_distro: Option<&str>) -> PortResult<Vec<String>> {
        stub()
    }

    pub fn require_wsl_ipc_string(_name: &str, _value: Option<&str>) -> PortResult<String> {
        stub()
    }

    pub fn require_wsl_ipc_strings(_name: &str, _values: &[String]) -> PortResult<Vec<String>> {
        stub()
    }

    pub fn pending_restart_after_wsl_install(_available: bool) -> PortResult<bool> {
        stub()
    }

    pub struct WslServersController;

    impl WslServersController {
        pub fn initialize(&mut self) -> PortResult<()> {
            stub()
        }

        pub fn add_server(&mut self, _distro: &str) -> PortResult<()> {
            stub()
        }

        pub fn remove_server(&mut self, _id: &str) -> PortResult<()> {
            stub()
        }

        pub fn probe_addable(&mut self, _distros: &[String]) -> PortResult<()> {
            stub()
        }

        pub fn get_state(&self) -> PortResult<WslServersState> {
            stub()
        }
    }

    pub fn create_wsl_servers_controller(
        _app_version: &str,
        _persisted: Vec<WslServerConfig>,
    ) -> WslServersController {
        WslServersController
    }
}

use wsl::{
    clear_wsl_distro_state, create_wsl_servers_controller, expect_opencode_version,
    pending_restart_after_wsl_install, require_wsl_ipc_string, require_wsl_ipc_strings,
    wsl_server_id_to_restart, wsl_server_ids_to_start_on_initialize, wsl_terminal_args, Runtime,
    WslDistroProbe, WslOpencodeCheck, WslServerConfig, WslServerItem, NOTE,
};

#[test]
#[ignore = "porting: desktop WSL servers not implemented"]
fn starts_every_configured_wsl_server_on_initialization() {
    let servers = vec![
        WslServerConfig {
            id: "wsl:Debian".to_string(),
            distro: "Debian".to_string(),
        },
        WslServerConfig {
            id: "wsl:Ubuntu-24.04".to_string(),
            distro: "Ubuntu-24.04".to_string(),
        },
    ];
    assert_eq!(
        wsl_server_ids_to_start_on_initialize(&servers).expect(NOTE),
        vec!["wsl:Debian".to_string(), "wsl:Ubuntu-24.04".to_string()]
    );
}

#[test]
#[ignore = "porting: desktop WSL servers not implemented"]
fn rejects_an_update_that_did_not_install_the_desktop_version() {
    assert!(expect_opencode_version(Some("1.16.2"), "1.16.2", "Debian").is_ok());

    let error = expect_opencode_version(Some("1.14.35"), "1.16.2", "Debian").expect_err(NOTE);
    assert!(error
        .to_string()
        .contains("OpenCode update finished but Debian still reports 1.14.35; expected 1.16.2"));
}

#[test]
#[ignore = "porting: desktop WSL servers not implemented"]
fn restarts_an_existing_distro_server_after_updating_opencode() {
    let servers = vec![WslServerItem {
        config: WslServerConfig {
            id: "wsl:Debian".to_string(),
            distro: "Debian".to_string(),
        },
        runtime: Runtime::Ready {
            url: String::new(),
            username: None,
            password: None,
        },
    }];

    assert_eq!(
        wsl_server_id_to_restart(&servers, "Debian").expect(NOTE),
        Some("wsl:Debian".to_string())
    );
    assert_eq!(wsl_server_id_to_restart(&[], "Debian").expect(NOTE), None);
}

#[test]
#[ignore = "porting: desktop WSL servers not implemented"]
fn clears_cached_distro_probes_when_removing_a_wsl_server() {
    let mut probes = std::collections::BTreeMap::new();
    probes.insert(
        "Debian".to_string(),
        WslDistroProbe {
            name: "Debian".to_string(),
            can_execute: true,
            has_bash: true,
            has_curl: true,
            error: None,
        },
    );
    let mut checks = std::collections::BTreeMap::new();
    checks.insert(
        "Debian".to_string(),
        WslOpencodeCheck {
            distro: "Debian".to_string(),
            resolved_path: Some("/home/luke/.opencode/bin/opencode".to_string()),
            version: Some("1.16.2".to_string()),
            expected_version: "1.16.2".to_string(),
            matches_desktop: true,
            error: None,
        },
    );

    let (next_probes, next_checks) =
        clear_wsl_distro_state(&probes, &checks, "Debian").expect(NOTE);
    assert!(next_probes.is_empty());
    assert!(next_checks.is_empty());
}

#[test]
#[ignore = "porting: desktop WSL servers not implemented"]
fn opens_terminals_for_distro_names_containing_spaces() {
    assert_eq!(
        wsl_terminal_args(Some("Ubuntu Preview")).expect(NOTE),
        vec![
            "/c".to_string(),
            "start".to_string(),
            String::new(),
            "wsl".to_string(),
            "-d".to_string(),
            "Ubuntu Preview".to_string()
        ]
    );
}

#[test]
#[ignore = "porting: desktop WSL servers not implemented"]
fn validates_wsl_ipc_identifiers_at_the_module_boundary() {
    assert_eq!(
        require_wsl_ipc_string("distro", Some("Debian")).expect(NOTE),
        "Debian"
    );
    assert_eq!(
        require_wsl_ipc_strings("distro", &["Debian".to_string(), "Ubuntu".to_string()])
            .expect(NOTE),
        vec!["Debian".to_string(), "Ubuntu".to_string()]
    );
    assert!(require_wsl_ipc_string("distro", Some("")).is_err());
    assert!(require_wsl_ipc_string("server id", None).is_err());
    assert!(require_wsl_ipc_strings("distro", &[]).is_err());
}

#[test]
#[ignore = "porting: desktop WSL servers not implemented"]
fn derives_a_required_windows_restart_from_the_post_install_runtime_probe() {
    assert!(pending_restart_after_wsl_install(false).expect(NOTE));
    assert!(!pending_restart_after_wsl_install(true).expect(NOTE));
}

#[test]
#[ignore = "porting: desktop WSL servers not implemented"]
fn ignores_stale_background_opencode_checks_after_removing_a_wsl_server() {
    let mut controller = create_wsl_servers_controller("1.16.2", Vec::new());
    controller.add_server("Debian").expect(NOTE);
    controller.remove_server("wsl:Debian").expect(NOTE);

    let state = controller.get_state().expect(NOTE);
    assert!(state.servers.is_empty());
    assert!(state.opencode_checks.is_empty());
}

#[test]
#[ignore = "porting: desktop WSL servers not implemented"]
fn ignores_stale_startup_opencode_checks_after_removing_a_wsl_server() {
    let persisted = vec![WslServerConfig {
        id: "wsl:Debian".to_string(),
        distro: "Debian".to_string(),
    }];
    let mut controller = create_wsl_servers_controller("1.16.2", persisted);
    controller.initialize().expect(NOTE);
    controller.remove_server("wsl:Debian").expect(NOTE);

    let state = controller.get_state().expect(NOTE);
    assert!(state.servers.is_empty());
    assert!(state.opencode_checks.is_empty());
}

#[test]
#[ignore = "porting: desktop WSL servers not implemented"]
fn probes_addable_distros_in_parallel_before_checking_opencode() {
    let mut controller = create_wsl_servers_controller("1.16.2", Vec::new());
    controller
        .probe_addable(&["Debian".to_string(), "Ubuntu".to_string()])
        .expect(NOTE);

    let state = controller.get_state().expect(NOTE);
    assert_eq!(
        state.distro_probes.keys().cloned().collect::<Vec<_>>(),
        vec!["Debian".to_string(), "Ubuntu".to_string()]
    );
    assert_eq!(
        state.opencode_checks.keys().cloned().collect::<Vec<_>>(),
        vec!["Debian".to_string(), "Ubuntu".to_string()]
    );
}

#[test]
#[ignore = "porting: desktop WSL servers not implemented"]
fn does_not_check_opencode_in_addable_distros_that_cannot_execute_commands() {
    let mut controller = create_wsl_servers_controller("1.16.2", Vec::new());
    controller
        .probe_addable(&["Debian".to_string(), "Ubuntu".to_string()])
        .expect(NOTE);

    let state = controller.get_state().expect(NOTE);
    assert_eq!(
        state.distro_probes.keys().cloned().collect::<Vec<_>>(),
        vec!["Debian".to_string(), "Ubuntu".to_string()]
    );
    assert_eq!(
        state.opencode_checks.keys().cloned().collect::<Vec<_>>(),
        vec!["Debian".to_string()]
    );
}
