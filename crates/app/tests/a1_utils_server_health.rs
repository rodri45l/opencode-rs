//! Port of packages/app/src/utils/server-health.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct Health {
    healthy: bool,
    version: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
enum FetchOutcome {
    Ok { status: u16, body: String },
    NetworkError,
}

// Local stub (fast wave): real module lands later.
fn check_server_health(_outcomes: &[FetchOutcome], _retry_count: usize) -> (Health, Vec<String>) {
    (
        Health {
            healthy: false,
            version: None,
        },
        Vec::new(),
    )
}

#[test]
#[ignore = "porting: utils/server-health not implemented"]
fn returns_healthy_response_with_version() {
    let outcomes = vec![FetchOutcome::Ok {
        status: 200,
        body: r#"{"healthy":true,"version":"1.2.3"}"#.into(),
    }];
    let (health, paths) = check_server_health(&outcomes, 0);
    assert_eq!(
        health,
        Health {
            healthy: true,
            version: Some("1.2.3".into())
        }
    );
    assert_eq!(paths, vec!["/api/health".to_string()]);
}

#[test]
#[ignore = "porting: utils/server-health not implemented"]
fn falls_back_to_the_v1_health_endpoint() {
    let outcomes = vec![
        FetchOutcome::Ok {
            status: 404,
            body: String::new(),
        },
        FetchOutcome::Ok {
            status: 200,
            body: r#"{"healthy":true,"version":"1.18.4"}"#.into(),
        },
    ];
    let (health, paths) = check_server_health(&outcomes, 0);
    assert_eq!(
        health,
        Health {
            healthy: true,
            version: Some("1.18.4".into())
        }
    );
    assert_eq!(
        paths,
        vec!["/api/health".to_string(), "/global/health".to_string()]
    );
}

#[test]
#[ignore = "porting: utils/server-health not implemented"]
fn falls_back_when_the_current_health_response_is_malformed() {
    let outcomes = vec![
        FetchOutcome::Ok {
            status: 200,
            body: "{}".into(),
        },
        FetchOutcome::Ok {
            status: 200,
            body: r#"{"healthy":true,"version":"1.18.4"}"#.into(),
        },
    ];
    let (health, paths) = check_server_health(&outcomes, 0);
    assert_eq!(
        health,
        Health {
            healthy: true,
            version: Some("1.18.4".into())
        }
    );
    assert_eq!(
        paths,
        vec!["/api/health".to_string(), "/global/health".to_string()]
    );
}

#[test]
#[ignore = "porting: utils/server-health not implemented"]
fn returns_unhealthy_when_request_fails() {
    let outcomes = vec![FetchOutcome::NetworkError, FetchOutcome::NetworkError];
    let (health, _paths) = check_server_health(&outcomes, 0);
    assert_eq!(
        health,
        Health {
            healthy: false,
            version: None
        }
    );
}

#[test]
#[ignore = "porting: utils/server-health not implemented"]
fn retries_transient_failures_and_eventually_succeeds() {
    let outcomes = vec![
        FetchOutcome::NetworkError,
        FetchOutcome::NetworkError,
        FetchOutcome::Ok {
            status: 200,
            body: r#"{"healthy":true,"version":"1.2.3"}"#.into(),
        },
    ];
    let (health, _paths) = check_server_health(&outcomes, 2);
    assert_eq!(
        health,
        Health {
            healthy: true,
            version: Some("1.2.3".into())
        }
    );
}

#[test]
#[ignore = "porting: utils/server-health not implemented"]
fn returns_unhealthy_when_retries_are_exhausted() {
    let outcomes = vec![
        FetchOutcome::NetworkError,
        FetchOutcome::NetworkError,
        FetchOutcome::NetworkError,
        FetchOutcome::NetworkError,
        FetchOutcome::NetworkError,
        FetchOutcome::NetworkError,
    ];
    let (health, _paths) = check_server_health(&outcomes, 2);
    assert_eq!(
        health,
        Health {
            healthy: false,
            version: None
        }
    );
}
