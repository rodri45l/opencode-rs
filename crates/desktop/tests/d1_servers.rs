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
//! controller is expressed as a synchronous state machine. `pollWslHealth` is not
//! ported (real timer/abort semantics).

use std::collections::BTreeMap;

use opencode_desktop::wsl_servers::{
    clear_wsl_distro_state, create_wsl_servers_controller,
    create_wsl_servers_controller_with_probe, expect_opencode_version,
    pending_restart_after_wsl_install, require_wsl_ipc_string, require_wsl_ipc_strings,
    wsl_server_id_to_restart, wsl_server_ids_to_start_on_initialize, wsl_terminal_args, Runtime,
    WslDistroProbe, WslOpencodeCheck, WslServerConfig, WslServerItem,
};

#[test]
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
        wsl_server_ids_to_start_on_initialize(&servers),
        vec!["wsl:Debian".to_string(), "wsl:Ubuntu-24.04".to_string()]
    );
}

#[test]
fn rejects_an_update_that_did_not_install_the_desktop_version() {
    assert!(expect_opencode_version(Some("1.16.2"), "1.16.2", "Debian").is_ok());

    let error = expect_opencode_version(Some("1.14.35"), "1.16.2", "Debian").expect_err("update");
    assert!(error
        .to_string()
        .contains("OpenCode update finished but Debian still reports 1.14.35; expected 1.16.2"));
}

#[test]
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
        wsl_server_id_to_restart(&servers, "Debian"),
        Some("wsl:Debian".to_string())
    );
    assert_eq!(wsl_server_id_to_restart(&[], "Debian"), None);
}

#[test]
fn clears_cached_distro_probes_when_removing_a_wsl_server() {
    let mut probes = BTreeMap::new();
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
    let mut checks = BTreeMap::new();
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

    let (next_probes, next_checks) = clear_wsl_distro_state(&probes, &checks, "Debian");
    assert!(next_probes.is_empty());
    assert!(next_checks.is_empty());
}

#[test]
fn opens_terminals_for_distro_names_containing_spaces() {
    assert_eq!(
        wsl_terminal_args(Some("Ubuntu Preview")),
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
fn validates_wsl_ipc_identifiers_at_the_module_boundary() {
    assert_eq!(
        require_wsl_ipc_string("distro", Some("Debian")).expect("valid"),
        "Debian"
    );
    assert_eq!(
        require_wsl_ipc_strings("distro", &["Debian".to_string(), "Ubuntu".to_string()])
            .expect("valid"),
        vec!["Debian".to_string(), "Ubuntu".to_string()]
    );
    assert!(require_wsl_ipc_string("distro", Some("")).is_err());
    assert!(require_wsl_ipc_string("server id", None).is_err());
    assert!(require_wsl_ipc_strings("distro", &[]).is_err());
}

#[test]
fn derives_a_required_windows_restart_from_the_post_install_runtime_probe() {
    assert!(pending_restart_after_wsl_install(false));
    assert!(!pending_restart_after_wsl_install(true));
}

#[test]
fn ignores_stale_background_opencode_checks_after_removing_a_wsl_server() {
    let mut controller = create_wsl_servers_controller("1.16.2", Vec::new());
    controller.add_server("Debian").expect("add");
    controller.remove_server("wsl:Debian").expect("remove");

    let state = controller.get_state();
    assert!(state.servers.is_empty());
    assert!(state.opencode_checks.is_empty());
}

#[test]
fn ignores_stale_startup_opencode_checks_after_removing_a_wsl_server() {
    let persisted = vec![WslServerConfig {
        id: "wsl:Debian".to_string(),
        distro: "Debian".to_string(),
    }];
    let mut controller = create_wsl_servers_controller("1.16.2", persisted);
    controller.initialize().expect("initialize");
    controller.remove_server("wsl:Debian").expect("remove");

    let state = controller.get_state();
    assert!(state.servers.is_empty());
    assert!(state.opencode_checks.is_empty());
}

#[test]
fn probes_addable_distros_in_parallel_before_checking_opencode() {
    let mut controller = create_wsl_servers_controller("1.16.2", Vec::new());
    controller
        .probe_addable(&["Debian".to_string(), "Ubuntu".to_string()])
        .expect("probe");

    let state = controller.get_state();
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
fn does_not_check_opencode_in_addable_distros_that_cannot_execute_commands() {
    let mut controller =
        create_wsl_servers_controller_with_probe("1.16.2", Vec::new(), |distro| distro == "Debian");
    controller
        .probe_addable(&["Debian".to_string(), "Ubuntu".to_string()])
        .expect("probe");

    let state = controller.get_state();
    assert_eq!(
        state.distro_probes.keys().cloned().collect::<Vec<_>>(),
        vec!["Debian".to_string(), "Ubuntu".to_string()]
    );
    assert_eq!(
        state.opencode_checks.keys().cloned().collect::<Vec<_>>(),
        vec!["Debian".to_string()]
    );
}
