//! Port of packages/app/src/utils/server-health.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::server_health::{check_server_health, FetchOutcome, Health};

#[test]
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
