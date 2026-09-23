//! Server health probing (port of packages/app/src/utils/server-health.ts).
//!
//! The reference performs the two-step V2-then-V1 probe against a live fetch.
//! Here the scripted fetch outcomes are injected so the probe logic is
//! exercised offline.

#[derive(Clone, Debug, PartialEq)]
pub struct Health {
    pub healthy: bool,
    pub version: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FetchOutcome {
    Ok { status: u16, body: String },
    NetworkError,
}

fn parse_health(outcome: &FetchOutcome) -> Option<Health> {
    match outcome {
        FetchOutcome::Ok { status, body } if (200..300).contains(status) => {
            let value: serde_json::Value = serde_json::from_str(body).ok()?;
            let healthy = value.get("healthy")?.as_bool()?;
            let version = value
                .get("version")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string());
            Some(Health { healthy, version })
        }
        _ => None,
    }
}

/// Run the V2 health probe, falling back to the V1 endpoint, retrying transient
/// failures up to `retry_count` times. Returns the resolved health plus the
/// request paths attempted, in order.
pub fn check_server_health(outcomes: &[FetchOutcome], retry_count: usize) -> (Health, Vec<String>) {
    let mut index = 0usize;
    let mut paths = Vec::new();
    let mut attempt = 0usize;

    let next = |index: &mut usize| -> FetchOutcome {
        let outcome = outcomes
            .get(*index)
            .cloned()
            .unwrap_or(FetchOutcome::NetworkError);
        *index += 1;
        outcome
    };

    loop {
        paths.push("/api/health".to_string());
        let current = next(&mut index);
        if let Some(health) = parse_health(&current) {
            return (health, paths);
        }

        paths.push("/global/health".to_string());
        let fallback = next(&mut index);
        if let Some(health) = parse_health(&fallback) {
            return (health, paths);
        }

        if attempt >= retry_count {
            return (
                Health {
                    healthy: false,
                    version: None,
                },
                paths,
            );
        }
        attempt += 1;
    }
}
