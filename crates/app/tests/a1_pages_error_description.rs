//! Port of packages/app/src/pages/error-description.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct AppError {
    message: String,
    local_server_startup: Option<bool>,
}

// Local stub (fast wave): real module lands later.
fn error_description_key(_error: &AppError) -> String {
    String::new()
}

#[test]
#[ignore = "porting: pages/error-description not implemented"]
fn describes_local_server_startup_errors() {
    assert_eq!(
        error_description_key(&AppError {
            message: "migration failed".into(),
            local_server_startup: Some(true)
        }),
        "error.page.description.localServerStartup"
    );
}

#[test]
#[ignore = "porting: pages/error-description not implemented"]
fn uses_the_generic_description_for_other_errors() {
    assert_eq!(
        error_description_key(&AppError {
            message: "unknown".into(),
            local_server_startup: None
        }),
        "error.page.description"
    );
    assert_eq!(
        error_description_key(&AppError {
            message: "unknown".into(),
            local_server_startup: Some(false)
        }),
        "error.page.description"
    );
}
