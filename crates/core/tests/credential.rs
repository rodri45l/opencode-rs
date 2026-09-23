//! Port of packages/core/test/credential.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: create/list/update/remove, updating keeps identity, and
//! creating a replacement drops the previous active credential.

use opencode_core::credential::{
    CredentialCreate, CredentialPatch, CredentialStore, CredentialValue, IntegrationId,
};

#[test]
fn stores_updates_lists_and_removes_credentials() {
    let credentials = CredentialStore::new().unwrap();
    let integration_id = IntegrationId::make("openai");
    let created = credentials
        .create(CredentialCreate {
            integration_id: integration_id.clone(),
            label: Some("Work".into()),
            value: CredentialValue::key("secret"),
        })
        .unwrap();

    assert_eq!(
        credentials.list(&integration_id).unwrap(),
        vec![created.clone()]
    );

    credentials
        .update(
            &created.id,
            CredentialPatch {
                label: Some("Personal".into()),
                value: None,
            },
        )
        .unwrap();
    assert_eq!(
        credentials.list(&integration_id).unwrap()[0]
            .label
            .as_deref(),
        Some("Personal")
    );

    let replacement = credentials
        .create(CredentialCreate {
            integration_id: integration_id.clone(),
            label: Some("Replacement".into()),
            value: CredentialValue::key("replacement"),
        })
        .unwrap();
    assert_eq!(
        credentials.list(&integration_id).unwrap(),
        vec![replacement.clone()]
    );

    credentials.remove(&replacement.id).unwrap();
    assert_eq!(credentials.list(&integration_id).unwrap(), Vec::new());
}
