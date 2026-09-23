//! Port of packages/console/app/test/requestBody.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/app/src/routes/zen/util/requestBody.ts:
//! the root `model` field is patched (not a nested `model`), the leading chunk
//! is consumed without buffering the rest, streaming is detected after a large
//! message, a late model field buffers only up to that point, stream usage
//! options are appended at the end, and a UTF-8 BOM is preserved.
//! Re-derived: streaming `ReadableStream` reads are represented as an explicit
//! chunk list and a read count; byte-level assertions replace deep JSON equality.

#[allow(dead_code)]
mod request_body {
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: console request body patching not implemented";

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PreparedRequest {
        pub model: String,
        pub reads: usize,
        pub body: Vec<u8>,
    }

    impl PreparedRequest {
        pub fn stream(&self, _provider_model: &str, _include_usage: bool) -> PortResult<Vec<u8>> {
            Err(NotImplemented(NOTE))
        }
    }

    pub fn prepare_request_body(_chunks: Vec<Vec<u8>>) -> PortResult<PreparedRequest> {
        Err(NotImplemented(NOTE))
    }
}

use request_body::{prepare_request_body, NOTE};

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
#[ignore = "porting: console request body patching not implemented"]
fn patches_the_leading_model_without_buffering_the_remaining_body() {
    let chunks = vec![
        br#"{"model":"client-model","stream":true,"messages":["#.to_vec(),
        br#"{"role":"user","content":"large payload"}"#.to_vec(),
        b"]}".to_vec(),
    ];

    let request = prepare_request_body(chunks).expect(NOTE);
    assert_eq!(request.model, "client-model");
    assert_eq!(request.reads, 1);

    let output = text(&request.stream("provider-model", false).expect(NOTE));
    assert!(output.contains("\"model\":\"provider-model\""));
    assert!(output.contains("\"role\":\"user\",\"content\":\"large payload\""));
}

#[test]
#[ignore = "porting: console request body patching not implemented"]
fn ignores_model_fields_nested_before_the_root_model() {
    let chunks = vec![
        br#"{"metadata":{"model":"ox-alpha-free"},"model":"glm-5.3","messages":[],"stream":false}"#
            .to_vec(),
    ];

    let request = prepare_request_body(chunks).expect(NOTE);
    assert_eq!(request.model, "glm-5.3");

    let output = text(&request.stream("provider-model", false).expect(NOTE));
    assert!(output.contains("\"model\":\"provider-model\""));
    assert!(output.contains("\"metadata\":{\"model\":\"ox-alpha-free\"}"));
}

#[test]
#[ignore = "porting: console request body patching not implemented"]
fn appends_stream_usage_options_at_the_end_of_the_request() {
    let chunks = vec![br#"{"model":"client-model","stream":true,"messages":[]}   "#.to_vec()];

    let request = prepare_request_body(chunks).expect(NOTE);
    let output = request.stream("provider-model", true).expect(NOTE);

    let rendered = text(&output);
    assert!(rendered.contains("\"stream_options\":{\"include_usage\":true}"));
    assert!(output.ends_with(b"   "));
}

#[test]
#[ignore = "porting: console request body patching not implemented"]
fn detects_streaming_after_a_large_message_while_forwarding() {
    let content = "x".repeat(128 * 1024);
    let chunks = vec![
        br#"{"model":"client-model","messages":["#.to_vec(),
        format!("{{\"role\":\"user\",\"content\":\"{content}\"}}").into_bytes(),
        br#"],"stream":true}"#.to_vec(),
    ];

    let request = prepare_request_body(chunks).expect(NOTE);
    assert_eq!(request.reads, 1);

    let output = text(&request.stream("provider-model", true).expect(NOTE));
    assert!(output.contains("\"model\":\"provider-model\""));
    assert!(output.contains("\"stream_options\":{\"include_usage\":true}"));
}

#[test]
#[ignore = "porting: console request body patching not implemented"]
fn buffers_through_a_late_model_field_and_then_streams_the_rest() {
    let content = "こんにちは".repeat(32 * 1024);
    let chunks = vec![
        br#"{"messages":["#.to_vec(),
        format!("{{\"role\":\"user\",\"content\":\"{content}\"}}").into_bytes(),
        br#"],"model":"client-model","stream":true,"extra":"after-model"}"#.to_vec(),
    ];

    let request = prepare_request_body(chunks).expect(NOTE);
    assert_eq!(request.model, "client-model");
    assert_eq!(request.reads, 3);

    let output = text(&request.stream("provider-model", true).expect(NOTE));
    assert!(output.contains("\"model\":\"provider-model\""));
    assert!(output.contains("\"extra\":\"after-model\""));
    assert!(output.contains("\"stream_options\":{\"include_usage\":true}"));
}

#[test]
#[ignore = "porting: console request body patching not implemented"]
fn preserves_a_utf8_bom_while_patching_the_model() {
    let mut body = vec![0xef, 0xbb, 0xbf];
    body.extend_from_slice(br#"{"messages":[],"model":"client-model","stream":false}"#);
    let chunks = vec![body];

    let request = prepare_request_body(chunks).expect(NOTE);
    let output = request.stream("provider-model", false).expect(NOTE);

    assert_eq!(output[0..3].to_vec(), vec![0xef, 0xbb, 0xbf]);
    assert!(text(&output).contains("\"model\":\"provider-model\""));
}
