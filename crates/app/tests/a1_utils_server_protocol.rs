//! Port of packages/app/src/utils/server-protocol.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
enum Protocol {
    V1,
    V2,
}

#[derive(Clone, Debug, PartialEq)]
struct HealthResponse {
    healthy: bool,
    version: Option<String>,
    pid: Option<i64>,
    status: u16,
}

// Local stub (fast wave): real module lands later.
fn detect_server_protocol(_global_health: HealthResponse, _api_health: HealthResponse) -> Protocol {
    Protocol::V1
}

#[test]
#[ignore = "porting: utils/server-protocol not implemented"]
fn prefers_the_legacy_health_endpoint_when_both_api_generations_exist() {
    let global = HealthResponse {
        healthy: true,
        version: Some("1.18.4".into()),
        pid: None,
        status: 200,
    };
    let api = HealthResponse {
        healthy: true,
        version: Some("2.0.0".into()),
        pid: Some(123),
        status: 200,
    };
    assert_eq!(detect_server_protocol(global, api), Protocol::V1);
}

#[test]
#[ignore = "porting: utils/server-protocol not implemented"]
fn recognizes_v2_health_by_its_process_identifier() {
    let global = HealthResponse {
        healthy: false,
        version: None,
        pid: None,
        status: 404,
    };
    let api = HealthResponse {
        healthy: true,
        version: Some("2.0.0".into()),
        pid: Some(123),
        status: 200,
    };
    assert_eq!(detect_server_protocol(global, api), Protocol::V2);
}

#[test]
#[ignore = "porting: utils/server-protocol not implemented"]
fn recognizes_the_transitional_v1_api_health_response() {
    let global = HealthResponse {
        healthy: false,
        version: None,
        pid: None,
        status: 404,
    };
    let api = HealthResponse {
        healthy: true,
        version: None,
        pid: None,
        status: 200,
    };
    assert_eq!(detect_server_protocol(global, api), Protocol::V1);
}
