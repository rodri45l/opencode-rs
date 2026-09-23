//! V1/V2 server API compatibility (port of packages/app/src/utils/server-compat.ts).
//!
//! Only the pure request-shaping and routing helpers are modelled here.

#[derive(Clone, Debug, PartialEq)]
pub struct Mention {
    pub text: String,
    pub start: i64,
    pub end: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PromptFile {
    pub uri: String,
    pub name: String,
    pub mention: Option<Mention>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LegacyPart {
    Text {
        id: String,
        text: String,
    },
    File {
        id: String,
        mime: String,
        url: String,
        filename: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct PromptRequest {
    pub session_id: String,
    pub id: String,
    pub text: String,
    pub agent: Option<String>,
    pub model: Option<Model>,
    pub files: Vec<PromptFile>,
    pub legacy_parts: Option<Vec<LegacyPart>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileSource {
    pub source_type: String,
    pub text_value: String,
    pub start: i64,
    pub end: i64,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PromptPart {
    Text {
        text: String,
    },
    File {
        mime: String,
        url: String,
        filename: String,
        source: Option<FileSource>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct PromptBody {
    pub message_id: String,
    pub agent: Option<String>,
    pub model: Option<Model>,
    pub parts: Vec<PromptPart>,
}

fn mime(uri: &str) -> String {
    if let Some(rest) = uri.strip_prefix("data:") {
        let end = rest.find([';', ',']).unwrap_or(rest.len());
        let mime = &rest[..end];
        if !mime.is_empty() {
            return mime.to_string();
        }
    }
    "application/octet-stream".to_string()
}

pub fn convert_v1_prompt(request: &PromptRequest) -> PromptBody {
    let parts = if let Some(legacy) = &request.legacy_parts {
        legacy
            .iter()
            .map(|part| match part {
                LegacyPart::Text { text, .. } => PromptPart::Text { text: text.clone() },
                LegacyPart::File {
                    mime,
                    url,
                    filename,
                    ..
                } => PromptPart::File {
                    mime: mime.clone(),
                    url: url.clone(),
                    filename: filename.clone(),
                    source: None,
                },
            })
            .collect()
    } else {
        let mut parts = vec![PromptPart::Text {
            text: request.text.clone(),
        }];
        for file in &request.files {
            let (part_mime, source) = match &file.mention {
                Some(mention) => (
                    "text/plain".to_string(),
                    Some(FileSource {
                        source_type: "file".to_string(),
                        text_value: mention.text.clone(),
                        start: mention.start,
                        end: mention.end,
                        path: file.uri.clone(),
                    }),
                ),
                None => (mime(&file.uri), None),
            };
            parts.push(PromptPart::File {
                mime: part_mime,
                url: file.uri.clone(),
                filename: file.name.clone(),
                source,
            });
        }
        parts
    };

    PromptBody {
        message_id: request.id.clone(),
        agent: request.agent.clone(),
        model: request.model.clone(),
        parts,
    }
}

pub fn v1_list_path() -> String {
    "/experimental/session".to_string()
}

pub fn v1_file_find_query(_limit: i64) -> (String, String) {
    ("/find/file".to_string(), "false".to_string())
}

pub fn v1_permission_reply_path(session_id: &str, request_id: &str) -> String {
    format!("/session/{session_id}/permissions/{request_id}")
}

pub fn v1_connect_paths(integration_id: &str) -> Vec<String> {
    vec![
        format!("/auth/{integration_id}"),
        "/instance/dispose".to_string(),
        "/instance/dispose".to_string(),
    ]
}

pub fn v1_oauth_paths(integration_id: &str) -> Vec<String> {
    vec![
        format!("/provider/{integration_id}/oauth/callback"),
        "/instance/dispose".to_string(),
        "/instance/dispose".to_string(),
    ]
}
