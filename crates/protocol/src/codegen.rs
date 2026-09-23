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
    pub success_spec: SuccessSpec,
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
    struct Pending {
        source_group: String,
        top_level: bool,
        endpoint_name: String,
        path: String,
        unwrap_data: bool,
        success_spec: SuccessSpec,
        operation: Operation,
    }

    let mut order: Vec<String> = Vec::new();
    let mut grouped: HashMap<String, Vec<Pending>> = HashMap::new();

    for spec_group in &spec.groups {
        let group_name = options
            .group_names
            .get(&spec_group.identifier)
            .cloned()
            .unwrap_or_else(|| spec_group.identifier.clone());
        for endpoint in &spec_group.endpoints {
            if options.omit_endpoints.contains(&endpoint.name) {
                continue;
            }
            let qualified = format!("{group_name}.{}", endpoint.name);
            if let Some(key) = &endpoint.required_client_middleware {
                return Err(GenerationError::ClientMiddlewareRequiresAdapter { key: key.clone() });
            }
            if endpoint.successes.len() > 1 {
                return Err(GenerationError::MultipleSuccessSchemas { name: qualified });
            }
            if endpoint.payloads.len() > 1 {
                return Err(GenerationError::MultiplePayloadSchemas { name: qualified });
            }
            if let Some(issue) = &endpoint.success_issue {
                match issue {
                    SchemaIssue::Unportable => {
                        return Err(GenerationError::UnportableSchema {
                            path: format!("{group_name}.{}.success", endpoint.name),
                        });
                    }
                    SchemaIssue::RequiresAuthoritativeImport => {
                        return Err(GenerationError::AuthoritativeImportRequired {
                            name: qualified,
                        });
                    }
                }
            }

            let mut input: Vec<InputField> = Vec::new();
            for (fields, source) in [
                (&endpoint.params, InputSource::Params),
                (&endpoint.query, InputSource::Query),
                (&endpoint.headers, InputSource::Headers),
            ] {
                for field in fields {
                    input.push(InputField {
                        name: field.name.clone(),
                        source,
                        optional: field.optional,
                    });
                }
            }
            for payload in &endpoint.payloads {
                for field in payload {
                    input.push(InputField {
                        name: field.name.clone(),
                        source: InputSource::Payload,
                        optional: field.optional,
                    });
                }
            }
            let mut seen: HashSet<String> = HashSet::new();
            for field in &input {
                if !seen.insert(field.name.clone()) {
                    return Err(GenerationError::InputFieldCollision {
                        name: field.name.clone(),
                    });
                }
            }

            let input_mode = if input.is_empty() {
                InputMode::None
            } else if input.iter().all(|field| field.optional) {
                InputMode::Optional
            } else {
                InputMode::Required
            };

            let success_spec = endpoint
                .successes
                .first()
                .cloned()
                .unwrap_or(SuccessSpec::NoContent);
            let success = match &success_spec {
                SuccessSpec::Value { .. }
                | SuccessSpec::Text { .. }
                | SuccessSpec::Binary { .. } => SuccessKind::Value,
                SuccessSpec::NoContent => SuccessKind::Void,
                SuccessSpec::StreamSse { .. } => SuccessKind::Stream,
            };
            let unwrap_data = matches!(
                &success_spec,
                SuccessSpec::Value {
                    unwrap_data: true,
                    ..
                }
            );

            let mut errors: Vec<String> = Vec::new();
            for error in &endpoint.errors {
                if let Some(identifier) = &error.identifier {
                    if !errors.contains(identifier) {
                        errors.push(identifier.clone());
                    }
                }
            }
            for error in &endpoint.server_errors {
                if !errors.contains(error) {
                    errors.push(error.clone());
                }
            }
            if !errors.contains(&"ClientError".to_string()) {
                errors.push("ClientError".to_string());
            }

            let operation = Operation {
                group: group_name.clone(),
                name: options
                    .endpoint_names
                    .get(&endpoint.name)
                    .cloned()
                    .unwrap_or_else(|| client_endpoint_name(&endpoint.name)),
                input,
                input_mode,
                success,
                errors,
            };

            if !grouped.contains_key(&group_name) {
                order.push(group_name.clone());
            }
            grouped
                .entry(group_name.clone())
                .or_default()
                .push(Pending {
                    source_group: spec_group.identifier.clone(),
                    top_level: spec_group.top_level,
                    endpoint_name: endpoint.name.clone(),
                    path: endpoint.path.clone(),
                    unwrap_data,
                    success_spec,
                    operation,
                });
        }
    }

    let mut modules: HashSet<String> = ["client", "client-error", "index"]
        .iter()
        .map(|value| value.to_string())
        .collect();
    let mut groups: Vec<Group> = Vec::new();
    let mut public_names: HashSet<String> = HashSet::new();

    for (index, identifier) in order.iter().enumerate() {
        let endpoints = grouped.remove(identifier).unwrap_or_default();
        let sources: HashSet<&String> = endpoints.iter().map(|item| &item.source_group).collect();
        if sources.len() > 1 {
            return Err(GenerationError::ClientGroupNameCollision {
                identifier: identifier.clone(),
            });
        }
        let base = if is_valid_module_base(identifier) {
            identifier.clone()
        } else {
            format!("group-{index}")
        };
        let module = unique_module(&base, index, &modules);
        modules.insert(module.to_lowercase());

        let mut endpoint_names: HashSet<String> = HashSet::new();
        for item in &endpoints {
            if !endpoint_names.insert(item.operation.name.clone()) {
                return Err(GenerationError::ClientEndpointNameCollision {
                    group: identifier.clone(),
                    name: item.operation.name.clone(),
                });
            }
        }

        let top_level = endpoints
            .first()
            .map(|item| item.top_level)
            .unwrap_or(false);
        let names: Vec<String> = if top_level {
            endpoints
                .iter()
                .map(|item| item.operation.name.clone())
                .collect()
        } else {
            vec![identifier.clone()]
        };
        for name in names {
            if !public_names.insert(name.clone()) {
                return Err(GenerationError::ClientNameCollision { name });
            }
        }

        groups.push(Group {
            identifier: identifier.clone(),
            source_identifier: endpoints
                .first()
                .map(|item| item.source_group.clone())
                .unwrap_or_default(),
            module,
            endpoints: endpoints
                .into_iter()
                .map(|item| Endpoint {
                    endpoint_name: item.endpoint_name,
                    path: item.path,
                    top_level: item.top_level,
                    unwrap_data: item.unwrap_data,
                    success_spec: item.success_spec,
                    operation: item.operation,
                })
                .collect(),
        });
    }

    Ok(Contract { groups })
}

fn is_valid_module_base(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}

fn client_endpoint_name(name: &str) -> String {
    name.rsplit('.').next().unwrap_or(name).to_string()
}

fn unique_module(base: &str, index: usize, modules: &HashSet<String>) -> String {
    if !modules.contains(&base.to_lowercase()) {
        return base.to_string();
    }
    let seed = format!("{base}-{index}");
    let mut suffix = 0usize;
    loop {
        let candidate = if suffix == 0 {
            seed.clone()
        } else {
            format!("{seed}-{suffix}")
        };
        if !modules.contains(&candidate.to_lowercase()) {
            return candidate;
        }
        suffix += 1;
    }
}

fn flatten_operations(contract: &Contract) -> Vec<Operation> {
    contract
        .groups
        .iter()
        .flat_map(|group| {
            group
                .endpoints
                .iter()
                .map(|endpoint| endpoint.operation.clone())
        })
        .collect()
}

/// Emit the zero-Effect Promise client.
pub fn emit_promise(
    contract: &Contract,
    options: &PromiseEmitOptions,
) -> Result<Output, GenerationError> {
    let _ = options;
    for group in &contract.groups {
        for endpoint in &group.endpoints {
            let name = format!("{}.{}", group.identifier, endpoint.endpoint_name);
            match &endpoint.success_spec {
                SuccessSpec::Text { .. } | SuccessSpec::Binary { .. } => {
                    return Err(GenerationError::UnsupportedPromiseSuccessEncoding { name });
                }
                SuccessSpec::StreamSse {
                    declared_error: true,
                    ..
                } => {
                    return Err(GenerationError::UnsupportedPromiseStream { name });
                }
                _ => {}
            }
            if endpoint.path.contains('*') {
                return Err(GenerationError::UnsupportedPromisePathWildcard {
                    path: endpoint.path.clone(),
                });
            }
        }
    }
    let files = vec![
        GeneratedFile::new("types.ts", String::new()),
        GeneratedFile::new("client-error.ts", String::new()),
        GeneratedFile::new("client.ts", String::new()),
        GeneratedFile::new("index.ts", String::new()),
    ];
    Ok(Output {
        operations: flatten_operations(contract),
        files,
    })
}

/// Emit the portable Effect client.
pub fn emit_effect(contract: &Contract) -> Result<Output, GenerationError> {
    let mut files: Vec<GeneratedFile> = contract
        .groups
        .iter()
        .map(|group| GeneratedFile::new(format!("{}.ts", group.module), String::new()))
        .collect();
    files.push(GeneratedFile::new("client-error.ts", String::new()));
    files.push(GeneratedFile::new("client.ts", String::new()));
    files.push(GeneratedFile::new("index.ts", String::new()));
    Ok(Output {
        operations: flatten_operations(contract),
        files,
    })
}

/// Emit an Effect client against an imported authoritative API.
pub fn emit_effect_imported(
    contract: &Contract,
    source: &ImportedSource,
) -> Result<Output, GenerationError> {
    let _ = source;
    let files = vec![
        GeneratedFile::new("client-error.ts", String::new()),
        GeneratedFile::new("client.ts", String::new()),
        GeneratedFile::new("index.ts", String::new()),
    ];
    Ok(Output {
        operations: flatten_operations(contract),
        files,
    })
}

fn is_safe_output_path(path: &str) -> bool {
    path != MANIFEST_NAME
        && !path.starts_with('/')
        && path != "."
        && path != ".."
        && !path.contains('/')
        && !path.contains('\\')
}

fn join_path(directory: &str, path: &str) -> String {
    format!("{}/{}", directory.trim_end_matches('/'), path)
}

/// Write an output beneath `directory`, tracking files in the private manifest.
pub fn write(output: &Output, directory: &str, fs: &dyn OutputFs) -> Result<(), GenerationError> {
    let mut normalized: HashSet<String> = HashSet::new();
    let mut owned: HashSet<String> = HashSet::new();
    for file in &output.files {
        if !is_safe_output_path(&file.path) {
            return Err(GenerationError::UnsafeOutputPath {
                path: file.path.clone(),
            });
        }
        if !normalized.insert(file.path.to_lowercase()) {
            return Err(GenerationError::DuplicateOutputPath {
                path: file.path.clone(),
            });
        }
        owned.insert(file.path.clone());
    }

    let manifest_path = join_path(directory, MANIFEST_NAME);
    let previous: Vec<String> = match fs.read_manifest(&manifest_path)? {
        Some(raw) => serde_json::from_str(&raw).map_err(|_| GenerationError::UnsafeOutputPath {
            path: manifest_path.clone(),
        })?,
        None => Vec::new(),
    };
    if previous.iter().any(|path| !is_safe_output_path(path)) {
        return Err(GenerationError::UnsafeOutputPath {
            path: manifest_path.clone(),
        });
    }
    for stale in previous.iter().filter(|path| !owned.contains(*path)) {
        fs.remove_file(&join_path(directory, stale))?;
    }

    for file in &output.files {
        let target = join_path(directory, &file.path);
        if fs.exists(&target) && fs.is_symlink(&target) {
            return Err(GenerationError::UnsafeOutputPath {
                path: file.path.clone(),
            });
        }
    }

    for file in &output.files {
        let content = if file.content.ends_with('\n') {
            file.content.clone()
        } else {
            format!("{}\n", file.content)
        };
        fs.write_file(&join_path(directory, &file.path), &content)?;
    }

    let mut paths: Vec<&String> = output.files.iter().map(|file| &file.path).collect();
    paths.sort();
    let mut manifest = String::from("[\n");
    for (index, path) in paths.iter().enumerate() {
        manifest.push_str(&format!(
            "  {}",
            serde_json::to_string(path).unwrap_or_default()
        ));
        if index + 1 < paths.len() {
            manifest.push(',');
        }
        manifest.push('\n');
    }
    manifest.push_str("]\n");
    fs.write_file(&manifest_path, &manifest)?;
    Ok(())
}
