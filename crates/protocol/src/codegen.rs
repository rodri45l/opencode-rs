//! HTTP API client codegen contract.
//!
//! Mirrors the observable contract of `packages/httpapi-codegen`: reflecting an
//! `HttpApi` into a [`Contract`] of grouped [`Endpoint`]s with flattened inputs,
//! resolved success kinds, declared error tags, and safe module paths, then
//! emitting virtual [`Output`] files and writing them beneath a directory owned
//! by a private manifest.
//!
//! The reference emitters render TypeScript; that rendering is not reproducible
//! in Rust, so only the *contract* the emitters share (operations, inputs,
//! success/void/stream classification, error tags, module paths, and writer
//! safety) is modelled here. The compile/emit/write phases are stubbed until
//! implemented; they return [`GenerationError`], never panic.

use std::collections::{HashMap, HashSet};

/// An HTTP method accepted by the generic endpoint constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Head,
    Trace,
}

/// The transport channel an input field is read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputSource {
    Params,
    Query,
    Headers,
    Payload,
}

/// One flattened input field on an operation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InputField {
    pub name: String,
    pub source: InputSource,
    pub optional: bool,
}

/// How the generated client accepts the flattened input object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputMode {
    None,
    Optional,
    Required,
}

/// The public shape of a successful response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SuccessKind {
    Value,
    Void,
    Stream,
}

/// A resolved public operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    pub group: String,
    pub name: String,
    pub input: Vec<InputField>,
    pub input_mode: InputMode,
    pub success: SuccessKind,
    pub errors: Vec<String>,
}

/// A resolved endpoint, keeping its transport and source identity internally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub endpoint_name: String,
    pub path: String,
    pub top_level: bool,
    pub unwrap_data: bool,
    pub operation: Operation,
}

/// A resolved group of endpoints with its consumer-facing and source names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub identifier: String,
    pub source_identifier: String,
    pub module: String,
    pub endpoints: Vec<Endpoint>,
}

/// The shared, emitter-independent contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contract {
    pub groups: Vec<Group>,
}

/// A virtual output file produced by an emitter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}

impl GeneratedFile {
    /// Construct a generated file.
    pub fn new(path: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
        }
    }
}

/// The output of one emitter: the public operations plus virtual files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub operations: Vec<Operation>,
    pub files: Vec<GeneratedFile>,
}

/// How a success schema is expressed in the source contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuccessSpec {
    /// A single value; `unwrap_data` mirrors an exact `{ data: A }` envelope.
    Value {
        unwrap_data: bool,
        identifier: Option<String>,
    },
    /// No content (`HttpApiSchema.NoContent`), emitted with status 204.
    NoContent,
    /// A text response (`HttpApiSchema.asText`), unsupported by the Promise emitter.
    Text { identifier: Option<String> },
    /// A binary response (`HttpApiSchema.asUint8Array`), unsupported by the Promise emitter.
    Binary { identifier: Option<String> },
    /// A server-sent event stream.
    StreamSse {
        identifier: Option<String>,
        /// Whether the stream declares its own error (unsupported by Promise).
        declared_error: bool,
    },
}

/// A declared error response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredError {
    pub status: u16,
    pub identifier: Option<String>,
}

/// Why a source schema cannot be projected exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaIssue {
    /// Semantics cannot be emitted exactly (opaque, custom, spoofed, lexical).
    Unportable,
    /// A hidden transformation needs the authoritative schema imported.
    RequiresAuthoritativeImport,
}

/// One source field on a transport channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSpec {
    pub name: String,
    pub optional: bool,
    /// Whether the schema can be emitted exactly (false forces authoritative import).
    pub portable: bool,
    pub identifier: Option<String>,
}

impl FieldSpec {
    /// A required, portable field.
    pub fn required(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            optional: false,
            portable: true,
            identifier: None,
        }
    }

    /// An optional, portable field.
    pub fn optional(name: impl Into<String>) -> Self {
        Self {
            optional: true,
            ..Self::required(name)
        }
    }
}

/// A source endpoint before reflection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointSpec {
    pub method: HttpMethod,
    pub name: String,
    pub path: String,
    pub params: Vec<FieldSpec>,
    pub query: Vec<FieldSpec>,
    pub headers: Vec<FieldSpec>,
    /// Zero, one, or multiple payload alternatives.
    pub payloads: Vec<Vec<FieldSpec>>,
    /// Zero, one, or multiple success schemas.
    pub successes: Vec<SuccessSpec>,
    pub errors: Vec<DeclaredError>,
    /// Errors contributed by server-only middleware.
    pub server_errors: Vec<String>,
    /// A client-required middleware key that needs an adapter.
    pub required_client_middleware: Option<String>,
    /// A success schema that cannot be projected exactly.
    pub success_issue: Option<SchemaIssue>,
}

impl EndpointSpec {
    /// Start an endpoint with a method, operation name, and path.
    pub fn new(method: HttpMethod, name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method,
            name: name.into(),
            path: path.into(),
            params: Vec::new(),
            query: Vec::new(),
            headers: Vec::new(),
            payloads: Vec::new(),
            successes: Vec::new(),
            errors: Vec::new(),
            server_errors: Vec::new(),
            required_client_middleware: None,
            success_issue: None,
        }
    }

    /// Mark the success schema as unportable.
    pub fn success_unportable(mut self) -> Self {
        self.success_issue = Some(SchemaIssue::Unportable);
        self
    }

    /// Mark the success schema as needing an authoritative import.
    pub fn success_requires_import(mut self) -> Self {
        self.success_issue = Some(SchemaIssue::RequiresAuthoritativeImport);
        self
    }

    /// Add a path parameter.
    pub fn param(mut self, name: impl Into<String>) -> Self {
        self.params.push(FieldSpec::required(name));
        self
    }

    /// Add a query field.
    pub fn query(mut self, name: impl Into<String>) -> Self {
        self.query.push(FieldSpec::required(name));
        self
    }

    /// Add an optional query field.
    pub fn query_optional(mut self, name: impl Into<String>) -> Self {
        self.query.push(FieldSpec::optional(name));
        self
    }

    /// Add a required header.
    pub fn header(mut self, name: impl Into<String>) -> Self {
        self.headers.push(FieldSpec::required(name));
        self
    }

    /// Add one payload struct with the given fields.
    pub fn payload(mut self, fields: &[&str]) -> Self {
        self.payloads
            .push(fields.iter().map(|f| FieldSpec::required(*f)).collect());
        self
    }

    /// Add a value success.
    pub fn success_value(mut self) -> Self {
        self.successes.push(SuccessSpec::Value {
            unwrap_data: false,
            identifier: None,
        });
        self
    }

    /// Add an exact `{ data: A }` success envelope.
    pub fn success_data_envelope(mut self) -> Self {
        self.successes.push(SuccessSpec::Value {
            unwrap_data: true,
            identifier: None,
        });
        self
    }

    /// Add a no-content success.
    pub fn success_no_content(mut self) -> Self {
        self.successes.push(SuccessSpec::NoContent);
        self
    }

    /// Add an SSE stream success.
    pub fn success_stream(mut self) -> Self {
        self.successes.push(SuccessSpec::StreamSse {
            identifier: None,
            declared_error: false,
        });
        self
    }

    /// Add an SSE stream success that declares its own error.
    pub fn success_stream_with_error(mut self) -> Self {
        self.successes.push(SuccessSpec::StreamSse {
            identifier: None,
            declared_error: true,
        });
        self
    }

    /// Add a text response success.
    pub fn success_text(mut self) -> Self {
        self.successes.push(SuccessSpec::Text { identifier: None });
        self
    }

    /// Add a binary response success.
    pub fn success_binary(mut self) -> Self {
        self.successes
            .push(SuccessSpec::Binary { identifier: None });
        self
    }

    /// Add a declared error.
    pub fn error(mut self, status: u16, identifier: impl Into<String>) -> Self {
        self.errors.push(DeclaredError {
            status,
            identifier: Some(identifier.into()),
        });
        self
    }

    /// Contribute a server-only middleware error tag.
    pub fn server_error(mut self, tag: impl Into<String>) -> Self {
        self.server_errors.push(tag.into());
        self
    }

    /// Require a client middleware adapter.
    pub fn requires_client_middleware(mut self, key: impl Into<String>) -> Self {
        self.required_client_middleware = Some(key.into());
        self
    }
}

/// A source group before reflection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupSpec {
    pub identifier: String,
    pub top_level: bool,
    pub endpoints: Vec<EndpointSpec>,
}

impl GroupSpec {
    /// A non-top-level group.
    pub fn new(identifier: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            top_level: false,
            endpoints: Vec::new(),
        }
    }

    /// Mark the group as top-level (endpoints hang off the client root).
    pub fn top_level(mut self) -> Self {
        self.top_level = true;
        self
    }

    /// Add an endpoint.
    pub fn endpoint(mut self, endpoint: EndpointSpec) -> Self {
        self.endpoints.push(endpoint);
        self
    }
}

/// An `HttpApi` source contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiSpec {
    pub identifier: String,
    pub groups: Vec<GroupSpec>,
}

impl ApiSpec {
    /// A named API with no groups.
    pub fn new(identifier: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            groups: Vec::new(),
        }
    }

    /// Add a group.
    pub fn group(mut self, group: GroupSpec) -> Self {
        self.groups.push(group);
        self
    }
}

/// Explicit compile-time renames and omissions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompileOptions {
    /// Maps a source group identifier to its consumer-facing identifier.
    pub group_names: HashMap<String, String>,
    /// Maps a source endpoint name to its public operation name.
    pub endpoint_names: HashMap<String, String>,
    /// Source endpoint names to drop entirely.
    pub omit_endpoints: HashSet<String>,
}

/// Per-operation output type overrides for the Promise emitter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PromiseEmitOptions {
    /// Maps `group.endpoint` to an authoritative imported wire type.
    pub output_types: HashMap<String, ImportedWireType>,
}

/// An authoritative imported wire type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedWireType {
    pub name: String,
    pub import: String,
}

/// How the imported Effect emitter references the authoritative API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportedSource {
    /// Import the API value itself.
    Api { module: String, api: String },
    /// Import one group.
    Group { module: String, group: String },
    /// Import individual endpoint constants by `group.endpoint` key.
    Endpoints {
        module: String,
        endpoints: HashMap<String, String>,
    },
}

/// A codegen failure. `reason()` reproduces the reference message verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationError {
    ClientMiddlewareRequiresAdapter { key: String },
    MultipleSuccessSchemas { name: String },
    MultiplePayloadSchemas { name: String },
    InputFieldCollision { name: String },
    ClientGroupNameCollision { identifier: String },
    ClientEndpointNameCollision { group: String, name: String },
    ClientNameCollision { name: String },
    UnportableSchema { path: String },
    AuthoritativeImportRequired { name: String },
    UnsupportedPromiseSuccessEncoding { name: String },
    UnsupportedPromisePathWildcard { path: String },
    UnsupportedPromiseStream { name: String },
    DuplicateOutputPath { path: String },
    UnsafeOutputPath { path: String },
    NotImplemented { topic: &'static str },
}

impl GenerationError {
    /// The reference failure message.
    pub fn reason(&self) -> String {
        match self {
            GenerationError::ClientMiddlewareRequiresAdapter { key } => {
                format!("Client middleware requires adapter: {key}")
            }
            GenerationError::MultipleSuccessSchemas { name } => {
                format!("Multiple success schemas: {name}")
            }
            GenerationError::MultiplePayloadSchemas { name } => {
                format!("Multiple payload schemas: {name}")
            }
            GenerationError::InputFieldCollision { name } => {
                format!("Input field collision: {name}")
            }
            GenerationError::ClientGroupNameCollision { identifier } => {
                format!("Client group name collision: {identifier}")
            }
            GenerationError::ClientEndpointNameCollision { group, name } => {
                format!("Client endpoint name collision: {group}.{name}")
            }
            GenerationError::ClientNameCollision { name } => {
                format!("Client name collision: {name}")
            }
            GenerationError::UnportableSchema { path } => format!("Unportable schema: {path}"),
            GenerationError::AuthoritativeImportRequired { name } => {
                format!("Effect schema requires authoritative import: {name}")
            }
            GenerationError::UnsupportedPromiseSuccessEncoding { name } => {
                format!("Unsupported Promise success encoding: {name}")
            }
            GenerationError::UnsupportedPromisePathWildcard { path } => {
                format!("Unsupported Promise path wildcard: {path}")
            }
            GenerationError::UnsupportedPromiseStream { name } => {
                format!("Unsupported Promise stream: {name}")
            }
            GenerationError::DuplicateOutputPath { path } => {
                format!("Duplicate output path: {path}")
            }
            GenerationError::UnsafeOutputPath { path } => format!("Unsafe output path: {path}"),
            GenerationError::NotImplemented { topic } => format!("not implemented: {topic}"),
        }
    }
}

impl std::fmt::Display for GenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.reason())
    }
}

impl std::error::Error for GenerationError {}

/// The manifest written by [`write`], tracking owned files.
pub const MANIFEST_NAME: &str = ".httpapi-codegen.json";

/// Minimal filesystem the writer needs. The reference injects a noop
/// `FileSystem`; Rust tests inject a recording fake.
pub trait OutputFs {
    /// Whether a path exists.
    fn exists(&self, _path: &str) -> bool {
        false
    }

    /// Whether a path exists and is a symbolic link.
    fn is_symlink(&self, _path: &str) -> bool {
        false
    }

    /// Read the previous manifest, if any.
    fn read_manifest(&self, _path: &str) -> Result<Option<String>, GenerationError> {
        Ok(None)
    }

    /// Write one file.
    fn write_file(&self, path: &str, content: &str) -> Result<(), GenerationError>;

    /// Remove one file.
    fn remove_file(&self, path: &str) -> Result<(), GenerationError>;
}

/// Reflect an `HttpApi` source into the emitter-independent [`Contract`].
pub fn compile(spec: &ApiSpec, options: &CompileOptions) -> Result<Contract, GenerationError> {
    let _ = (spec, options);
    Err(GenerationError::NotImplemented {
        topic: "codegen compile",
    })
}

/// Emit the zero-Effect Promise client.
pub fn emit_promise(
    contract: &Contract,
    options: &PromiseEmitOptions,
) -> Result<Output, GenerationError> {
    let _ = (contract, options);
    Err(GenerationError::NotImplemented {
        topic: "codegen emit_promise",
    })
}

/// Emit the portable Effect client.
pub fn emit_effect(contract: &Contract) -> Result<Output, GenerationError> {
    let _ = contract;
    Err(GenerationError::NotImplemented {
        topic: "codegen emit_effect",
    })
}

/// Emit an Effect client against an imported authoritative API.
pub fn emit_effect_imported(
    contract: &Contract,
    source: &ImportedSource,
) -> Result<Output, GenerationError> {
    let _ = (contract, source);
    Err(GenerationError::NotImplemented {
        topic: "codegen emit_effect_imported",
    })
}

/// Write an output beneath `directory`, tracking files in the private manifest.
pub fn write(output: &Output, directory: &str, fs: &dyn OutputFs) -> Result<(), GenerationError> {
    let _ = (output, directory, fs);
    Err(GenerationError::NotImplemented {
        topic: "codegen write",
    })
}
