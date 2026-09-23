//! Server error formatting (port of packages/app/src/utils/server-errors.ts).

#[derive(Clone, Debug, PartialEq)]
pub enum ServerError {
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
pub struct WrappedError {
    pub cause_body: Option<Box<ServerError>>,
    pub cause_status: Option<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SessionNotFoundBody {
    pub session_id: String,
    pub status: u16,
}

fn config_file(path: &str) -> String {
    if path.is_empty() || path == "config" {
        "config".to_string()
    } else {
        path.to_string()
    }
}

pub fn parse_readable_config_invalid_error(error: &ServerError) -> String {
    match error {
        ServerError::ConfigInvalidIssues { path, issues } => {
            let file = config_file(path);
            let messages: Vec<String> = issues
                .iter()
                .map(|(parts, message)| {
                    let message = message.trim();
                    if parts.is_empty() {
                        message.to_string()
                    } else {
                        format!("{}: {message}", parts.join("."))
                    }
                })
                .filter(|message| !message.is_empty())
                .collect();
            if messages.is_empty() {
                format!("Arquivo de config em {file} invalido")
            } else {
                format!(
                    "Arquivo de config em {file} invalido: {}",
                    messages.join("\n")
                )
            }
        }
        ServerError::ConfigInvalidMessage { path, message } => {
            let file = config_file(path);
            let detail = message.trim();
            if detail.is_empty() {
                format!("Arquivo de config em {file} invalido")
            } else {
                format!("Arquivo de config em {file} invalido: {detail}")
            }
        }
        _ => String::new(),
    }
}

fn parse_readable_provider_model_not_found(error: &ServerError) -> String {
    match error {
        ServerError::ProviderModelNotFound {
            provider_id,
            model_id,
            suggestions,
        } => {
            let provider = provider_id.trim();
            let model = model_id.trim();
            let list: Vec<&str> = suggestions
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .take(5)
                .collect();
            let body = format!("Modelo nao encontrado: {provider}/{model}");
            let tail = "Revise provider/model no config";
            if list.is_empty() {
                format!("{body}\n{tail}")
            } else {
                format!("{body}\nVoce quis dizer: {}\n{tail}", list.join(", "))
            }
        }
        _ => String::new(),
    }
}

pub fn format_server_error(error: &ServerError, wrapped: Option<&WrappedError>) -> String {
    let unwrapped = wrapped
        .and_then(|value| value.cause_body.as_deref())
        .unwrap_or(error);
    match unwrapped {
        ServerError::ConfigInvalidIssues { .. } | ServerError::ConfigInvalidMessage { .. } => {
            parse_readable_config_invalid_error(unwrapped)
        }
        ServerError::ProviderModelNotFound { .. } => {
            parse_readable_provider_model_not_found(unwrapped)
        }
        ServerError::Plain(message) if !message.is_empty() => message.clone(),
        ServerError::Plain(_) => "Erro desconhecido".to_string(),
        ServerError::Unknown => "Erro desconhecido".to_string(),
    }
}

pub fn is_session_not_found_error(body: &SessionNotFoundBody, session_id: &str) -> bool {
    body.status == 404 && !body.session_id.is_empty() && body.session_id == session_id
}
