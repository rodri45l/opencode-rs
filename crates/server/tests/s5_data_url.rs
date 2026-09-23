//! Port of packages/opencode/test/util/data-url.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `decodeDataUrl` decodes both base64 and percent-encoded
//! plain data URLs into their UTF-8 body.
#![allow(dead_code)]

use opencode_server::data_url::decode_data_url;

#[test]
fn decodes_base64_data_urls() {
    let body = "{\n  \"ok\": true\n}\n";
    // base64 of the body above.
    let url = "data:text/plain;base64,ewogICJvayI6IHRydWUKfQo=";
    assert_eq!(decode_data_url(url), body);
}

#[test]
fn decodes_plain_data_urls() {
    assert_eq!(
        decode_data_url("data:text/plain,hello%20world"),
        "hello world"
    );
}
