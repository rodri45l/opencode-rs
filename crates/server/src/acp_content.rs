//! ACP content-block projections.
//!
//! Ports the observable behaviour of `packages/opencode/src/acp/content.ts`:
//! `contentBlockToParts`, `promptContentToParts`, and `partsToContentChunks`.

use base64::Engine;
use serde_json::{json, Value};
use std::path::Path;

/// A typed ACP content error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpContentError {
    /// The behaviour has not been ported yet.
    NotImplemented(&'static str),
}

impl std::fmt::Display for AcpContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "not implemented: {what}"),
        }
    }
}

impl std::error::Error for AcpContentError {}

/// Project a list of ACP content blocks into prompt parts.
pub fn prompt_content_to_parts(blocks: &Value) -> Result<Value, AcpContentError> {
    let mut parts = Vec::new();
    for block in blocks.as_array().cloned().unwrap_or_default() {
        if let Some(mapped) = content_block_to_parts(&block)?.as_array_mut() {
            parts.append(mapped);
        }
    }
    Ok(Value::Array(parts))
}

/// Project a single ACP content block into prompt parts.
pub fn content_block_to_parts(block: &Value) -> Result<Value, AcpContentError> {
    let block_type = block
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match block_type {
        "text" => {
            let mut part = json!({
                "type": "text",
                "text": block.get("text").and_then(Value::as_str).unwrap_or_default(),
            });
            if let Some(flags) =
                audience_flags(block.get("annotations").and_then(|a| a.get("audience")))
            {
                if flags == "assistant" {
                    part["synthetic"] = json!(true);
                } else if flags == "user" {
                    part["ignored"] = json!(true);
                }
            }
            Ok(json!([part]))
        }
        "image" => {
            let mime = block
                .get("mimeType")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let uri = block.get("uri").and_then(Value::as_str);
            let data = block
                .get("data")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !data.is_empty() {
                return Ok(json!([{
                    "type": "file",
                    "url": format!("data:{mime};base64,{data}"),
                    "filename": filename_from_uri(uri).unwrap_or_else(|| "image".to_string()),
                    "mime": mime,
                }]));
            }
            if let Some(uri) = uri {
                if uri.starts_with("data:")
                    || uri.starts_with("http://")
                    || uri.starts_with("https://")
                {
                    return Ok(json!([{
                        "type": "file",
                        "url": uri,
                        "filename": filename_from_uri(Some(uri)).unwrap_or_else(|| "image".to_string()),
                        "mime": mime,
                    }]));
                }
            }
            Ok(json!([]))
        }
        "resource_link" => {
            let part = resource_link_to_part(block);
            Ok(json!([part]))
        }
        "resource" => {
            let resource = block.get("resource").cloned().unwrap_or(Value::Null);
            if resource.get("text").and_then(Value::as_str).is_some() {
                let uri = resource
                    .get("uri")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let text = resource
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if let Ok(parsed) = url::Url::parse(uri) {
                    if parsed.scheme() == "file" {
                        let line = parsed
                            .fragment()
                            .and_then(|fragment| fragment.strip_prefix('L'))
                            .and_then(|rest| {
                                let digits: String =
                                    rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
                                if digits.is_empty() {
                                    None
                                } else {
                                    Some(digits)
                                }
                            });
                        let filepath = parsed.path().to_string();
                        let label = match line {
                            Some(line) => format!("{filepath}:{line}"),
                            None => filepath,
                        };
                        return Ok(
                            json!([{ "type": "text", "text": format!("[{label}]\n{text}") }]),
                        );
                    }
                }
                return Ok(json!([{ "type": "text", "text": format!("[{uri}]\n{text}") }]));
            }
            if let Some(mime) = resource.get("mimeType").and_then(Value::as_str) {
                let uri = resource
                    .get("uri")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let blob = resource
                    .get("blob")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let url = if uri.starts_with("data:") {
                    uri.to_string()
                } else {
                    format!("data:{mime};base64,{blob}")
                };
                return Ok(json!([{
                    "type": "file",
                    "url": url,
                    "filename": filename_from_uri(Some(uri)).unwrap_or_else(|| "file".to_string()),
                    "mime": mime,
                }]));
            }
            Ok(json!([]))
        }
        _ => Ok(json!([])),
    }
}

/// Project replay parts into ACP content chunks.
pub fn parts_to_content_chunks(parts: &Value) -> Result<Value, AcpContentError> {
    let mut chunks = Vec::new();
    for part in parts.as_array().cloned().unwrap_or_default() {
        if let Some(mapped) = part_to_content_chunks(&part)?.as_array_mut() {
            chunks.append(mapped);
        }
    }
    Ok(Value::Array(chunks))
}

/// Project a single replay part into ACP content chunks.
pub fn part_to_content_chunks(part: &Value) -> Result<Value, AcpContentError> {
    match part.get("type").and_then(Value::as_str).unwrap_or_default() {
        "text" => {
            let text = part.get("text").and_then(Value::as_str).unwrap_or_default();
            if text.is_empty() {
                return Ok(json!([]));
            }
            let mut content = json!({ "type": "text", "text": text });
            if part.get("synthetic").and_then(Value::as_bool) == Some(true) {
                content["annotations"] = json!({ "audience": ["assistant"] });
            } else if part.get("ignored").and_then(Value::as_bool) == Some(true) {
                content["annotations"] = json!({ "audience": ["user"] });
            }
            Ok(json!([{ "content": content }]))
        }
        "file" => file_part_to_content_chunks(part),
        "reasoning" => {
            let text = part.get("text").and_then(Value::as_str).unwrap_or_default();
            if text.is_empty() {
                return Ok(json!([]));
            }
            Ok(json!([{ "content": { "type": "text", "text": text } }]))
        }
        _ => Ok(json!([])),
    }
}

fn audience_flags(audience: Option<&Value>) -> Option<&'static str> {
    let audience = audience?.as_array()?;
    if audience.len() == 1 {
        match audience[0].as_str() {
            Some("assistant") => return Some("assistant"),
            Some("user") => return Some("user"),
            _ => {}
        }
    }
    None
}

fn resource_link_to_part(link: &Value) -> Value {
    let uri = link.get("uri").and_then(Value::as_str).unwrap_or_default();
    let mime = link
        .get("mimeType")
        .and_then(Value::as_str)
        .unwrap_or("text/plain");
    let filename = link.get("name").and_then(Value::as_str);
    uri_to_file_part(uri, mime, filename)
}

fn uri_to_file_part(uri: &str, mime: &str, filename: Option<&str>) -> Value {
    if uri.starts_with("file://") {
        return json!({
            "type": "file",
            "url": uri,
            "filename": filename.map(str::to_string).or_else(|| filename_from_uri(Some(uri))).unwrap_or_else(|| "file".to_string()),
            "mime": mime,
        });
    }
    if uri.starts_with("zed://") {
        if let Ok(parsed) = url::Url::parse(uri) {
            if let Some(pathname) = parsed
                .query_pairs()
                .find(|(key, _)| key == "path")
                .map(|(_, value)| value.into_owned())
            {
                let derived = Path::new(&pathname)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let name = filename.map(str::to_string).unwrap_or_else(|| {
                    if derived.is_empty() {
                        "file".to_string()
                    } else {
                        derived
                    }
                });
                return json!({
                    "type": "file",
                    "url": path_to_file_url(&pathname),
                    "filename": name,
                    "mime": mime,
                });
            }
        }
    }
    json!({ "type": "text", "text": uri })
}

fn file_part_to_content_chunks(part: &Value) -> Result<Value, AcpContentError> {
    let url = part.get("url").and_then(Value::as_str).unwrap_or_default();
    if url.starts_with("file://") {
        return Ok(json!([{
            "content": {
                "type": "resource_link",
                "uri": url,
                "name": part.get("filename").and_then(Value::as_str).unwrap_or("file"),
                "mimeType": part.get("mime").and_then(Value::as_str).unwrap_or_default(),
            },
        }]));
    }
    if !url.starts_with("data:") {
        return Ok(json!([]));
    }
    let Some((mime, base64)) = decode_data_url(url) else {
        return Ok(json!([]));
    };
    let filename = part
        .get("filename")
        .and_then(Value::as_str)
        .unwrap_or("file");
    if mime.starts_with("image/") {
        return Ok(json!([{
            "content": {
                "type": "image",
                "mimeType": mime,
                "data": base64,
                "uri": path_to_file_url(filename),
            },
        }]));
    }
    let resource = if mime.starts_with("text/") || mime == "application/json" {
        let text = base64::engine::general_purpose::STANDARD
            .decode(&base64)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_default();
        json!({
            "uri": path_to_file_url(filename),
            "mimeType": mime,
            "text": text,
        })
    } else {
        json!({
            "uri": path_to_file_url(filename),
            "mimeType": mime,
            "blob": base64,
        })
    };
    Ok(json!([{ "content": { "type": "resource", "resource": resource } }]))
}

fn decode_data_url(url: &str) -> Option<(String, String)> {
    let rest = url.strip_prefix("data:")?;
    let (mime, payload) = rest.split_once(";base64,")?;
    Some((mime.to_string(), payload.to_string()))
}

fn filename_from_uri(uri: Option<&str>) -> Option<String> {
    let uri = uri?;
    if uri.starts_with("data:") {
        return None;
    }
    if let Ok(parsed) = url::Url::parse(uri) {
        let name = Path::new(parsed.path())
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        return name.filter(|name| !name.is_empty());
    }
    Path::new(uri)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
}

fn path_to_file_url(path: &str) -> String {
    let candidate = Path::new(path);
    let absolute = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(candidate)
    };
    format!("file://{}", absolute.to_string_lossy())
}
