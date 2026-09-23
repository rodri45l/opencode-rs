//! Port of packages/app/src/pages/error-description.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::error_description::{error_description_key, AppError};

#[test]
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
