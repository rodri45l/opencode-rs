//! Error page description key (port of packages/app/src/pages/error-description.ts).

#[derive(Clone, Debug, PartialEq)]
pub struct AppError {
    pub message: String,
    pub local_server_startup: Option<bool>,
}

pub fn error_description_key(error: &AppError) -> String {
    if error.local_server_startup == Some(true) {
        "error.page.description.localServerStartup".to_string()
    } else {
        "error.page.description".to_string()
    }
}
