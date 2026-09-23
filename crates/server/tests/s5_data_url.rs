//! Port of packages/opencode/test/util/data-url.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `decodeDataUrl` decodes both base64 and percent-encoded
//! plain data URLs into their UTF-8 body.
#![allow(dead_code)]

// Fast-wave local stubs: `util::data_url` is not implemented in this crate yet.
mod data_url {
    pub fn decode_data_url(_url: &str) -> Result<String, &'static str> {
        Err("porting: data-url::decodeDataUrl not implemented")
    }
}

#[test]
#[ignore = "porting: data-url not implemented"]
fn decodes_base64_data_urls() {
    let body = "{\n  \"ok\": true\n}\n";
    // base64 of the body above.
    let url = "data:text/plain;base64,ewogICJvayI6IHRydWUKfQo=";
    assert_eq!(data_url::decode_data_url(url).unwrap(), body);
}

#[test]
#[ignore = "porting: data-url not implemented"]
fn decodes_plain_data_urls() {
    assert_eq!(
        data_url::decode_data_url("data:text/plain,hello%20world").unwrap(),
        "hello world"
    );
}
