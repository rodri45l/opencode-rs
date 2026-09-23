//! Zen request-body streaming patch.
//!
//! Port of `packages/console/app/src/routes/zen/util/requestBody.ts`
//! (upstream 18ef3cc): the root `model` field is patched without buffering the
//! remaining body, a late model field buffers only up to that point, and stream
//! usage options are appended at the end of the request.

/// A prepared request body with its detected root model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRequest {
    pub model: String,
    pub reads: usize,
    pub body: Vec<u8>,
    model_start: usize,
    model_end: usize,
}

impl PreparedRequest {
    /// Patch the model and optionally append stream usage options.
    pub fn stream(&self, provider_model: &str, include_usage: bool) -> Vec<u8> {
        let mut output = Vec::with_capacity(self.body.len() + provider_model.len() + 48);
        output.extend_from_slice(&self.body[..self.model_start]);
        output.extend_from_slice(provider_model.as_bytes());
        output.extend_from_slice(&self.body[self.model_end..]);
        if include_usage {
            if let Some(close) = output.iter().rposition(|&byte| byte == b'}') {
                let tail = output.split_off(close);
                output.extend_from_slice(b",\"stream_options\":{\"include_usage\":true}");
                output.extend_from_slice(&tail);
            }
        }
        output
    }
}

/// Scan the chunk list, find the root `model` value, and record the read count.
pub fn prepare_request_body(chunks: Vec<Vec<u8>>) -> PreparedRequest {
    let mut body = Vec::new();
    let mut reads = 0;
    let mut found: Option<(usize, usize, String)> = None;
    for (index, chunk) in chunks.iter().enumerate() {
        body.extend_from_slice(chunk);
        if found.is_none() {
            reads = index + 1;
            if let Some(model) = find_root_model(&body) {
                found = Some(model);
            }
        }
    }
    let (model_start, model_end, model) =
        found.unwrap_or_else(|| (body.len(), body.len(), String::new()));
    PreparedRequest {
        model,
        reads,
        body,
        model_start,
        model_end,
    }
}

/// Find the string value of the depth-1 `model` key, returning byte offsets.
fn find_root_model(bytes: &[u8]) -> Option<(usize, usize, String)> {
    let mut index = if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        3
    } else {
        0
    };
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut phase: u8 = 0; // 0 = key, 1 = colon, 2 = value, 3 = comma
    let mut key = String::new();
    let mut content_start = 0usize;

    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                if depth == 1 && phase == 0 {
                    key = String::from_utf8_lossy(&bytes[content_start..index]).into_owned();
                    phase = 1;
                } else if depth == 1 && phase == 2 {
                    if key == "model" {
                        let value =
                            String::from_utf8_lossy(&bytes[content_start..index]).into_owned();
                        return Some((content_start, index, value));
                    }
                    phase = 3;
                }
                in_string = false;
            }
            index += 1;
            continue;
        }

        match byte {
            b'"' => {
                in_string = true;
                content_start = index + 1;
                index += 1;
            }
            b'{' | b'[' => {
                if depth == 1 && phase == 2 {
                    phase = 3;
                }
                depth += 1;
                index += 1;
            }
            b'}' | b']' => {
                depth -= 1;
                index += 1;
            }
            b':' if phase == 1 => {
                phase = 2;
                index += 1;
            }
            b',' if phase == 3 => {
                phase = 0;
                index += 1;
            }
            _ => {
                if depth == 1 && phase == 2 && !byte.is_ascii_whitespace() {
                    phase = 3;
                }
                index += 1;
            }
        }
    }
    None
}
