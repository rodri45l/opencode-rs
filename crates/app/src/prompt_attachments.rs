//! Prompt attachment helpers (port of
//! packages/app/src/components/prompt-input/files.ts and paste.ts).

#[derive(Clone, Debug, PartialEq)]
pub struct AttachmentFile {
    pub name: String,
    pub browser_mime: String,
    pub bytes: Vec<u8>,
}

const IMAGE_MIMES: [&str; 4] = ["image/png", "image/jpeg", "image/gif", "image/webp"];
const TEXT_MIMES: [&str; 8] = [
    "application/json",
    "application/ld+json",
    "application/toml",
    "application/x-toml",
    "application/x-yaml",
    "application/xml",
    "application/yaml",
    "application/xml",
];

const SAMPLE: usize = 4096;
const LARGE_PASTE_CHARS: usize = 8000;
const LARGE_PASTE_BREAKS: usize = 120;

fn kind(mime: &str) -> String {
    mime.split(';').next().unwrap_or("").trim().to_lowercase()
}

fn extension(name: &str) -> String {
    match name.rfind('.') {
        Some(index) => name[index + 1..].to_lowercase(),
        None => String::new(),
    }
}

fn image_extension(suffix: &str) -> Option<&'static str> {
    match suffix {
        "gif" => Some("image/gif"),
        "jpeg" | "jpg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

fn is_text_mime(mime: &str) -> bool {
    if mime.is_empty() {
        return false;
    }
    mime.starts_with("text/")
        || TEXT_MIMES.contains(&mime)
        || mime.ends_with("+json")
        || mime.ends_with("+xml")
}

fn text_bytes(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return true;
    }
    let mut count = 0usize;
    for byte in bytes {
        if *byte == 0 {
            return false;
        }
        if *byte < 9 || (*byte > 13 && *byte < 32) {
            count += 1;
        }
    }
    count as f64 / bytes.len() as f64 <= 0.3
}

pub fn attachment_mime(file: &AttachmentFile) -> Option<String> {
    let mime = kind(&file.browser_mime);
    if IMAGE_MIMES.contains(&mime.as_str()) {
        return Some(mime);
    }
    if mime == "application/pdf" {
        return Some(mime);
    }

    let suffix = extension(&file.name);
    let fallback =
        image_extension(&suffix).or_else(|| (suffix == "pdf").then_some("application/pdf"));
    if (mime.is_empty() || mime == "application/octet-stream") && fallback.is_some() {
        return fallback.map(|value| value.to_string());
    }

    if is_text_mime(&mime) {
        return Some("text/plain".to_string());
    }
    let sample = &file.bytes[..file.bytes.len().min(SAMPLE)];
    if !text_bytes(sample) {
        return None;
    }
    Some("text/plain".to_string())
}

pub fn paste_mode(text: &str) -> String {
    if large_paste(text) {
        return "manual".to_string();
    }
    if text.contains('\n') || text.contains('\r') {
        return "manual".to_string();
    }
    "native".to_string()
}

fn large_paste(text: &str) -> bool {
    if text.len() >= LARGE_PASTE_CHARS {
        return true;
    }
    let mut breaks = 0;
    for character in text.chars() {
        if character != '\n' {
            continue;
        }
        breaks += 1;
        if breaks >= LARGE_PASTE_BREAKS {
            return true;
        }
    }
    false
}

#[derive(Default)]
pub struct PickOutcome {
    pub picked_paths: Vec<String>,
    pub files: Vec<AttachmentFile>,
    pub fallback: usize,
    pub errors: Vec<String>,
}

pub fn pick_attachment_files(
    directory: &str,
    has_picker: bool,
    picker_fails: bool,
    fallback: &mut usize,
    out: &mut PickOutcome,
) {
    if !has_picker {
        *fallback += 1;
        return;
    }
    out.picked_paths.push(directory.to_string());
    if picker_fails {
        out.errors.push("picker unavailable".to_string());
    }
}
