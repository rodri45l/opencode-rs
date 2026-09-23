//! Server protocol detection (port of packages/app/src/utils/server-protocol.ts).

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Protocol {
    V1,
    V2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: Option<String>,
    pub pid: Option<i64>,
    pub status: u16,
}

pub fn detect_server_protocol(
    global_health: HealthResponse,
    api_health: HealthResponse,
) -> Protocol {
    if (200..300).contains(&global_health.status) && global_health.healthy {
        return Protocol::V1;
    }
    if (200..300).contains(&api_health.status) && api_health.pid.is_some() {
        return Protocol::V2;
    }
    if (200..300).contains(&api_health.status) && api_health.healthy {
        return Protocol::V1;
    }
    Protocol::V2
}
