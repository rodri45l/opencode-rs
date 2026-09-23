//! Cloudflare R2 SQL cursor pagination.
//!
//! Derived from the observable behaviour pinned by
//! `packages/stats/core/src/r2-sql.ts` (upstream 18ef3cc).

use std::collections::BTreeMap;

/// Maximum rows returned by a single R2 SQL page.
const R2_SQL_MAX_ROWS: usize = 10_000;
/// Number of automatic retries for a transient page failure.
const R2_SQL_MAX_RETRIES: usize = 2;

/// A decoded R2 SQL row. Null cells are dropped at the transport boundary.
pub type R2SqlData = BTreeMap<String, String>;

/// An error raised while running an R2 SQL stats query.
#[derive(Debug)]
pub struct R2SqlQueryError {
    message: String,
    request_id: Option<String>,
    status: Option<u16>,
    code: Option<i64>,
    cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl R2SqlQueryError {
    pub fn new(
        message: impl Into<String>,
        request_id: Option<String>,
        status: Option<u16>,
        code: Option<i64>,
        cause: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self {
            message: message.into(),
            request_id,
            status,
            code,
            cause,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    pub fn status(&self) -> Option<u16> {
        self.status
    }

    pub fn code(&self) -> Option<i64> {
        self.code
    }

    pub fn cause(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        self.cause.as_deref()
    }
}

impl std::fmt::Display for R2SqlQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.cause {
            Some(cause) => write!(f, "{}: {}", self.message, cause),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for R2SqlQueryError {}

fn is_transient(error: &R2SqlQueryError) -> bool {
    error.code == Some(40005)
        || error.status == Some(429)
        || error.status.is_some_and(|status| status >= 500)
}

fn sql_escape_identifier(column: &str) -> String {
    format!("\"{}\"", column.replace('"', "\"\""))
}

fn sql_escape_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn page_query(query: &str, columns: &[&str], cursor: Option<&R2SqlData>) -> String {
    let keys: Vec<String> = columns
        .iter()
        .map(|column| format!("COALESCE({}, '')", sql_escape_identifier(column)))
        .collect();
    let values: Vec<String> = columns
        .iter()
        .map(|column| {
            let raw = cursor
                .and_then(|row| row.get(*column))
                .map(String::as_str)
                .unwrap_or("");
            sql_escape_literal(raw)
        })
        .collect();
    let after: Vec<String> = (0..keys.len())
        .map(|index| {
            let mut parts: Vec<String> = (0..index)
                .map(|before| format!("{} = {}", keys[before], values[before]))
                .collect();
            parts.push(format!("{} > {}", keys[index], values[index]));
            parts.join(" AND ")
        })
        .collect();
    let where_clause = if cursor.is_some() {
        format!(
            "WHERE {}",
            after
                .iter()
                .map(|condition| format!("({condition})"))
                .collect::<Vec<_>>()
                .join(" OR ")
        )
    } else {
        String::new()
    };
    format!(
        "SELECT * FROM ({query}) AS stats_page\n{where_clause}\nORDER BY {}\nLIMIT {R2_SQL_MAX_ROWS}",
        keys.join(", ")
    )
}

/// Page through an aggregate query, retrying transient page failures without
/// repeating earlier rows.
pub fn query_r2_sql_pages<F>(
    query: &str,
    columns: Option<&[&str]>,
    mut fetch_rows: F,
) -> Result<Vec<R2SqlData>, R2SqlQueryError>
where
    F: FnMut(&str) -> Result<Vec<R2SqlData>, R2SqlQueryError>,
{
    let paginate = columns.is_some_and(|columns| !columns.is_empty());
    let mut rows: Vec<R2SqlData> = Vec::new();
    let mut cursor: Option<R2SqlData> = None;
    loop {
        let page_query = if paginate {
            page_query(query, columns.unwrap_or(&[]), cursor.as_ref())
        } else {
            query.to_string()
        };
        let mut attempts = 0;
        let page = loop {
            match fetch_rows(&page_query) {
                Ok(page) => break page,
                Err(error) => {
                    if attempts < R2_SQL_MAX_RETRIES && is_transient(&error) {
                        attempts += 1;
                        continue;
                    }
                    return Err(error);
                }
            }
        };
        if page.len() >= R2_SQL_MAX_ROWS && !paginate {
            return Err(R2SqlQueryError::new(
                format!("R2 SQL stats query reached the {R2_SQL_MAX_ROWS} row limit"),
                None,
                None,
                None,
                None,
            ));
        }
        let complete = page.len() < R2_SQL_MAX_ROWS;
        cursor = page.last().cloned();
        rows.extend(page);
        if complete {
            return Ok(rows);
        }
    }
}
