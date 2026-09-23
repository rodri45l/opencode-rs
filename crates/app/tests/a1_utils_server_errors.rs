//! Port of packages/app/src/utils/server-errors.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

fn t(key: &str, vars: &[(&str, &str)]) -> String {
    let text = match key {
        "error.chain.unknown" => "Erro desconhecido",
        "error.chain.configInvalid" => "Arquivo de config em {{path}} invalido",
        "error.chain.configInvalidWithMessage" => {
            "Arquivo de config em {{path}} invalido: {{message}}"
        }
        "error.chain.modelNotFound" => "Modelo nao encontrado: {{provider}}/{{model}}",
        "error.chain.didYouMean" => "Voce quis dizer: {{suggestions}}",
        "error.chain.checkConfig" => "Revise provider/model no config",
        _ => return key.to_string(),
    };
    let mut out = text.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("{{{{{k}}}}}"), v);
    }
    out
}

#[derive(Clone, Debug, PartialEq)]
enum ServerError {
    ConfigInvalidIssues {
        path: String,
        issues: Vec<(Vec<String>, String)>,
    },
    ConfigInvalidMessage {
        path: String,
        message: String,
    },
    ProviderModelNotFound {
        provider_id: String,
        model_id: String,
        suggestions: Vec<String>,
    },
    Plain(String),
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
struct WrappedError {
    cause_body: Option<Box<ServerError>>,
    cause_status: Option<u16>,
}

#[derive(Clone, Debug, PartialEq)]
struct SessionNotFoundBody {
    session_id: String,
    status: u16,
}

// Local stubs (fast wave): real module lands later.
fn parse_readable_config_invalid_error(_error: &ServerError) -> String {
    String::new()
}

fn format_server_error(_error: &ServerError, _wrapped: Option<&WrappedError>) -> String {
    String::new()
}

fn is_session_not_found_error(_body: &SessionNotFoundBody, _session_id: &str) -> bool {
    false
}

#[test]
#[ignore = "porting: utils/server-errors not implemented"]
fn formats_issues_with_file_path() {
    let error = ServerError::ConfigInvalidIssues {
        path: "opencode.config.ts".into(),
        issues: vec![
            (vec!["settings".into(), "host".into()], "Required".into()),
            (vec!["mode".into()], "Invalid".into()),
        ],
    };
    let expected = [
        "Arquivo de config em opencode.config.ts invalido: settings.host: Required",
        "mode: Invalid",
    ]
    .join("\n");
    assert_eq!(parse_readable_config_invalid_error(&error), expected);
}

#[test]
#[ignore = "porting: utils/server-errors not implemented"]
fn uses_trimmed_message_when_issues_are_missing() {
    let error = ServerError::ConfigInvalidMessage {
        path: "config".into(),
        message: "  Bad value  ".into(),
    };
    assert_eq!(
        parse_readable_config_invalid_error(&error),
        "Arquivo de config em config invalido: Bad value"
    );
}

#[test]
#[ignore = "porting: utils/server-errors not implemented"]
fn formats_config_invalid_errors() {
    let error = ServerError::ConfigInvalidMessage {
        path: "config".into(),
        message: "Missing host".into(),
    };
    assert_eq!(
        format_server_error(&error, None),
        "Arquivo de config em config invalido: Missing host"
    );
}

#[test]
#[ignore = "porting: utils/server-errors not implemented"]
fn returns_error_messages() {
    assert_eq!(
        format_server_error(
            &ServerError::Plain("Request failed with status 503".into()),
            None
        ),
        "Request failed with status 503"
    );
}

#[test]
#[ignore = "porting: utils/server-errors not implemented"]
fn returns_provided_string_errors() {
    assert_eq!(
        format_server_error(
            &ServerError::Plain("Failed to connect to server".into()),
            None
        ),
        "Failed to connect to server"
    );
}

#[test]
#[ignore = "porting: utils/server-errors not implemented"]
fn uses_translated_unknown_fallback() {
    assert_eq!(
        format_server_error(&ServerError::Unknown, None),
        t("error.chain.unknown", &[])
    );
}

#[test]
#[ignore = "porting: utils/server-errors not implemented"]
fn falls_back_for_unknown_error_objects_and_names() {
    assert_eq!(
        format_server_error(&ServerError::Unknown, None),
        "Erro desconhecido"
    );
}

#[test]
#[ignore = "porting: utils/server-errors not implemented"]
fn formats_provider_model_errors_using_provider_model() {
    let error = ServerError::ProviderModelNotFound {
        provider_id: "openai".into(),
        model_id: "gpt-4.1".into(),
        suggestions: Vec::new(),
    };
    let expected = [
        "Modelo nao encontrado: openai/gpt-4.1",
        "Revise provider/model no config",
    ]
    .join("\n");
    assert_eq!(format_server_error(&error, None), expected);
}

#[test]
#[ignore = "porting: utils/server-errors not implemented"]
fn formats_provider_model_suggestions() {
    let error = ServerError::ProviderModelNotFound {
        provider_id: "x".into(),
        model_id: "y".into(),
        suggestions: vec!["x/y2".into(), "x/y3".into()],
    };
    let expected = [
        "Modelo nao encontrado: x/y",
        "Voce quis dizer: x/y2, x/y3",
        "Revise provider/model no config",
    ]
    .join("\n");
    assert_eq!(format_server_error(&error, None), expected);
}

#[test]
#[ignore = "porting: utils/server-errors not implemented"]
fn unwraps_sdk_wrapped_errors_from_cause_body() {
    let wrapped = WrappedError {
        cause_body: Some(Box::new(ServerError::ConfigInvalidMessage {
            path: "config".into(),
            message: "Missing host".into(),
        })),
        cause_status: Some(400),
    };
    assert_eq!(
        format_server_error(&ServerError::Unknown, Some(&wrapped)),
        "Arquivo de config em config invalido: Missing host"
    );
}

#[test]
#[ignore = "porting: utils/server-errors not implemented"]
fn matches_an_sdk_wrapped_error_for_the_requested_session() {
    let body = SessionNotFoundBody {
        session_id: "ses_missing".into(),
        status: 404,
    };
    assert!(is_session_not_found_error(&body, "ses_missing"));
}

#[test]
#[ignore = "porting: utils/server-errors not implemented"]
fn rejects_errors_for_other_sessions_and_other_404_responses() {
    let body = SessionNotFoundBody {
        session_id: "ses_parent".into(),
        status: 404,
    };
    assert!(!is_session_not_found_error(&body, "ses_tab"));
    let provider_body = SessionNotFoundBody {
        session_id: String::new(),
        status: 404,
    };
    assert!(!is_session_not_found_error(&provider_body, "ses_tab"));
}
