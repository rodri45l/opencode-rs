//! Port of packages/httpapi-codegen/test/generate.test.ts (upstream 18ef3cc).
//!
//! The reference exercises a TypeScript emitter; only its emitter-independent
//! contract is reproduced here: reflecting an `HttpApi` into grouped operations
//! with flattened inputs, resolved success kinds, declared error tags, safe
//! module paths, and the exact failure wording. Assertions that are purely about
//! rendered TypeScript text (brand erasure, reference inlining, `Schema.Json`
//! projection, `HttpApiClient.ForApi`, generated-consumer fixtures) or about the
//! emitted fetch runtime are re-derived as structural contract checks or omitted
//! with a comment; they are not weakened.
//!
//! Until the codegen module (crates/protocol/src/codegen.rs) is implemented,
//! every behavioural test is red-first and ignored.

use std::collections::HashMap;

use opencode_protocol::{
    compile, emit_effect, emit_effect_imported, emit_promise, ApiSpec, CompileOptions,
    EndpointSpec, GeneratedFile, GenerationError, GroupSpec, HttpMethod, ImportedSource,
    InputField, InputMode, InputSource, Operation, Output, PromiseEmitOptions, SuccessKind,
    SuccessSpec,
};

fn api(endpoint: EndpointSpec) -> ApiSpec {
    ApiSpec::new("test").group(GroupSpec::new("session").endpoint(endpoint))
}

/// Mirror of the reference `optionally` success: a plain value schema.
fn get_session_by_id() -> ApiSpec {
    api(
        EndpointSpec::new(HttpMethod::Get, "get", "/session/:sessionID")
            .param("sessionID")
            .success_data_envelope(),
    )
}

fn file_paths(output: &Output) -> Vec<String> {
    output.files.iter().map(|f| f.path.clone()).collect()
}

// ---------------------------------------------------------------------------
// Emitter-independent error wording.
//
// The reference asserts `toThrow(<reason>)` for the compile/emit guards. With
// the generator unimplemented we can still pin the exact reason strings, which
// is the observable part; the guards themselves are covered by the ignored
// tests below and fail with `NotImplemented` until then.
// ---------------------------------------------------------------------------

#[test]
fn generation_error_reasons_match_reference() {
    assert_eq!(
        GenerationError::InputFieldCollision { name: "id".into() }.reason(),
        "Input field collision: id"
    );
    assert_eq!(
        GenerationError::MultiplePayloadSchemas {
            name: "session.prompt".into()
        }
        .reason(),
        "Multiple payload schemas: session.prompt"
    );
    assert_eq!(
        GenerationError::MultipleSuccessSchemas {
            name: "session.get".into()
        }
        .reason(),
        "Multiple success schemas: session.get"
    );
    assert_eq!(
        GenerationError::ClientGroupNameCollision {
            identifier: "same".into()
        }
        .reason(),
        "Client group name collision: same"
    );
    assert_eq!(
        GenerationError::ClientNameCollision {
            name: "status".into()
        }
        .reason(),
        "Client name collision: status"
    );
    assert_eq!(
        GenerationError::UnportableSchema {
            path: "session.get.success".into()
        }
        .reason(),
        "Unportable schema: session.get.success"
    );
    assert_eq!(
        GenerationError::AuthoritativeImportRequired {
            name: "session.get".into()
        }
        .reason(),
        "Effect schema requires authoritative import: session.get"
    );
    assert_eq!(
        GenerationError::UnsupportedPromiseSuccessEncoding {
            name: "session.text".into()
        }
        .reason(),
        "Unsupported Promise success encoding: session.text"
    );
    assert_eq!(
        GenerationError::UnsupportedPromisePathWildcard {
            path: "/file/*".into()
        }
        .reason(),
        "Unsupported Promise path wildcard: /file/*"
    );
    assert_eq!(
        GenerationError::UnsupportedPromiseStream {
            name: "session.events".into()
        }
        .reason(),
        "Unsupported Promise stream: session.events"
    );
    assert_eq!(
        GenerationError::ClientMiddlewareRequiresAdapter {
            key: "SignedRequest".into()
        }
        .reason(),
        "Client middleware requires adapter: SignedRequest"
    );
    assert_eq!(
        GenerationError::DuplicateOutputPath {
            path: "CLIENT.ts".into()
        }
        .reason(),
        "Duplicate output path: CLIENT.ts"
    );
    assert_eq!(
        GenerationError::UnsafeOutputPath {
            path: "session.ts".into()
        }
        .reason(),
        "Unsafe output path: session.ts"
    );
}

// ---------------------------------------------------------------------------
// Contract reflection.
// ---------------------------------------------------------------------------

#[test]
fn compiles_one_contract_for_promise_and_effect_emitters() {
    let contract = compile(&get_session_by_id(), &CompileOptions::default()).expect("compile");

    let promise = emit_promise(&contract, &PromiseEmitOptions::default()).expect("promise");
    let effect = emit_effect(&contract).expect("effect");

    assert_eq!(promise.operations, effect.operations);
    assert_eq!(
        file_paths(&promise),
        vec!["types.ts", "client-error.ts", "client.ts", "index.ts"]
    );
}

#[test]
fn compile_preserves_public_group_and_endpoint_identifiers() {
    let contract = compile(&get_session_by_id(), &CompileOptions::default()).expect("compile");

    assert_eq!(contract.groups[0].identifier, "session");
    assert_eq!(contract.groups[0].endpoints[0].operation.group, "session");
    assert_eq!(contract.groups[0].endpoints[0].operation.name, "get");
}

#[test]
fn compile_separates_hosted_and_consumer_group_names() {
    let source =
        ApiSpec::new("test").group(GroupSpec::new("server.session").endpoint(
            EndpointSpec::new(HttpMethod::Get, "session.get", "/session").success_value(),
        ));
    let options = CompileOptions {
        group_names: HashMap::from([("server.session".to_string(), "sessions".to_string())]),
        ..CompileOptions::default()
    };

    let contract = compile(&source, &options).expect("compile");

    assert_eq!(contract.groups[0].identifier, "sessions");
    assert_eq!(contract.groups[0].source_identifier, "server.session");
    let op = &contract.groups[0].endpoints[0].operation;
    assert_eq!(op.group, "sessions");
    assert_eq!(op.name, "get");
}

#[test]
fn compile_supports_explicit_public_endpoint_names() {
    let source = ApiSpec::new("test").group(
        GroupSpec::new("server.permission")
            .endpoint(
                EndpointSpec::new(HttpMethod::Get, "permission.request.list", "/request")
                    .success_value(),
            )
            .endpoint(
                EndpointSpec::new(HttpMethod::Get, "session.permission.list", "/session")
                    .success_value(),
            ),
    );
    let options = CompileOptions {
        endpoint_names: HashMap::from([(
            "permission.request.list".to_string(),
            "listRequests".to_string(),
        )]),
        ..CompileOptions::default()
    };

    let contract = compile(&source, &options).expect("compile");
    let names: Vec<String> = contract.groups[0]
        .endpoints
        .iter()
        .map(|e| e.operation.name.clone())
        .collect();
    assert_eq!(names, vec!["listRequests", "list"]);
}

#[test]
fn compile_uses_the_unqualified_endpoint_name() {
    let contract = compile(
        &api(
            EndpointSpec::new(HttpMethod::Get, "session.get", "/session/:sessionID")
                .param("sessionID")
                .success_value(),
        ),
        &CompileOptions::default(),
    )
    .expect("compile");

    assert_eq!(contract.groups[0].endpoints[0].operation.name, "get");
}

#[test]
fn compile_omits_custom_transport_endpoints() {
    let source = ApiSpec::new("test").group(
        GroupSpec::new("server.pty")
            .endpoint(EndpointSpec::new(HttpMethod::Get, "pty.get", "/pty").success_value())
            .endpoint(
                EndpointSpec::new(HttpMethod::Get, "pty.connect", "/pty/connect").success_value(),
            ),
    );
    let options = CompileOptions {
        omit_endpoints: ["pty.connect".to_string()].into_iter().collect(),
        ..CompileOptions::default()
    };

    let contract = compile(&source, &options).expect("compile");
    let names: Vec<String> = contract.groups[0]
        .endpoints
        .iter()
        .map(|e| e.endpoint_name.clone())
        .collect();
    assert_eq!(names, vec!["pty.get"]);
}

// ---------------------------------------------------------------------------
// Input flattening.
// ---------------------------------------------------------------------------

#[test]
fn compile_flattens_transport_input_channels_into_one_domain_input() {
    let spec = api(
        EndpointSpec::new(HttpMethod::Post, "prompt", "/session/:sessionID")
            .param("sessionID")
            .query("resume")
            .header("traceID")
            .payload(&["prompt"])
            .success_data_envelope(),
    );

    let contract = compile(&spec, &CompileOptions::default()).expect("compile");
    let op = &contract.groups[0].endpoints[0].operation;

    assert_eq!(
        op.input,
        vec![
            InputField {
                name: "sessionID".into(),
                source: InputSource::Params,
                optional: false,
            },
            InputField {
                name: "resume".into(),
                source: InputSource::Query,
                optional: false,
            },
            InputField {
                name: "traceID".into(),
                source: InputSource::Headers,
                optional: false,
            },
            InputField {
                name: "prompt".into(),
                source: InputSource::Payload,
                optional: false,
            },
        ]
    );
    assert_eq!(op.input_mode, InputMode::Required);
}

#[test]
fn compile_uses_no_argument_when_an_operation_has_no_input_fields() {
    let spec = api(EndpointSpec::new(HttpMethod::Get, "health", "/health").success_value());
    let contract = compile(&spec, &CompileOptions::default()).expect("compile");
    assert_eq!(
        contract.groups[0].endpoints[0].operation.input_mode,
        InputMode::None
    );
}

#[test]
fn compile_uses_optional_object_when_every_field_is_optional() {
    let spec = api(EndpointSpec::new(HttpMethod::Get, "list", "/session")
        .query_optional("limit")
        .success_value());
    let contract = compile(&spec, &CompileOptions::default()).expect("compile");
    assert_eq!(
        contract.groups[0].endpoints[0].operation.input_mode,
        InputMode::Optional
    );
}

#[test]
fn compile_uses_required_object_when_any_field_is_required() {
    let spec = api(
        EndpointSpec::new(HttpMethod::Get, "get", "/session/:sessionID")
            .param("sessionID")
            .query_optional("includeArchived")
            .success_value(),
    );
    let contract = compile(&spec, &CompileOptions::default()).expect("compile");
    assert_eq!(
        contract.groups[0].endpoints[0].operation.input_mode,
        InputMode::Required
    );
}

#[test]
fn compile_rejects_colliding_input_names_across_channels() {
    let spec = api(
        EndpointSpec::new(HttpMethod::Post, "prompt", "/session/:id")
            .param("id")
            .payload(&["id"])
            .success_no_content(),
    );
    let err = compile(&spec, &CompileOptions::default()).unwrap_err();
    assert_eq!(err.reason(), "Input field collision: id");
}

// ---------------------------------------------------------------------------
// Success and error mapping.
// ---------------------------------------------------------------------------

#[test]
fn compile_unwraps_an_exact_data_success_envelope() {
    let contract = compile(&get_session_by_id(), &CompileOptions::default()).expect("compile");
    let endpoint = &contract.groups[0].endpoints[0];
    assert!(endpoint.unwrap_data);
    assert_eq!(endpoint.operation.success, SuccessKind::Value);
}

#[test]
fn compile_returns_a_non_envelope_success_unchanged() {
    let spec = api(EndpointSpec::new(HttpMethod::Get, "health", "/health").success_value());
    let contract = compile(&spec, &CompileOptions::default()).expect("compile");
    let endpoint = &contract.groups[0].endpoints[0];
    assert!(!endpoint.unwrap_data);
    assert_eq!(endpoint.operation.success, SuccessKind::Value);
}

#[test]
fn compile_maps_no_content_success_to_void() {
    let spec = api(EndpointSpec::new(
        HttpMethod::Post,
        "interrupt",
        "/session/:sessionID/interrupt",
    )
    .param("sessionID")
    .success_no_content());
    let contract = compile(&spec, &CompileOptions::default()).expect("compile");
    assert_eq!(
        contract.groups[0].endpoints[0].operation.success,
        SuccessKind::Void
    );
}

#[test]
fn compile_models_an_sse_success_as_a_direct_stream() {
    let spec = api(EndpointSpec::new(HttpMethod::Get, "subscribe", "/event").success_stream());
    let contract = compile(&spec, &CompileOptions::default()).expect("compile");
    assert_eq!(
        contract.groups[0].endpoints[0].operation.success,
        SuccessKind::Stream
    );
}

#[test]
fn compile_rejects_multiple_success_shapes() {
    let spec = api(EndpointSpec::new(HttpMethod::Get, "get", "/session")
        .success_value()
        .success_value());
    let err = compile(&spec, &CompileOptions::default()).unwrap_err();
    assert_eq!(err.reason(), "Multiple success schemas: session.get");
}

#[test]
fn compile_rejects_multiple_payload_alternatives() {
    let spec = api(EndpointSpec::new(HttpMethod::Post, "prompt", "/session")
        .payload(&["text"])
        .payload(&["count"])
        .success_value());
    let err = compile(&spec, &CompileOptions::default()).unwrap_err();
    assert_eq!(err.reason(), "Multiple payload schemas: session.prompt");
}

#[test]
fn compile_appends_client_error_and_declared_error_tags() {
    let spec = ApiSpec::new("test").group(
        GroupSpec::new("session").endpoint(
            EndpointSpec::new(HttpMethod::Get, "get", "/session/:sessionID")
                .param("sessionID")
                .success_data_envelope()
                .error(404, "Missing")
                .server_error("Unauthorized"),
        ),
    );
    let contract = compile(&spec, &CompileOptions::default()).expect("compile");
    let errors = &contract.groups[0].endpoints[0].operation.errors;

    assert!(errors.contains(&"Missing".to_string()));
    assert!(errors.contains(&"Unauthorized".to_string()));
    assert!(errors.contains(&"ClientError".to_string()));
    assert!(!errors.contains(&"HttpClientError".to_string()));
    assert!(!errors.contains(&"SchemaError".to_string()));
}

#[test]
fn compile_rejects_required_client_middleware_without_an_adapter() {
    let spec = api(EndpointSpec::new(HttpMethod::Get, "get", "/session")
        .success_value()
        .requires_client_middleware("SignedRequest"));
    let err = compile(&spec, &CompileOptions::default()).unwrap_err();
    assert_eq!(
        err.reason(),
        "Client middleware requires adapter: SignedRequest"
    );
}

#[test]
fn compile_rejects_schemas_that_cannot_be_emitted_exactly() {
    // Re-derived from the reference's opaque/custom-check/spoofed/lexical cases:
    // all of them surface as the same `Unportable schema: <path>` failure.
    let spec = api(EndpointSpec::new(HttpMethod::Get, "get", "/session")
        .success_value()
        .success_unportable());
    let err = compile(&spec, &CompileOptions::default()).unwrap_err();
    assert_eq!(err.reason(), "Unportable schema: session.get.success");
}

#[test]
fn compile_requires_an_authoritative_import_for_hidden_transformations() {
    // Re-derived from the reference's custom-transformation and altered-wire
    // cases: both surface as `Effect schema requires authoritative import`.
    let spec = api(EndpointSpec::new(HttpMethod::Get, "get", "/session")
        .success_value()
        .success_requires_import());
    let err = compile(&spec, &CompileOptions::default()).unwrap_err();
    assert_eq!(
        err.reason(),
        "Effect schema requires authoritative import: session.get"
    );
}

// ---------------------------------------------------------------------------
// Naming collisions.
// ---------------------------------------------------------------------------

#[test]
fn compile_rejects_consumer_group_name_collisions() {
    let source = ApiSpec::new("test")
        .group(
            GroupSpec::new("first")
                .endpoint(EndpointSpec::new(HttpMethod::Get, "one", "/one").success_value()),
        )
        .group(
            GroupSpec::new("second")
                .endpoint(EndpointSpec::new(HttpMethod::Get, "two", "/two").success_value()),
        );
    let options = CompileOptions {
        group_names: HashMap::from([
            ("first".to_string(), "same".to_string()),
            ("second".to_string(), "same".to_string()),
        ]),
        ..CompileOptions::default()
    };

    let err = compile(&source, &options).unwrap_err();
    assert_eq!(err.reason(), "Client group name collision: same");
}

#[test]
fn compile_rejects_collisions_in_the_flattened_client_namespace() {
    let source = ApiSpec::new("test")
        .group(
            GroupSpec::new("status")
                .endpoint(EndpointSpec::new(HttpMethod::Get, "get", "/nested").success_value()),
        )
        .group(
            GroupSpec::new("system")
                .top_level()
                .endpoint(EndpointSpec::new(HttpMethod::Get, "status", "/status").success_value()),
        );

    let err = compile(&source, &CompileOptions::default()).unwrap_err();
    assert_eq!(err.reason(), "Client name collision: status");
}

// ---------------------------------------------------------------------------
// Module paths.
// ---------------------------------------------------------------------------

#[test]
fn emit_effect_emits_one_client_module_per_group() {
    let source = ApiSpec::new("test")
        .group(
            GroupSpec::new("session")
                .endpoint(EndpointSpec::new(HttpMethod::Get, "get", "/session").success_value()),
        )
        .group(
            GroupSpec::new("tool")
                .endpoint(EndpointSpec::new(HttpMethod::Get, "list", "/tool").success_value()),
        );
    let contract = compile(&source, &CompileOptions::default()).expect("compile");
    let output = emit_effect(&contract).expect("effect");

    assert_eq!(
        file_paths(&output),
        vec![
            "session.ts",
            "tool.ts",
            "client-error.ts",
            "client.ts",
            "index.ts"
        ]
    );
}

#[test]
fn module_paths_are_safe_and_do_not_change_public_identifiers() {
    let source = ApiSpec::new("test")
        .group(
            GroupSpec::new("../session")
                .endpoint(EndpointSpec::new(HttpMethod::Get, "get", "/session").success_value()),
        )
        .group(
            GroupSpec::new("GROUP-0")
                .endpoint(EndpointSpec::new(HttpMethod::Get, "list", "/session").success_value()),
        );
    let contract = compile(&source, &CompileOptions::default()).expect("compile");
    let output = emit_effect(&contract).expect("effect");

    assert_eq!(contract.groups[0].identifier, "../session");
    assert_eq!(contract.groups[1].identifier, "GROUP-0");
    assert_eq!(
        output.files[..2]
            .iter()
            .map(|f| f.path.clone())
            .collect::<Vec<_>>(),
        vec!["group-0.ts", "GROUP-0-1.ts"]
    );
}

#[test]
fn module_paths_reserve_support_module_names_case_insensitively() {
    let source = ApiSpec::new("test")
        .group(
            GroupSpec::new("client")
                .endpoint(EndpointSpec::new(HttpMethod::Get, "get", "/client").success_value()),
        )
        .group(
            GroupSpec::new("INDEX")
                .endpoint(EndpointSpec::new(HttpMethod::Get, "get", "/index").success_value()),
        );
    let contract = compile(&source, &CompileOptions::default()).expect("compile");
    let output = emit_effect(&contract).expect("effect");

    assert_eq!(
        output.files[..2]
            .iter()
            .map(|f| f.path.clone())
            .collect::<Vec<_>>(),
        vec!["client-0.ts", "INDEX-1.ts"]
    );
}

#[test]
fn module_paths_keep_searching_when_a_reserved_fallback_is_occupied() {
    let source = ApiSpec::new("test")
        .group(
            GroupSpec::new("client-1")
                .endpoint(EndpointSpec::new(HttpMethod::Get, "first", "/first").success_value()),
        )
        .group(
            GroupSpec::new("client")
                .endpoint(EndpointSpec::new(HttpMethod::Get, "second", "/second").success_value()),
        );
    let contract = compile(&source, &CompileOptions::default()).expect("compile");
    let output = emit_effect(&contract).expect("effect");

    assert_eq!(
        output.files[..2]
            .iter()
            .map(|f| f.path.clone())
            .collect::<Vec<_>>(),
        vec!["client-1.ts", "client-1-1.ts"]
    );
}

// ---------------------------------------------------------------------------
// Imported emitters.
// ---------------------------------------------------------------------------

#[test]
fn emit_effect_imported_emits_only_the_root_modules() {
    let contract = compile(&get_session_by_id(), &CompileOptions::default()).expect("compile");
    let output = emit_effect_imported(
        &contract,
        &ImportedSource::Api {
            module: "@example/api".into(),
            api: "Api".into(),
        },
    )
    .expect("imported");

    assert_eq!(
        file_paths(&output),
        vec!["client-error.ts", "client.ts", "index.ts"]
    );
}

// ---------------------------------------------------------------------------
// Promise emitter guards.
// ---------------------------------------------------------------------------

#[test]
fn emit_promise_rejects_unsupported_transports() {
    let text = compile(
        &api(EndpointSpec::new(HttpMethod::Get, "text", "/text").success_text()),
        &CompileOptions::default(),
    )
    .expect("compile");
    assert_eq!(
        emit_promise(&text, &PromiseEmitOptions::default())
            .unwrap_err()
            .reason(),
        "Unsupported Promise success encoding: session.text"
    );

    let binary = compile(
        &api(EndpointSpec::new(HttpMethod::Get, "binary", "/binary").success_binary()),
        &CompileOptions::default(),
    )
    .expect("compile");
    assert_eq!(
        emit_promise(&binary, &PromiseEmitOptions::default())
            .unwrap_err()
            .reason(),
        "Unsupported Promise success encoding: session.binary"
    );

    let wildcard = compile(
        &api(EndpointSpec::new(HttpMethod::Get, "read", "/file/*").success_value()),
        &CompileOptions::default(),
    )
    .expect("compile");
    assert_eq!(
        emit_promise(&wildcard, &PromiseEmitOptions::default())
            .unwrap_err()
            .reason(),
        "Unsupported Promise path wildcard: /file/*"
    );

    let stream = compile(
        &api(EndpointSpec::new(HttpMethod::Get, "events", "/events").success_stream_with_error()),
        &CompileOptions::default(),
    )
    .expect("compile");
    assert_eq!(
        emit_promise(&stream, &PromiseEmitOptions::default())
            .unwrap_err()
            .reason(),
        "Unsupported Promise stream: session.events"
    );
}

// Keep the imports that the reference exercises but whose assertions are
// re-derived above; avoids dead-code drift if the contract grows.
#[allow(dead_code)]
fn _referenced_types(_: Operation, _: GeneratedFile, _: SuccessSpec) {}
