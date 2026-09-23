//! Tool surface for the agent loop.
//!
//! Ports the observable behaviour of `packages/opencode/src/tool/**`: the
//! context/permission contract every tool shares, the write/grep/web-fetch
//! implementations, the external-directory guard, and the `Tool.define`
//! wrapper. Tools that have not landed yet return [`ToolError::NotImplemented`].

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::{json, Value};

/// Typed error returned by a tool execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    /// The behaviour has not been ported yet.
    NotImplemented(&'static str),
    /// The model called a tool with arguments that failed its schema.
    InvalidArguments {
        /// The tool id.
        tool: String,
        /// The actionable message fed back to the model.
        message: String,
    },
    /// A generic diagnostic.
    Message(String),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "not implemented: {what}"),
            Self::InvalidArguments { tool, message } => {
                write!(
                    f,
                    "{tool} tool was called with invalid arguments: {message}"
                )
            }
            Self::Message(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for ToolError {}

/// A permission request raised by a tool.
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionRequest {
    /// The permission being requested, e.g. `external_directory`.
    pub permission: String,
    /// The resource patterns the request covers.
    pub patterns: Vec<String>,
    /// The patterns a standing approval would cover.
    pub always: Vec<String>,
    /// Structured metadata about the request.
    pub metadata: Value,
}

/// The context passed to a tool execution.
#[derive(Debug, Clone, Default)]
pub struct ToolContext {
    /// Owning session id.
    pub session_id: String,
    /// Owning message id.
    pub message_id: String,
    /// Tool call id.
    pub call_id: String,
    /// Requesting agent name.
    pub agent: String,
    /// The instance directory relative paths resolve against.
    pub directory: PathBuf,
    /// Permission requests raised during execution.
    pub requests: Vec<PermissionRequest>,
}

impl ToolContext {
    /// Record a permission request on the context.
    pub fn ask(&mut self, request: PermissionRequest) {
        self.requests.push(request);
    }
}

/// A file attachment returned by a tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    /// Attachment kind; always `file`.
    pub kind: String,
    /// The media type.
    pub mime: String,
    /// The `data:` URL carrying the bytes.
    pub url: String,
}

/// The result of a tool execution.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolResult {
    /// Short human title.
    pub title: String,
    /// Model-facing output.
    pub output: String,
    /// Structured metadata.
    pub metadata: Value,
    /// Attachments, when the tool produced media.
    pub attachments: Option<Vec<Attachment>>,
}

/// Resolve a tool path against the instance directory.
fn resolve(directory: &Path, file_path: &str) -> PathBuf {
    let path = Path::new(file_path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        directory.join(path)
    }
}

fn relative(directory: &Path, path: &Path) -> String {
    path.strip_prefix(directory)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn to_slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Standard base64 encoding (RFC 4648, padded).
fn base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Arguments for [`WriteTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteArgs {
    /// Target file path.
    pub file_path: String,
    /// Content to write.
    pub content: String,
}

/// The `write` tool.
#[derive(Debug, Default)]
pub struct WriteTool;

impl WriteTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Write `content` to `file_path`, preserving an existing BOM.
    pub fn execute(&self, args: WriteArgs, ctx: &mut ToolContext) -> Result<ToolResult, ToolError> {
        let path = resolve(&ctx.directory, &args.file_path);
        let existed = path.exists();
        let had_bom = existed
            && std::fs::read(&path)
                .map(|bytes| bytes.starts_with(&[0xEF, 0xBB, 0xBF]))
                .unwrap_or(false);

        let mut bytes = args.content.into_bytes();
        if had_bom && !bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            let mut with_bom = vec![0xEF, 0xBB, 0xBF];
            with_bom.append(&mut bytes);
            bytes = with_bom;
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ToolError::Message(e.to_string()))?;
        }
        std::fs::write(&path, &bytes).map_err(|e| ToolError::Message(e.to_string()))?;

        Ok(ToolResult {
            title: relative(&ctx.directory, &path),
            output: "Wrote file successfully".to_string(),
            metadata: json!({
                "filepath": path.to_string_lossy(),
                "exists": existed,
            }),
            attachments: None,
        })
    }
}

/// Arguments for [`GrepTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrepArgs {
    /// The search pattern.
    pub pattern: String,
    /// Directory or file to search.
    pub path: Option<String>,
    /// Optional include glob.
    pub include: Option<String>,
}

/// The `grep` tool.
#[derive(Debug, Default)]
pub struct GrepTool;

impl GrepTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Search files for `pattern`.
    pub fn execute(
        &self,
        _args: GrepArgs,
        _ctx: &mut ToolContext,
    ) -> Result<ToolResult, ToolError> {
        Err(ToolError::NotImplemented("tool::GrepTool::execute"))
    }
}

/// A raw HTTP response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    /// The status code.
    pub status: u16,
    /// The `content-type` header value.
    pub content_type: String,
    /// The response body bytes.
    pub body: Vec<u8>,
}

/// Minimal HTTP client abstraction so the fetch tool is testable in-process.
pub trait HttpClient {
    /// Perform a `GET`.
    fn get(&self, url: &str) -> Result<HttpResponse, ToolError>;
}

/// Arguments for [`WebFetchTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebFetchArgs {
    /// The URL to fetch.
    pub url: String,
    /// `text`, `markdown`, or `html`.
    pub format: String,
}

/// The `webfetch` tool.
#[derive(Debug, Default)]
pub struct WebFetchTool;

impl WebFetchTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Fetch `url` using `client` and render it in `format`.
    pub fn execute_with(
        &self,
        client: &dyn HttpClient,
        args: WebFetchArgs,
        _ctx: &mut ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let response = client.get(&args.url)?;
        let content_type = response
            .content_type
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();

        if content_type.starts_with("image/") && content_type != "image/svg+xml" {
            let mime = content_type.clone();
            return Ok(ToolResult {
                title: "webfetch".to_string(),
                output: "Image fetched successfully".to_string(),
                metadata: Value::Null,
                attachments: Some(vec![Attachment {
                    kind: "file".to_string(),
                    mime: mime.clone(),
                    url: format!("data:{mime};base64,{}", base64(&response.body)),
                }]),
            });
        }

        let body = String::from_utf8_lossy(&response.body).into_owned();
        let output = if args.format == "text" && content_type == "text/html" {
            extract_text(&body)
        } else {
            body
        };

        Ok(ToolResult {
            title: "webfetch".to_string(),
            output,
            metadata: Value::Null,
            attachments: None,
        })
    }
}

/// Extract visible text from HTML, dropping script/style contents.
fn extract_text(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    loop {
        let lower = rest.to_ascii_lowercase();
        let Some(open) = lower.find('<') else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..open]);
        let Some(close) = rest[open..].find('>') else {
            break;
        };
        let tag = &rest[open + 1..open + close];
        let tag_name = tag
            .trim_start_matches('/')
            .split(|c: char| c.is_whitespace() || c == '/')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        if tag_name == "script" || tag_name == "style" {
            let end_marker = format!("</{tag_name}");
            if let Some(end) = lower[open + close + 1..].find(&end_marker) {
                let after = open + close + 1 + end;
                if let Some(end_close) = rest[after..].find('>') {
                    rest = &rest[after + end_close + 1..];
                    continue;
                }
            }
            break;
        }
        rest = &rest[open + close + 1..];
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Options for [`assert_external_directory`].
#[derive(Debug, Clone, Default)]
pub struct ExternalDirectoryOptions {
    /// `directory` when the target itself is a directory.
    pub kind: Option<String>,
    /// Skip the check entirely.
    pub bypass: bool,
}

/// Ask for `external_directory` permission when `target` is outside the
/// instance directory.
pub fn assert_external_directory(
    ctx: &mut ToolContext,
    target: Option<&str>,
    options: &ExternalDirectoryOptions,
) -> Result<(), ToolError> {
    if options.bypass {
        return Ok(());
    }
    let Some(target) = target.filter(|target| !target.is_empty()) else {
        return Ok(());
    };

    let path = resolve(&ctx.directory, target);
    let inside = path.starts_with(&ctx.directory);
    if inside {
        return Ok(());
    }

    let scope = if options.kind.as_deref() == Some("directory") {
        path.clone()
    } else {
        path.parent().map(Path::to_path_buf).unwrap_or(path.clone())
    };
    let pattern = to_slash(&scope.join("*"));
    ctx.ask(PermissionRequest {
        permission: "external_directory".to_string(),
        patterns: vec![pattern.clone()],
        always: vec![pattern],
        metadata: Value::Null,
    });
    Ok(())
}

/// The execute function of a constructed tool.
pub type ToolExecute = Arc<dyn Fn(Value, &mut ToolContext) -> Result<ToolResult, ToolError>>;

/// A constructed tool instance.
#[derive(Clone)]
pub struct ToolSpec {
    /// Tool description shown to the model.
    pub description: String,
    /// The tool's parameter schema.
    pub parameters: Value,
    /// The execute function.
    pub execute: ToolExecute,
}

/// A tool definition that materializes a fresh [`ToolSpec`] per `init`.
#[derive(Clone)]
pub struct ToolDefinition {
    id: String,
    factory: Arc<dyn Fn() -> ToolSpec>,
}

impl ToolDefinition {
    /// Define a tool from a factory.
    pub fn define(
        id: impl Into<String>,
        factory: impl Fn() -> ToolSpec + 'static,
    ) -> Result<Self, ToolError> {
        Ok(Self {
            id: id.into(),
            factory: Arc::new(factory),
        })
    }

    /// The tool id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Materialize a fresh tool instance.
    pub fn init(&self) -> Result<ToolSpec, ToolError> {
        Ok((self.factory)())
    }
}
