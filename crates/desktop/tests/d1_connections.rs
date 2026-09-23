//! Port of packages/desktop/src/renderer/wsl/connections.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/renderer/wsl/connections.ts: a WSL
//! server is published as a connection only once its runtime reports ready, the
//! connection label uses the renderer translation, and desktop startup is not
//! blocked on a configured WSL default that is not ready.

#[allow(dead_code)]
mod connections {
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

    pub const NOTE: &str = "porting: desktop WSL connections not implemented";

    fn stub<T>() -> PortResult<T> {
        Err(NotImplemented(NOTE))
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum RuntimeKind {
        Starting,
        Ready,
        Failed,
        Stopped,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Runtime {
        pub kind: RuntimeKind,
        pub url: Option<String>,
        pub username: Option<String>,
        pub password: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ServerConfig {
        pub id: String,
        pub distro: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ServerState {
        pub config: ServerConfig,
        pub runtime: Runtime,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct WslServersState {
        pub servers: Vec<ServerState>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Connection {
        pub display_name: String,
        pub label: String,
    }

    pub fn ready_wsl_connections(
        _state: &WslServersState,
        _label: Option<&str>,
    ) -> PortResult<Vec<Connection>> {
        stub()
    }

    pub fn available_startup_server(
        _key: &str,
        _state: Option<&WslServersState>,
    ) -> PortResult<String> {
        stub()
    }
}

use connections::{
    available_startup_server, ready_wsl_connections, Connection, Runtime, RuntimeKind,
    ServerConfig, ServerState, WslServersState, NOTE,
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
#[ignore = "porting: desktop WSL connections not implemented"]
fn publishes_a_wsl_server_only_after_it_reports_ready() {
    assert_eq!(
        ready_wsl_connections(&state(RuntimeKind::Starting), None).expect(NOTE),
        Vec::<Connection>::new()
    );
    assert_eq!(
        ready_wsl_connections(&state(RuntimeKind::Failed), None).expect(NOTE),
        Vec::<Connection>::new()
    );
    assert_eq!(
        ready_wsl_connections(&state(RuntimeKind::Stopped), None).expect(NOTE),
        Vec::<Connection>::new()
    );
    assert_eq!(
        ready_wsl_connections(&state(RuntimeKind::Ready), None).expect(NOTE),
        vec![Connection {
            display_name: "Debian".to_string(),
            label: "WSL".to_string(),
        }]
    );
}

#[test]
#[ignore = "porting: desktop WSL connections not implemented"]
fn uses_the_renderer_translation_for_the_wsl_connection_label() {
    let connections =
        ready_wsl_connections(&state(RuntimeKind::Ready), Some("Translated WSL")).expect(NOTE);
    assert_eq!(connections[0].label, "Translated WSL");
}

#[test]
#[ignore = "porting: desktop WSL connections not implemented"]
fn does_not_block_desktop_startup_on_a_configured_wsl_default() {
    let key = "wsl:Debian";
    assert_eq!(available_startup_server(key, None).expect(NOTE), "sidecar");
    assert_eq!(
        available_startup_server(key, Some(&state(RuntimeKind::Starting))).expect(NOTE),
        "sidecar"
    );
    assert_eq!(
        available_startup_server(key, Some(&state(RuntimeKind::Ready))).expect(NOTE),
        key
    );
}
