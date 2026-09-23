//! Reasoning summary extraction.
//!
//! Port of packages/tui/src/context/thinking.ts `reasoningSummary` (upstream 18ef3cc).

/// A reasoning summary split into an optional disclosure title and body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReasoningSummary {
    pub title: Option<String>,
    pub body: String,
}

/// Split a leading bolded title block from the markdown body.
pub fn reasoning_summary(text: &str) -> ReasoningSummary {
    let content = text.trim();
    if let Some(inner) = content.strip_prefix("**") {
        if let Some(close) = inner.find("**") {
            let title = &inner[..close];
            let after = &inner[close + 2..];
            let is_boundary =
                after.is_empty() || after.starts_with("\n\n") || after.starts_with("\r\n\r\n");
            if !title.is_empty() && !title.contains('\n') && !title.contains('*') && is_boundary {
                let body = after.trim_start_matches(['\r', '\n']).trim_end();
                return ReasoningSummary {
                    title: Some(title.trim().to_string()),
                    body: body.to_string(),
                };
            }
        }
    }
    ReasoningSummary {
        title: None,
        body: content.to_string(),
    }
}
