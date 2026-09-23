//! Port of packages/schema/test/event.test.ts (upstream 18ef3cc).

use opencode_schema::event::{define, durable, inventory, latest, DurableSpec};

#[test]
fn definition_is_pure() {
    let definitions = inventory(&[]);
    define("test.pure", None);
    assert!(definitions.is_empty());
}

#[test]
fn latest_selection_is_independent_of_declaration_order() {
    let historical = define(
        "test.versioned",
        Some(DurableSpec {
            aggregate: "id",
            version: 1,
        }),
    );
    let current = define(
        "test.versioned",
        Some(DurableSpec {
            aggregate: "id",
            version: 2,
        }),
    );

    let forward = latest(&[&historical, &current]).unwrap();
    assert!(std::ptr::eq(
        *forward.get("test.versioned").unwrap(),
        &current
    ));

    let reverse = latest(&[&current, &historical]).unwrap();
    assert!(std::ptr::eq(
        *reverse.get("test.versioned").unwrap(),
        &current
    ));
}

#[test]
fn durable_definitions_are_indexed_by_type_and_version() {
    let definition = define(
        "test.durable",
        Some(DurableSpec {
            aggregate: "id",
            version: 1,
        }),
    );

    let index = durable(&[&definition]).unwrap();
    assert!(std::ptr::eq(
        *index.get("test.durable.1").unwrap(),
        &definition
    ));
}
