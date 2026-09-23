//! Port of packages/schema/test/compatibility.test.ts (upstream 18ef3cc).

use opencode_schema::filesystem::FindInput;

#[test]
fn moved_class_schemas_remain_constructible() {
    let input = FindInput::new("src");
    assert_eq!(input.query, "src");
    assert!(input.kind.is_none());
    assert!(input.limit.is_none());
}
