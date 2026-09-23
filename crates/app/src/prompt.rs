//! Prompt extraction from message parts (port of packages/app/src/utils/prompt.ts).
//!
//! Only the text + data-URL image projection is modelled.

#[derive(Clone, Debug, PartialEq)]
pub enum Part {
    Text {
        id: String,
        text: String,
        session_id: String,
        message_id: String,
    },
    File {
        id: String,
        mime: String,
        url: String,
        filename: String,
        session_id: String,
        message_id: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum PromptItem {
    Text {
        content: String,
    },
    Image {
        filename: String,
        mime: String,
        blob_id: String,
    },
}

pub fn extract_prompt_from_parts(parts: &[Part]) -> Vec<PromptItem> {
    let text = parts
        .iter()
        .filter_map(|part| match part {
            Part::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .max_by_key(String::len)
        .unwrap_or_default();

    let mut result = vec![PromptItem::Text { content: text }];
    for part in parts {
        if let Part::File {
            mime,
            url,
            filename,
            ..
        } = part
        {
            if url.starts_with("data:") {
                result.push(PromptItem::Image {
                    filename: filename.clone(),
                    mime: mime.clone(),
                    blob_id: url.clone(),
                });
            }
        }
    }
    result
}
