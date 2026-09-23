//! Port of packages/desktop/src/renderer/wsl/connections.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/renderer/wsl/connections.ts: a WSL
//! server is published as a connection only once its runtime reports ready, the
//! connection label uses the renderer translation, and desktop startup is not
//! blocked on a configured WSL default that is not ready.

use opencode_desktop::connections::{
    available_startup_server, ready_wsl_connections, Connection, Runtime, RuntimeKind,
    ServerConfig, ServerState, WslServersState,
};

fn state(kind: RuntimeKind) -> WslServersState {
    WslServersState {
        servers: vec![ServerState {
            config: ServerConfig {
                id: "wsl:Debian".to_string(),
                distro: "Debian".to_string(),
            },
            runtime: Runtime {
                kind,
                url: None,
                username: None,
                password: None,
            },
        }],
    }
}

#[test]
fn publishes_a_wsl_server_only_after_it_reports_ready() {
    assert_eq!(
        ready_wsl_connections(&state(RuntimeKind::Starting), None),
        Vec::<Connection>::new()
    );
    assert_eq!(
        ready_wsl_connections(&state(RuntimeKind::Failed), None),
        Vec::<Connection>::new()
    );
    assert_eq!(
        ready_wsl_connections(&state(RuntimeKind::Stopped), None),
        Vec::<Connection>::new()
    );
    assert_eq!(
        ready_wsl_connections(&state(RuntimeKind::Ready), None),
        vec![Connection {
            display_name: "Debian".to_string(),
            label: "WSL".to_string(),
        }]
    );
}

#[test]
fn uses_the_renderer_translation_for_the_wsl_connection_label() {
    let connections = ready_wsl_connections(&state(RuntimeKind::Ready), Some("Translated WSL"));
    assert_eq!(connections[0].label, "Translated WSL");
}

#[test]
fn does_not_block_desktop_startup_on_a_configured_wsl_default() {
    let key = "wsl:Debian";
    assert_eq!(available_startup_server(key, None), "sidecar");
    assert_eq!(
        available_startup_server(key, Some(&state(RuntimeKind::Starting))),
        "sidecar"
    );
    assert_eq!(
        available_startup_server(key, Some(&state(RuntimeKind::Ready))),
        key
    );
}
