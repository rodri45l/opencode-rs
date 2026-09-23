//! Port of packages/stats/core/src/r2-sql.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/stats/core/src/r2-sql.ts; see docs/TEST-PORT.md.

use opencode_stats::r2_sql::{query_r2_sql_pages, R2SqlData, R2SqlQueryError};
use std::collections::BTreeMap;

#[derive(Debug)]
struct TimeoutError;

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TimeoutError: The operation timed out.")
    }
}

impl std::error::Error for TimeoutError {}

fn row(pairs: &[(&str, &str)]) -> R2SqlData {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect::<BTreeMap<_, _>>()
}

#[test]
fn includes_the_transport_failure_in_the_message_used_by_sync_logging() {
    let cause: Box<dyn std::error::Error + Send + Sync> = Box::new(TimeoutError);
    let error = R2SqlQueryError::new(
        "Failed to run R2 SQL stats query",
        None,
        None,
        None,
        Some(cause),
    );

    assert_eq!(
        error.to_string(),
        "Failed to run R2 SQL stats query: TimeoutError: The operation timed out."
    );
    let cause = error.cause().expect("cause is preserved");
    assert_eq!(cause.to_string(), "TimeoutError: The operation timed out.");
}

#[test]
fn preserves_an_r2_response_error_and_request_details() {
    let error = R2SqlQueryError::new(
        "R2 SQL stats query failed",
        Some("test-request".to_string()),
        Some(400),
        None,
        None,
    );

    assert_eq!(error.message(), "R2 SQL stats query failed");
    assert_eq!(error.status(), Some(400));
    assert_eq!(error.request_id(), Some("test-request"));
}

#[test]
fn still_rejects_capped_results_without_an_aggregate_cursor() {
    let result = query_r2_sql_pages("SELECT * FROM aggregates LIMIT 10000", None, |_query| {
        Ok(vec![BTreeMap::new(); 10_000])
    });
    let error = result.expect_err("capped result without a cursor is rejected");
    assert!(error.to_string().contains("10000 row limit"));
}

#[test]
fn propagates_a_later_page_failure_instead_of_returning_partial_aggregates() {
    let mut calls: Vec<String> = Vec::new();
    let error = R2SqlQueryError::new("second page failed", None, None, None, None);
    let result = query_r2_sql_pages("SELECT * FROM aggregates", Some(&["model"]), |query| {
        calls.push(query.to_string());
        if calls.len() == 1 {
            Ok((0..10_000)
                .map(|index| row(&[("model", &index.to_string())]))
                .collect())
        } else {
            Err(R2SqlQueryError::new(
                "second page failed",
                None,
                None,
                None,
                None,
            ))
        }
    });

    let returned = result.expect_err("later page failure propagates");
    assert_eq!(returned.message(), error.message());
    assert_eq!(calls.len(), 2);
}

#[test]
fn retries_a_transient_timeout_on_the_same_page_without_repeating_earlier_rows() {
    let mut calls: Vec<String> = Vec::new();
    let rows = query_r2_sql_pages("SELECT * FROM aggregates", Some(&["model"]), |query| {
        calls.push(query.to_string());
        match calls.len() {
            1 => Ok((0..10_000)
                .map(|index| row(&[("model", &format!("{index:05}"))]))
                .collect()),
            2 => Err(R2SqlQueryError::new(
                "query timeout",
                None,
                Some(400),
                Some(40005),
                None,
            )),
            _ => Ok(vec![row(&[("model", "10000")])]),
        }
    })
    .expect("transient page is retried");

    assert_eq!(rows.len(), 10_001);
    let unique: std::collections::BTreeSet<&str> = rows
        .iter()
        .map(|row| row.get("model").map(String::as_str).unwrap_or(""))
        .collect();
    assert_eq!(unique.len(), 10_001);
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[1], calls[2]);
    assert_ne!(calls[0], calls[1]);
}

#[test]
fn bounds_transient_retries_and_preserves_the_final_error() {
    let mut calls: Vec<String> = Vec::new();
    let result = query_r2_sql_pages("SELECT * FROM aggregates", None, |query| {
        calls.push(query.to_string());
        Err(R2SqlQueryError::new(
            "unavailable",
            None,
            Some(503),
            None,
            None,
        ))
    });

    let error = result.expect_err("persistent transient failures exhaust retries");
    assert_eq!(calls.len(), 3);
    assert_eq!(error.message(), "unavailable");
}

#[test]
#[ignore = "porting: sqlite-backed aggregate reads not implemented"]
fn reads_all_aggregate_rows_without_losing_or_repeating_groups() {
    for count in [0usize, 9_999, 10_000, 10_001, 20_005] {
        let mut pages: Vec<usize> = Vec::new();
        let rows = query_r2_sql_pages(
            "SELECT * FROM aggregates",
            Some(&["dimension", "model", "country"]),
            |_query| {
                let page: Vec<R2SqlData> = Vec::new();
                pages.push(page.len());
                Ok(page)
            },
        )
        .expect("aggregate reads succeed");

        assert_eq!(rows.len(), count);
        assert_eq!(pages.len(), count / 10_000 + 1);
        assert_eq!(pages.last().copied(), Some(count % 10_000));
    }
}
